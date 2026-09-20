//! Just enough HTTP and WebSocket (RFC 6455) for the CoreS3's web controller: read a request, hand
//! out the page or upgrade to a WebSocket, and decode the frames a browser sends. `no_std`, and
//! independent of the network stack, so it is tested on the host.
//!
//! A browser sends key events as WebSocket messages; each payload byte is one
//! `core_s3_protocol` event, exactly like a byte on the plain TCP connection.
//!
//! What it leaves out on purpose: fragmented messages need no special care here (every payload
//! byte stands alone), and frames longer than 64 KiB are refused.

#![no_std]

use sha1_smol::Sha1;

/// The most a request head (everything up to the blank line) may take.
pub const MAX_HEAD: usize = 2048;

/// The path a browser opens the WebSocket on.
pub const WEBSOCKET_PATH: &str = "/ws";

/// What a request asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Request<'a> {
    /// A WebSocket upgrade on [`WEBSOCKET_PATH`]; `key` is the client's `Sec-WebSocket-Key`.
    WebSocket { key: &'a str },
    /// An ordinary `GET` of `path` (without any query).
    Get { path: &'a str },
    /// Anything else.
    Other,
}

/// Where the request head ends in `bytes` (the index just past the blank line), if it has arrived.
pub fn head_length(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|at| at + 4)
}

/// Whether a request head (as delimited by [`head_length`]) says the client accepts gzip-compressed
/// bodies: an `Accept-Encoding` header listing `gzip` (any case) that is not refused with `q=0`.
pub fn accepts_gzip(head: &[u8]) -> bool {
    let Ok(head) = core::str::from_utf8(head) else {
        return false;
    };
    head.split("\r\n").skip(1).any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        name.trim().eq_ignore_ascii_case("accept-encoding")
            && value.split(',').any(|coding| {
                let mut parts = coding.split(';');
                let is_gzip = parts
                    .next()
                    .is_some_and(|token| token.trim().eq_ignore_ascii_case("gzip"));
                // `q=0`, `q=0.0`, ... mean "not acceptable".
                let refused = parts.any(|param| {
                    param
                        .trim()
                        .strip_prefix("q=")
                        .is_some_and(|q| !q.is_empty() && q.chars().all(|c| c == '0' || c == '.'))
                });
                is_gzip && !refused
            })
    })
}

/// Parses a request head, as delimited by [`head_length`].
pub fn parse_request(head: &[u8]) -> Request<'_> {
    let Ok(head) = core::str::from_utf8(head) else {
        return Request::Other;
    };
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next().unwrap_or("").split(' ');
    let (Some("GET"), Some(target), Some(version)) = (
        request_line.next(),
        request_line.next(),
        request_line.next(),
    ) else {
        return Request::Other;
    };
    if !version.starts_with("HTTP/1.") {
        return Request::Other;
    }
    let path = target.split(['?', '#']).next().unwrap_or(target);

    let mut upgrade = false;
    let mut connection_upgrade = false;
    let mut key = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        if name.eq_ignore_ascii_case("upgrade") {
            upgrade = value.eq_ignore_ascii_case("websocket");
        } else if name.eq_ignore_ascii_case("connection") {
            connection_upgrade = value
                .split(',')
                .any(|token| token.trim().eq_ignore_ascii_case("upgrade"));
        } else if name.eq_ignore_ascii_case("sec-websocket-key") {
            key = Some(value);
        }
    }
    match key {
        Some(key) if upgrade && connection_upgrade && path == WEBSOCKET_PATH => {
            Request::WebSocket { key }
        }
        _ => Request::Get { path },
    }
}

/// `Sec-WebSocket-Accept` for a client key: base64 of the SHA-1 of the key and a fixed GUID.
pub fn accept_key(key: &str) -> [u8; 28] {
    let mut sha = Sha1::new();
    sha.update(key.as_bytes());
    sha.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64(&sha.digest().bytes())
}

/// Standard base64 with padding, for the 20 bytes of a SHA-1 digest.
fn base64(digest: &[u8; 20]) -> [u8; 28] {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = [b'='; 28];
    for (group, chunk) in digest.chunks(3).enumerate() {
        let bits = chunk
            .iter()
            .fold(0u32, |bits, &byte| bits << 8 | u32::from(byte))
            << (8 * (3 - chunk.len()));
        for i in 0..=chunk.len() {
            out[group * 4 + i] = ALPHABET[(bits >> (18 - 6 * i) & 63) as usize];
        }
    }
    out
}

/// The `101 Switching Protocols` answer to a WebSocket upgrade with `key`. Returns its length.
pub fn handshake_response(key: &str, out: &mut [u8; 160]) -> usize {
    let mut length = 0;
    for part in [
        &b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"[..],
        b"Sec-WebSocket-Accept: ",
        &accept_key(key),
        b"\r\n\r\n",
    ] {
        out[length..length + part.len()].copy_from_slice(part);
        length += part.len();
    }
    length
}

/// A WebSocket close frame with no reason.
pub const CLOSE_FRAME: [u8; 2] = [0x88, 0x00];

/// The longest payload a control frame (ping, pong, close) may carry.
pub const MAX_CONTROL_PAYLOAD: usize = 125;

/// Builds the pong that answers a ping with `payload` (which must be echoed back). Returns its
/// length.
pub fn pong_frame(payload: &[u8], out: &mut [u8; 2 + MAX_CONTROL_PAYLOAD]) -> usize {
    let payload = &payload[..payload.len().min(MAX_CONTROL_PAYLOAD)];
    out[0] = 0x8a;
    out[1] = payload.len() as u8;
    out[2..2 + payload.len()].copy_from_slice(payload);
    2 + payload.len()
}

/// The longest payload [`text_frame`] takes: what fits the 7-bit length field.
pub const MAX_TEXT_PAYLOAD: usize = 125;

/// Builds an unmasked WebSocket text message (a single frame) carrying `payload`, as a server
/// sends it to a browser, which delivers it to the page's `onmessage`. `payload` must be valid
/// UTF-8 and at most [`MAX_TEXT_PAYLOAD`] bytes (longer is cut off). Returns the length.
pub fn text_frame(payload: &[u8], out: &mut [u8; 2 + MAX_TEXT_PAYLOAD]) -> usize {
    let payload = &payload[..payload.len().min(MAX_TEXT_PAYLOAD)];
    out[0] = 0x81;
    out[1] = payload.len() as u8;
    out[2..2 + payload.len()].copy_from_slice(payload);
    2 + payload.len()
}

/// What the decoder found while reading the bytes a client sent.
#[derive(Debug, PartialEq, Eq)]
pub enum Output {
    /// One byte of a text or binary message payload, already unmasked.
    Data(u8),
    /// The client sent a ping: answer with [`pong_frame`] of [`Decoder::ping_payload`].
    Ping,
    /// The client is closing: answer with [`CLOSE_FRAME`] and stop.
    Close,
    /// Something this server does not handle (an oversized frame): drop the connection.
    Error,
}

#[derive(Clone, Copy)]
enum State {
    /// Waiting for the FIN and opcode byte.
    Start,
    /// Waiting for the mask bit and the 7-bit length.
    Length,
    /// Reading the 2-byte extended length; `have` bytes so far.
    ExtendedLength {
        have: u8,
    },
    MaskKey {
        have: u8,
    },
    Payload,
    /// A frame that failed; every further byte is ignored.
    Failed,
}

/// Decodes the frames of a client one byte at a time, so it does not matter how the bytes were
/// split across reads.
pub struct Decoder {
    state: State,
    opcode: u8,
    masked: bool,
    length: usize,
    mask: [u8; 4],
    /// How many payload bytes have been seen (for unmasking).
    seen: usize,
    /// The payload of the latest ping, for the pong.
    ping: [u8; MAX_CONTROL_PAYLOAD],
    ping_length: usize,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub const fn new() -> Self {
        Self {
            state: State::Start,
            opcode: 0,
            masked: false,
            length: 0,
            mask: [0; 4],
            seen: 0,
            ping: [0; MAX_CONTROL_PAYLOAD],
            ping_length: 0,
        }
    }

    /// The payload of the ping that [`Output::Ping`] just reported.
    pub fn ping_payload(&self) -> &[u8] {
        &self.ping[..self.ping_length]
    }

    /// Feeds the next received byte.
    pub fn push(&mut self, byte: u8) -> Option<Output> {
        match self.state {
            State::Failed => None,
            State::Start => {
                self.opcode = byte & 0x0f;
                self.state = State::Length;
                None
            }
            State::Length => {
                self.masked = byte & 0x80 != 0;
                match byte & 0x7f {
                    126 => {
                        self.length = 0;
                        self.state = State::ExtendedLength { have: 0 };
                        None
                    }
                    127 => self.fail(),
                    length => {
                        self.length = usize::from(length);
                        self.after_length()
                    }
                }
            }
            State::ExtendedLength { have } => {
                self.length = self.length << 8 | usize::from(byte);
                if have == 1 {
                    self.after_length()
                } else {
                    self.state = State::ExtendedLength { have: have + 1 };
                    None
                }
            }
            State::MaskKey { have } => {
                self.mask[usize::from(have)] = byte;
                if have == 3 {
                    self.after_mask()
                } else {
                    self.state = State::MaskKey { have: have + 1 };
                    None
                }
            }
            State::Payload => {
                let byte = if self.masked {
                    byte ^ self.mask[self.seen % 4]
                } else {
                    byte
                };
                if self.opcode == 9 && self.seen < MAX_CONTROL_PAYLOAD {
                    self.ping[self.seen] = byte;
                }
                self.seen += 1;
                let output = match self.opcode {
                    0..=2 => Some(Output::Data(byte)),
                    _ => None,
                };
                if self.seen == self.length {
                    self.state = State::Start;
                    // A ping or close carries its payload before it is complete.
                    return output.or_else(|| self.finished_control_frame());
                }
                output
            }
        }
    }

    fn after_length(&mut self) -> Option<Output> {
        if self.masked {
            self.state = State::MaskKey { have: 0 };
            None
        } else {
            self.after_mask()
        }
    }

    fn after_mask(&mut self) -> Option<Output> {
        self.seen = 0;
        if self.opcode == 9 {
            self.ping_length = self.length.min(MAX_CONTROL_PAYLOAD);
        }
        if self.length == 0 {
            self.state = State::Start;
            return self.finished_control_frame();
        }
        self.state = State::Payload;
        None
    }

    /// A ping or close frame has been read completely.
    fn finished_control_frame(&self) -> Option<Output> {
        match self.opcode {
            8 => Some(Output::Close),
            9 => Some(Output::Ping),
            _ => None,
        }
    }

    fn fail(&mut self) -> Option<Output> {
        self.state = State::Failed;
        Some(Output::Error)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepts_gzip_reads_the_accept_encoding_header() {
        let head = |line: &str| {
            format!("GET / HTTP/1.1\r\nHost: 192.168.4.1\r\n{line}\r\nAccept: */*\r\n\r\n")
        };
        assert!(accepts_gzip(
            head("Accept-Encoding: gzip, deflate, br").as_bytes()
        ));
        assert!(accepts_gzip(head("accept-encoding:gzip").as_bytes()));
        assert!(accepts_gzip(
            head("ACCEPT-ENCODING: br;q=1.0, GZIP ;q=0.5").as_bytes()
        ));
        assert!(!accepts_gzip(head("Accept-Encoding: identity").as_bytes()));
        assert!(!accepts_gzip(
            head("Accept-Encoding: deflate, br").as_bytes()
        ));
        assert!(!accepts_gzip(
            head("Accept-Encoding: gzip;q=0, identity").as_bytes()
        ));
        assert!(!accepts_gzip(
            head("Accept-Encoding: gzip; q=0.0").as_bytes()
        ));
        assert!(!accepts_gzip(head("X-Accept-Encoding: gzip").as_bytes()));
        assert!(!accepts_gzip(head("Accept: gzip").as_bytes()));
        assert!(!accepts_gzip(b"GET / HTTP/1.1\r\n\r\n"));
        assert!(!accepts_gzip(&[0xff, 0xfe]));
        // The request line is not a header.
        assert!(!accepts_gzip(b"GET /accept-encoding:gzip HTTP/1.1\r\n\r\n"));
    }

    extern crate std;
    use super::*;
    use std::format;
    use std::vec::Vec;

    #[test]
    fn accept_key_matches_the_rfc_example() {
        // RFC 6455, section 1.3.
        assert_eq!(
            &accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
            b"s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn base64_pads_the_twenty_byte_digest_with_one_equals_sign() {
        assert_eq!(&base64(&[0; 20]), b"AAAAAAAAAAAAAAAAAAAAAAAAAAA=");
        assert_eq!(&base64(&[0xff; 20]), b"//////////////////////////8=");
    }

    const UPGRADE: &[u8] = b"GET /ws HTTP/1.1\r\nHost: 192.168.4.1\r\nUpgrade: websocket\r\n\
        Connection: keep-alive, Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
        Sec-WebSocket-Version: 13\r\n\r\n";

    #[test]
    fn a_browsers_upgrade_request_is_recognised() {
        assert_eq!(
            parse_request(UPGRADE),
            Request::WebSocket {
                key: "dGhlIHNhbXBsZSBub25jZQ=="
            }
        );
    }

    #[test]
    fn header_names_and_values_are_case_insensitive() {
        let request = b"GET /ws HTTP/1.1\r\nupgrade: WebSocket\r\nCONNECTION: upgrade\r\n\
            sec-websocket-key: abc\r\n\r\n";
        assert_eq!(parse_request(request), Request::WebSocket { key: "abc" });
    }

    #[test]
    fn plain_gets_give_their_path_without_the_query() {
        assert_eq!(
            parse_request(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n"),
            Request::Get { path: "/" }
        );
        assert_eq!(
            parse_request(b"GET /index.html?x=1#top HTTP/1.1\r\n\r\n"),
            Request::Get {
                path: "/index.html"
            }
        );
    }

    #[test]
    fn an_upgrade_needs_every_part_and_the_right_path() {
        // No key, wrong path, missing Upgrade / Connection headers: all just ordinary GETs.
        let no_key = b"GET /ws HTTP/1.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        assert_eq!(parse_request(no_key), Request::Get { path: "/ws" });
        let wrong_path = b"GET /other HTTP/1.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
            Sec-WebSocket-Key: k\r\n\r\n";
        assert_eq!(parse_request(wrong_path), Request::Get { path: "/other" });
        let no_upgrade = b"GET /ws HTTP/1.1\r\nConnection: Upgrade\r\nSec-WebSocket-Key: k\r\n\r\n";
        assert_eq!(parse_request(no_upgrade), Request::Get { path: "/ws" });
    }

    #[test]
    fn other_methods_and_junk_are_other() {
        assert_eq!(parse_request(b"POST / HTTP/1.1\r\n\r\n"), Request::Other);
        assert_eq!(parse_request(b"GET /\r\n\r\n"), Request::Other);
        assert_eq!(parse_request(b"GET / SPDY/3\r\n\r\n"), Request::Other);
        assert_eq!(parse_request(&[0xff, 0xfe, 0x00]), Request::Other);
        assert_eq!(parse_request(b""), Request::Other);
    }

    #[test]
    fn the_head_ends_at_the_blank_line() {
        assert_eq!(head_length(b"GET / HTTP/1.1\r\n\r\nrest"), Some(18));
        assert_eq!(head_length(b"GET / HTTP/1.1\r\nHost: x\r\n"), None);
        assert_eq!(head_length(b""), None);
    }

    #[test]
    fn the_handshake_response_is_a_complete_101() {
        let mut out = [0u8; 160];
        let length = handshake_response("dGhlIHNhbXBsZSBub25jZQ==", &mut out);
        assert_eq!(
            &out[..length],
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
              Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n"
        );
    }

    fn decode(bytes: &[u8]) -> Vec<Output> {
        let mut decoder = Decoder::new();
        bytes
            .iter()
            .filter_map(|&byte| decoder.push(byte))
            .collect()
    }

    fn data(bytes: &[u8]) -> Vec<Output> {
        bytes.iter().map(|&byte| Output::Data(byte)).collect()
    }

    #[test]
    fn the_rfc_masked_hello_decodes() {
        // RFC 6455, section 5.7: a single-frame masked text message.
        let frame = [
            0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58,
        ];
        assert_eq!(decode(&frame), data(b"Hello"));
    }

    #[test]
    fn a_frame_split_at_any_point_decodes_the_same() {
        let frame = [0x82, 0x83, 1, 2, 3, 4, 10 ^ 1, 20 ^ 2, 30 ^ 3];
        for cut in 0..=frame.len() {
            let mut decoder = Decoder::new();
            let mut out = Vec::new();
            for &byte in &frame[..cut] {
                out.extend(decoder.push(byte));
            }
            for &byte in &frame[cut..] {
                out.extend(decoder.push(byte));
            }
            assert_eq!(out, data(&[10, 20, 30]), "cut at {cut}");
        }
    }

    #[test]
    fn frames_follow_each_other_and_unmasked_frames_are_accepted() {
        let mut bytes = Vec::new();
        bytes.extend([0x82, 0x02, 6, 0x86]); // unmasked binary, two bytes
        bytes.extend([0x82, 0x81, 9, 9, 9, 9, 5 ^ 9]); // masked, one byte
        assert_eq!(decode(&bytes), data(&[6, 0x86, 5]));
    }

    #[test]
    fn the_mask_repeats_every_four_bytes() {
        let mask = [0xa1, 0xb2, 0xc3, 0xd4];
        let payload = [1u8, 2, 3, 4, 5, 6, 7];
        let mut frame = std::vec![0x82, 0x80 | payload.len() as u8];
        frame.extend(mask);
        frame.extend(payload.iter().enumerate().map(|(i, &b)| b ^ mask[i % 4]));
        assert_eq!(decode(&frame), data(&payload));
    }

    #[test]
    fn a_sixteen_bit_length_is_read() {
        let payload = [7u8; 300];
        let mut frame = std::vec![0x82, 126, (300 >> 8) as u8, (300 & 255) as u8];
        frame.extend(payload);
        assert_eq!(decode(&frame), data(&payload));
    }

    #[test]
    fn control_frames_are_reported() {
        assert_eq!(decode(&[0x89, 0x00]), [Output::Ping]);
        assert_eq!(decode(&[0x88, 0x80, 1, 2, 3, 4]), [Output::Close]);
        // A close with a two-byte status code, masked.
        assert_eq!(
            decode(&[0x88, 0x82, 0, 0, 0, 0, 0x03, 0xe8]),
            [Output::Close]
        );
        // A pong is ignored, and so is its payload.
        assert_eq!(decode(&[0x8a, 0x02, 1, 2]), []);
        // Data after a ping still arrives.
        assert_eq!(
            decode(&[0x89, 0x00, 0x82, 0x01, 42]),
            [Output::Ping, Output::Data(42)]
        );
    }

    #[test]
    fn a_sixty_four_bit_length_is_an_error() {
        assert_eq!(
            decode(&[0x82, 127, 0, 0, 0, 0, 0, 1, 0, 0]),
            [Output::Error]
        );
    }

    #[test]
    fn a_ping_is_answered_with_its_own_payload() {
        let mask = [9, 8, 7, 6];
        let payload = *b"abcde";
        let mut frame = std::vec![0x89, 0x80 | payload.len() as u8];
        frame.extend(mask);
        frame.extend(payload.iter().enumerate().map(|(i, &b)| b ^ mask[i % 4]));
        let mut decoder = Decoder::new();
        let outputs: Vec<_> = frame
            .iter()
            .filter_map(|&byte| decoder.push(byte))
            .collect();
        assert_eq!(outputs, [Output::Ping]);
        assert_eq!(decoder.ping_payload(), b"abcde");
        let mut out = [0u8; 2 + MAX_CONTROL_PAYLOAD];
        let length = pong_frame(decoder.ping_payload(), &mut out);
        assert_eq!(&out[..length], &[0x8a, 5, b'a', b'b', b'c', b'd', b'e']);
    }

    #[test]
    fn an_empty_ping_gets_an_empty_pong() {
        let mut decoder = Decoder::new();
        assert_eq!(decoder.push(0x89), None);
        assert_eq!(decoder.push(0x00), Some(Output::Ping));
        assert_eq!(decoder.ping_payload(), b"");
        let mut out = [0u8; 2 + MAX_CONTROL_PAYLOAD];
        assert_eq!(pong_frame(&[], &mut out), 2);
        assert_eq!(&out[..2], &[0x8a, 0]);
    }

    #[test]
    fn a_ping_with_a_payload_is_reported_once_it_is_complete() {
        assert_eq!(
            decode(&[0x89, 0x02, 1, 2, 0x82, 0x01, 7]),
            [Output::Ping, Output::Data(7)]
        );
    }

    #[test]
    fn a_text_frame_is_final_unmasked_and_carries_its_payload() {
        let mut out = [0u8; 2 + MAX_TEXT_PAYLOAD];
        let length = text_frame(b"fps 28.4", &mut out);
        assert_eq!(&out[..length], b"\x81\x08fps 28.4");
        let length = text_frame(b"", &mut out);
        assert_eq!(&out[..length], [0x81, 0x00]);
    }

    #[test]
    fn a_long_text_payload_is_cut_to_what_the_length_byte_holds() {
        let mut out = [0u8; 2 + MAX_TEXT_PAYLOAD];
        let length = text_frame(&[b'a'; 300], &mut out);
        assert_eq!(length, 2 + MAX_TEXT_PAYLOAD);
        assert_eq!(out[1], MAX_TEXT_PAYLOAD as u8);
    }

    #[test]
    fn what_a_client_decodes_of_our_text_frame_is_the_payload() {
        // The client-side decoder reads unmasked frames too (tests only; a real client is a browser).
        let mut out = [0u8; 2 + MAX_TEXT_PAYLOAD];
        let length = text_frame(b"fps 28.4", &mut out);
        assert_eq!(decode(&out[..length]), data(b"fps 28.4"));
    }
}
