//! The web controller: the board serves a page (`assets/controller.html`) on port 80, and the page
//! sends key events back over a WebSocket on `/ws`. Every byte of a WebSocket message is one
//! `core_s3_protocol` event, exactly like a byte on the plain TCP connection (`net`).
//!
//! The other way, the board pushes its frame rate to every connected page as a WebSocket text
//! message (`core_s3_protocol::encode_fps`) whenever the game has measured a new one (see
//! `platform::fps_sample`; about once a second).
//!
//! Three tasks run this. A task that is serving a page or holding a WebSocket is not listening, so
//! with only one or two a browser that loads the page and then opens its WebSocket (or a second tab)
//! could find nobody listening and be refused for a moment.

use core::fmt::Write as _;

use core_s3_protocol::{encode_fps, HeldKeys, FPS_MESSAGE_MAX};
use core_s3_ws::{
    handshake_response, head_length, parse_request, pong_frame, text_frame, Decoder, Output,
    Request, CLOSE_FRAME, MAX_CONTROL_PAYLOAD, MAX_HEAD, MAX_TEXT_PAYLOAD,
};
use embassy_futures::select::{select, Either};
use embassy_net::{
    tcp::{Error, TcpSocket},
    Stack,
};
use embassy_time::{Duration, Instant, Timer};
use esp_println::println;
use heapless::String;

use crate::{net, platform};

const PAGE: &str = include_str!("../assets/controller.html");
const PORT: u16 = 80;
/// How often a connection looks for a new frame rate sample to send.
const FPS_POLL: Duration = Duration::from_millis(500);

#[embassy_executor::task(pool_size = 3)]
pub async fn web_server(stack: Stack<'static>) {
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let mut head = [0u8; MAX_HEAD];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        // A phone that goes to sleep or leaves the network must not hold a slot forever.
        socket.set_keep_alive(Some(Duration::from_secs(5)));
        socket.set_timeout(Some(Duration::from_secs(20)));
        if let Err(err) = socket.accept(PORT).await {
            println!("web: accept failed: {err:?}");
            continue;
        }
        if let Err(err) = serve(&mut socket, &mut head).await {
            println!("web: connection ended: {err:?}");
        }
        // Get the last bytes out before the socket is dropped (which would reset the connection).
        let _ = socket.flush().await;
        socket.close();
        let _ = socket.flush().await;
    }
}

async fn serve(socket: &mut TcpSocket<'_>, head: &mut [u8; MAX_HEAD]) -> Result<(), Error> {
    let mut length = 0;
    let end = loop {
        if let Some(end) = head_length(&head[..length]) {
            break end;
        }
        if length == head.len() {
            return respond(socket, "431 Request Header Fields Too Large", "text/plain", b"").await;
        }
        let read = socket.read(&mut head[length..]).await?;
        if read == 0 {
            return Ok(());
        }
        length += read;
    };
    match parse_request(&head[..end]) {
        Request::WebSocket { key } => websocket(socket, key).await,
        Request::Get { path: "/" | "/index.html" } => {
            respond(socket, "200 OK", "text/html; charset=utf-8", PAGE.as_bytes()).await
        }
        Request::Get { path: "/favicon.ico" } => respond(socket, "204 No Content", "text/plain", b"").await,
        Request::Get { .. } => respond(socket, "404 Not Found", "text/plain", b"not found").await,
        Request::Other => respond(socket, "405 Method Not Allowed", "text/plain", b"").await,
    }
}

async fn respond(
    socket: &mut TcpSocket<'_>,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), Error> {
    let mut header = String::<192>::new();
    let _ = write!(
        header,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Cache-Control: no-cache\r\nConnection: close\r\n\r\n",
        body.len()
    );
    write_all(socket, header.as_bytes()).await?;
    write_all(socket, body).await
}

/// Turns a WebSocket upgrade into a stream of key events, until the browser goes away.
async fn websocket(socket: &mut TcpSocket<'_>, key: &str) -> Result<(), Error> {
    let mut response = [0u8; 160];
    let length = handshake_response(key, &mut response);
    write_all(socket, &response[..length]).await?;
    println!("web: controller connected: {:?}", socket.remote_endpoint());

    let mut decoder = Decoder::new();
    let mut held = HeldKeys::default();
    let mut bytes = [0u8; 64];
    // The counter of the last frame rate sample sent (none yet, so a connection that opens after
    // the first sample gets it at once).
    let mut fps_sent = None;
    let mut next_poll = Instant::now();
    let result = 'connection: loop {
        if Instant::now() >= next_poll {
            next_poll = Instant::now() + FPS_POLL;
            if let Err(err) = send_fps(socket, &mut fps_sent).await {
                break Err(err);
            }
        }
        // Wait for the browser, but not past the next look at the frame rate. The deadline is fixed
        // (not restarted per read) so a busy stream of key events cannot starve the frame rate.
        let read = match select(socket.read(&mut bytes), Timer::at(next_poll)).await {
            Either::First(Ok(0)) => break Ok(()),
            Either::First(Ok(read)) => read,
            Either::First(Err(err)) => break Err(err),
            Either::Second(()) => continue,
        };
        for &byte in &bytes[..read] {
            match decoder.push(byte) {
                Some(Output::Data(byte)) => net::handle_command_byte(byte, &mut held),
                Some(Output::Ping) => {
                    let mut pong = [0u8; 2 + MAX_CONTROL_PAYLOAD];
                    let length = pong_frame(decoder.ping_payload(), &mut pong);
                    if let Err(err) = write_all(socket, &pong[..length]).await {
                        break 'connection Err(err);
                    }
                }
                Some(Output::Close) => {
                    let _ = write_all(socket, &CLOSE_FRAME).await;
                    break 'connection Ok(());
                }
                Some(Output::Error) => break 'connection Ok(()),
                None => {}
            }
        }
    };
    // Whatever the browser was holding down, let go of it.
    net::release_held(&mut held);
    println!("web: controller disconnected");
    result
}

/// Sends the latest frame rate to the browser if the game has measured a new one since `sent`.
async fn send_fps(socket: &mut TcpSocket<'_>, sent: &mut Option<u16>) -> Result<(), Error> {
    let Some((counter, tenths)) = platform::fps_sample() else { return Ok(()) };
    if *sent == Some(counter) {
        return Ok(());
    }
    *sent = Some(counter);
    let mut message = [0u8; FPS_MESSAGE_MAX];
    let length = encode_fps(u32::from(tenths), &mut message);
    let mut frame = [0u8; 2 + MAX_TEXT_PAYLOAD];
    let length = text_frame(&message[..length], &mut frame);
    write_all(socket, &frame[..length]).await
}

async fn write_all(socket: &mut TcpSocket<'_>, mut data: &[u8]) -> Result<(), Error> {
    while !data.is_empty() {
        let written = socket.write(data).await?;
        data = &data[written..];
    }
    Ok(())
}
