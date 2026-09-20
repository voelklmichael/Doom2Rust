//! A minimal DHCP server for the CoreS3's own Wi-Fi network, so that a laptop that joins it gets
//! an address without any setup. `no_std`, and independent of the network stack: the firmware
//! feeds it one received datagram at a time and broadcasts whatever it returns.
//!
//! It hands out `/24` addresses from the server's own subnet, starting right after the server's
//! address, one per client MAC. It offers no gateway and no DNS server on purpose: the network
//! only carries the controller connection, and a client should not route anything else over it.

#![no_std]

/// The DHCP client and server ports; replies go to the broadcast address on the client port.
pub const SERVER_PORT: u16 = 67;
pub const CLIENT_PORT: u16 = 68;

/// The most this server ever writes: a BOOTP reply padded to the classic 300 bytes.
pub const REPLY_LEN: usize = 300;

/// How many clients can hold an address at once.
pub const MAX_CLIENTS: usize = 4;

/// How long a lease lasts, in seconds. Long, because the network only exists while the board runs.
const LEASE_SECONDS: u32 = 24 * 60 * 60;

const HEADER_LEN: usize = 236;
const COOKIE: [u8; 4] = [99, 130, 83, 99];
const FIRST_OPTION: usize = HEADER_LEN + COOKIE.len();

const BOOT_REQUEST: u8 = 1;
const BOOT_REPLY: u8 = 2;

const OPT_SUBNET_MASK: u8 = 1;
const OPT_REQUESTED_ADDRESS: u8 = 50;
const OPT_LEASE_TIME: u8 = 51;
const OPT_MESSAGE_TYPE: u8 = 53;
const OPT_SERVER_ID: u8 = 54;
const OPT_END: u8 = 255;
const OPT_PAD: u8 = 0;

const DISCOVER: u8 = 1;
const OFFER: u8 = 2;
const REQUEST: u8 = 3;
const ACK: u8 = 5;
const NAK: u8 = 6;

type Mac = [u8; 6];

pub struct Server {
    address: [u8; 4],
    clients: [Option<Mac>; MAX_CLIENTS],
}

impl Server {
    /// A server on `address` (its own address on a `/24`); clients get the next addresses up.
    pub const fn new(address: [u8; 4]) -> Self {
        Self {
            address,
            clients: [None; MAX_CLIENTS],
        }
    }

    /// Handles one datagram received on [`SERVER_PORT`]. Returns how many bytes of `reply` to
    /// broadcast to [`CLIENT_PORT`], or `None` if the datagram needs no answer.
    pub fn handle(&mut self, request: &[u8], reply: &mut [u8; REPLY_LEN]) -> Option<usize> {
        if request.len() < FIRST_OPTION
            || request[0] != BOOT_REQUEST
            || request[1] != 1 // Ethernet
            || request[2] != 6 // hardware address length
            || request[HEADER_LEN..FIRST_OPTION] != COOKIE
        {
            return None;
        }
        let mac: Mac = request[28..34].try_into().ok()?;
        let options = Options::parse(&request[FIRST_OPTION..]);
        match options.message_type? {
            DISCOVER => {
                let address = self.address_for(mac)?;
                Some(self.reply(request, reply, OFFER, address))
            }
            REQUEST => {
                // The client chose another server's offer: stay out of it.
                if options.server_id.is_some_and(|id| id != self.address) {
                    return None;
                }
                let asked = options
                    .requested_address
                    .unwrap_or_else(|| request[12..16].try_into().unwrap_or_default());
                match self.address_for(mac) {
                    Some(address) if address == asked => {
                        Some(self.reply(request, reply, ACK, address))
                    }
                    _ => Some(self.reply(request, reply, NAK, [0; 4])),
                }
            }
            _ => None,
        }
    }

    /// The address this client has, giving it the next free one if it has none. `None` when the
    /// pool is full.
    fn address_for(&mut self, mac: Mac) -> Option<[u8; 4]> {
        let index = match self.clients.iter().position(|client| *client == Some(mac)) {
            Some(index) => index,
            None => {
                let free = self.clients.iter().position(Option::is_none)?;
                self.clients[free] = Some(mac);
                free
            }
        };
        let mut address = self.address;
        address[3] = address[3].wrapping_add(1 + index as u8);
        Some(address)
    }

    fn reply(
        &self,
        request: &[u8],
        reply: &mut [u8; REPLY_LEN],
        kind: u8,
        address: [u8; 4],
    ) -> usize {
        reply.fill(0);
        reply[0] = BOOT_REPLY;
        reply[1] = 1;
        reply[2] = 6;
        reply[4..8].copy_from_slice(&request[4..8]); // transaction id
        reply[10..12].copy_from_slice(&request[10..12]); // flags
        reply[16..20].copy_from_slice(&address); // "your" address
        reply[20..24].copy_from_slice(&self.address); // next server
        reply[28..44].copy_from_slice(&request[28..44]); // client hardware address
        reply[HEADER_LEN..FIRST_OPTION].copy_from_slice(&COOKIE);

        let mut at = FIRST_OPTION;
        let mut put = |code: u8, value: &[u8]| {
            reply[at] = code;
            reply[at + 1] = value.len() as u8;
            reply[at + 2..at + 2 + value.len()].copy_from_slice(value);
            at += 2 + value.len();
        };
        put(OPT_MESSAGE_TYPE, &[kind]);
        put(OPT_SERVER_ID, &self.address);
        if kind != NAK {
            put(OPT_LEASE_TIME, &LEASE_SECONDS.to_be_bytes());
            put(OPT_SUBNET_MASK, &[255, 255, 255, 0]);
        }
        reply[at] = OPT_END;
        REPLY_LEN
    }
}

/// The options of a request that the server looks at.
struct Options {
    message_type: Option<u8>,
    server_id: Option<[u8; 4]>,
    requested_address: Option<[u8; 4]>,
}

impl Options {
    fn parse(mut bytes: &[u8]) -> Self {
        let mut options = Self {
            message_type: None,
            server_id: None,
            requested_address: None,
        };
        while let [code, rest @ ..] = bytes {
            match *code {
                OPT_END => break,
                OPT_PAD => bytes = rest,
                code => {
                    let [len, rest @ ..] = rest else { break };
                    let len = usize::from(*len);
                    if rest.len() < len {
                        break;
                    }
                    let (value, rest) = rest.split_at(len);
                    match (code, value) {
                        (OPT_MESSAGE_TYPE, [kind]) => options.message_type = Some(*kind),
                        (OPT_SERVER_ID, v) => options.server_id = v.try_into().ok(),
                        (OPT_REQUESTED_ADDRESS, v) => options.requested_address = v.try_into().ok(),
                        _ => {}
                    }
                    bytes = rest;
                }
            }
        }
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVER: [u8; 4] = [192, 168, 4, 1];
    const LAPTOP: Mac = [0xaa, 0xbb, 0xcc, 0x00, 0x00, 0x01];

    /// A client request as a laptop sends it: `kind`, plus the given extra options.
    fn request(mac: Mac, kind: u8, extra: &[(u8, &[u8])]) -> ([u8; 400], usize) {
        let mut packet = [0u8; 400];
        packet[0] = BOOT_REQUEST;
        packet[1] = 1;
        packet[2] = 6;
        packet[4..8].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
        packet[10] = 0x80; // broadcast flag
        packet[28..34].copy_from_slice(&mac);
        packet[HEADER_LEN..FIRST_OPTION].copy_from_slice(&COOKIE);
        let mut at = FIRST_OPTION;
        packet[at..at + 3].copy_from_slice(&[OPT_MESSAGE_TYPE, 1, kind]);
        at += 3;
        for (code, value) in extra {
            packet[at] = *code;
            packet[at + 1] = value.len() as u8;
            packet[at + 2..at + 2 + value.len()].copy_from_slice(value);
            at += 2 + value.len();
        }
        packet[at] = OPT_END;
        (packet, at + 1)
    }

    fn ask(
        server: &mut Server,
        mac: Mac,
        kind: u8,
        extra: &[(u8, &[u8])],
    ) -> Option<[u8; REPLY_LEN]> {
        let (packet, len) = request(mac, kind, extra);
        let mut reply = [0u8; REPLY_LEN];
        server.handle(&packet[..len], &mut reply).map(|n| {
            assert_eq!(n, REPLY_LEN);
            reply
        })
    }

    fn message_type(reply: &[u8; REPLY_LEN]) -> u8 {
        Options::parse(&reply[FIRST_OPTION..]).message_type.unwrap()
    }

    fn your_address(reply: &[u8; REPLY_LEN]) -> [u8; 4] {
        reply[16..20].try_into().unwrap()
    }

    #[test]
    fn discover_gets_an_offer_for_the_next_address() {
        let mut server = Server::new(SERVER);
        let reply = ask(&mut server, LAPTOP, DISCOVER, &[]).unwrap();
        assert_eq!(message_type(&reply), OFFER);
        assert_eq!(your_address(&reply), [192, 168, 4, 2]);
        assert_eq!(reply[0], BOOT_REPLY);
        assert_eq!(
            &reply[4..8],
            &[0x12, 0x34, 0x56, 0x78],
            "transaction id is echoed"
        );
        assert_eq!(reply[10], 0x80, "flags are echoed");
        assert_eq!(&reply[28..34], &LAPTOP, "client hardware address is echoed");
        assert_eq!(&reply[20..24], &SERVER);
        let options = Options::parse(&reply[FIRST_OPTION..]);
        assert_eq!(options.server_id, Some(SERVER));
    }

    #[test]
    fn offer_carries_lease_and_mask_but_no_gateway_or_dns() {
        let mut server = Server::new(SERVER);
        let reply = ask(&mut server, LAPTOP, DISCOVER, &[]).unwrap();
        let options = &reply[FIRST_OPTION..];
        let has = |code: u8| {
            let mut bytes = options;
            while let [c, rest @ ..] = bytes {
                if *c == OPT_END {
                    return false;
                }
                if *c == code {
                    return true;
                }
                let len = usize::from(rest[0]);
                bytes = &rest[1 + len..];
            }
            false
        };
        assert!(has(OPT_LEASE_TIME));
        assert!(has(OPT_SUBNET_MASK));
        assert!(!has(3), "no router option");
        assert!(!has(6), "no DNS option");
    }

    #[test]
    fn request_for_the_offered_address_is_acknowledged() {
        let mut server = Server::new(SERVER);
        ask(&mut server, LAPTOP, DISCOVER, &[]).unwrap();
        let reply = ask(
            &mut server,
            LAPTOP,
            REQUEST,
            &[
                (OPT_REQUESTED_ADDRESS, &[192, 168, 4, 2]),
                (OPT_SERVER_ID, &SERVER),
            ],
        )
        .unwrap();
        assert_eq!(message_type(&reply), ACK);
        assert_eq!(your_address(&reply), [192, 168, 4, 2]);
    }

    #[test]
    fn a_renewing_client_that_only_sets_ciaddr_is_acknowledged() {
        let mut server = Server::new(SERVER);
        ask(&mut server, LAPTOP, DISCOVER, &[]).unwrap();
        let (mut packet, len) = request(LAPTOP, REQUEST, &[]);
        packet[12..16].copy_from_slice(&[192, 168, 4, 2]);
        let mut reply = [0u8; REPLY_LEN];
        server.handle(&packet[..len], &mut reply).unwrap();
        assert_eq!(message_type(&reply), ACK);
    }

    #[test]
    fn a_client_keeps_its_address_and_others_get_the_next_ones() {
        let mut server = Server::new(SERVER);
        let other: Mac = [0xaa, 0xbb, 0xcc, 0x00, 0x00, 0x02];
        let first = your_address(&ask(&mut server, LAPTOP, DISCOVER, &[]).unwrap());
        let second = your_address(&ask(&mut server, other, DISCOVER, &[]).unwrap());
        let again = your_address(&ask(&mut server, LAPTOP, DISCOVER, &[]).unwrap());
        assert_eq!(first, [192, 168, 4, 2]);
        assert_eq!(second, [192, 168, 4, 3]);
        assert_eq!(again, first);
    }

    #[test]
    fn request_for_a_wrong_address_is_refused() {
        let mut server = Server::new(SERVER);
        let reply = ask(
            &mut server,
            LAPTOP,
            REQUEST,
            &[(OPT_REQUESTED_ADDRESS, &[10, 0, 0, 9])],
        )
        .unwrap();
        assert_eq!(message_type(&reply), NAK);
        assert_eq!(your_address(&reply), [0; 4]);
    }

    #[test]
    fn a_request_for_another_server_is_ignored() {
        let mut server = Server::new(SERVER);
        let reply = ask(
            &mut server,
            LAPTOP,
            REQUEST,
            &[
                (OPT_REQUESTED_ADDRESS, &[192, 168, 4, 2]),
                (OPT_SERVER_ID, &[10, 0, 0, 1]),
            ],
        );
        assert!(reply.is_none());
    }

    #[test]
    fn a_full_pool_stops_answering_new_clients() {
        let mut server = Server::new(SERVER);
        for n in 0..MAX_CLIENTS as u8 {
            assert!(ask(&mut server, [0, 0, 0, 0, 0, n], DISCOVER, &[]).is_some());
        }
        assert!(ask(&mut server, [0, 0, 0, 0, 0, 99], DISCOVER, &[]).is_none());
        // Someone who already has an address still gets it.
        assert!(ask(&mut server, [0, 0, 0, 0, 0, 0], DISCOVER, &[]).is_some());
    }

    #[test]
    fn other_message_types_and_junk_get_no_answer() {
        let mut server = Server::new(SERVER);
        assert!(ask(&mut server, LAPTOP, 7, &[]).is_none(), "release");
        assert!(ask(&mut server, LAPTOP, 8, &[]).is_none(), "inform");

        let mut reply = [0u8; REPLY_LEN];
        assert!(server.handle(&[], &mut reply).is_none());
        assert!(server.handle(&[0u8; 100], &mut reply).is_none());
        let (mut packet, len) = request(LAPTOP, DISCOVER, &[]);
        packet[0] = BOOT_REPLY;
        assert!(
            server.handle(&packet[..len], &mut reply).is_none(),
            "a reply is not a request"
        );
        let (mut packet, len) = request(LAPTOP, DISCOVER, &[]);
        packet[HEADER_LEN] = 0;
        assert!(
            server.handle(&packet[..len], &mut reply).is_none(),
            "bad magic cookie"
        );
    }

    #[test]
    fn truncated_options_do_not_panic() {
        let mut server = Server::new(SERVER);
        let (packet, len) = request(LAPTOP, DISCOVER, &[(OPT_SERVER_ID, &[1, 2, 3, 4])]);
        let mut reply = [0u8; REPLY_LEN];
        for cut in FIRST_OPTION..len {
            let _ = server.handle(&packet[..cut], &mut reply);
        }
    }
}
