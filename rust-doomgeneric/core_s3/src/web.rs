//! The web controller: the board serves a page (`assets/controller.html`) on port 80, and the page
//! sends key events back over a WebSocket on `/ws`. Every byte of a WebSocket message is one
//! `core_s3_protocol` event, exactly like a byte on the plain TCP connection (`net`).
//!
//! Three tasks run this. A task that is serving a page or holding a WebSocket is not listening, so
//! with only one or two a browser that loads the page and then opens its WebSocket (or a second tab)
//! could find nobody listening and be refused for a moment.

use core::fmt::Write as _;

use core_s3_protocol::HeldKeys;
use core_s3_ws::{
    handshake_response, head_length, parse_request, pong_frame, Decoder, Output, Request,
    CLOSE_FRAME, MAX_CONTROL_PAYLOAD, MAX_HEAD,
};
use embassy_net::{
    tcp::{Error, TcpSocket},
    Stack,
};
use embassy_time::Duration;
use esp_println::println;
use heapless::String;

use crate::net;

const PAGE: &str = include_str!("../assets/controller.html");
const PORT: u16 = 80;

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
    let result = 'connection: loop {
        let read = match socket.read(&mut bytes).await {
            Ok(0) => break Ok(()),
            Ok(read) => read,
            Err(err) => break Err(err),
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

async fn write_all(socket: &mut TcpSocket<'_>, mut data: &[u8]) -> Result<(), Error> {
    while !data.is_empty() {
        let written = socket.write(data).await?;
        data = &data[written..];
    }
    Ok(())
}
