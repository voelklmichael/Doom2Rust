//! Wi-Fi plus a TCP command server. Runs on core 0 under the esp-rtos embassy executor; the game
//! on core 1 picks the decoded events up with [`next_key_event`].
//!
//! With credentials in `wifi.env` the board joins that network as a station and gets its address
//! by DHCP. Without them it makes its own open network ([`AP_SSID`]) at [`AP_ADDRESS`] and runs a
//! small DHCP server, so a laptop that joins it needs no password and no setup.
//!
//! Every byte the controller sends is one `core_s3_protocol` command (see that crate).

use core_s3_dhcp::{Server as DhcpServer, CLIENT_PORT, MAX_CLIENTS, REPLY_LEN, SERVER_PORT};
use core_s3_protocol::{HeldKeys, KeyEvent, DEFAULT_PORT};
use embassy_executor::Spawner;
use embassy_net::{
    tcp::TcpSocket,
    udp::{PacketMetadata, UdpSocket},
    Config as NetConfig, Ipv4Address, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_println::println;
use esp_radio::wifi::{
    ap::AccessPointConfig, sta::StationConfig, Config, ControllerConfig, Interface, WifiController,
};
use static_cell::StaticCell;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

/// The board's own network, used when there are no credentials: open, at this address.
const AP_SSID: &str = "CoreS3-DOOM";
const AP_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 4, 1);

/// Where the controller connects, and what it has to do to get there.
pub struct Network {
    pub stack: Stack<'static>,
    /// The network the controller must join first: the board's own one, if it made one.
    pub own_network: Option<&'static str>,
}

/// Events wait here between the network task (core 0) and the game (core 1).
static KEY_EVENTS: Channel<CriticalSectionRawMutex, KeyEvent, 32> = Channel::new();

/// The oldest event the controller has sent that the game has not seen yet.
pub fn next_key_event() -> Option<KeyEvent> {
    KEY_EVENTS.try_receive().ok()
}

fn queue(event: KeyEvent) {
    // A full queue means the game has stalled; dropping input beats blocking the network.
    let _ = KEY_EVENTS.try_send(event);
}

/// Starts Wi-Fi and the command server: the station if credentials were compiled in, the board's
/// own network if not. Returns the network stack, so the caller can wait for an address.
pub fn start(spawner: Spawner, wifi: WIFI<'static>) -> Network {
    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let rng = Rng::new();
    let seed = u64::from(rng.random()) << 32 | u64::from(rng.random());

    if WIFI_SSID.is_empty() {
        println!("wifi: no credentials in wifi.env; making the open network {AP_SSID}");
        let access_point = AccessPointConfig::default()
            .with_ssid(AP_SSID)
            .with_max_connections(MAX_CLIENTS as u16);
        let controller = WifiController::new(
            wifi,
            ControllerConfig::default().with_initial_config(Config::AccessPoint(access_point)),
        )
        .expect("wifi controller");
        let (stack, runner) = embassy_net::new(
            Interface::access_point(),
            NetConfig::ipv4_static(StaticConfigV4 {
                address: Ipv4Cidr::new(AP_ADDRESS, 24),
                gateway: None,
                dns_servers: Default::default(),
            }),
            RESOURCES.init(StackResources::new()),
            seed,
        );
        spawner.spawn(access_point_task(controller).expect("spawn access_point_task"));
        spawner.spawn(dhcp_server(stack).expect("spawn dhcp_server"));
        spawner.spawn(net_task(runner).expect("spawn net_task"));
        spawner.spawn(command_server(stack).expect("spawn command_server"));
        return Network { stack, own_network: Some(AP_SSID) };
    }

    let mut controller =
        WifiController::new(wifi, ControllerConfig::default()).expect("wifi controller");
    controller
        .set_config(&Config::Station(
            StationConfig::default().with_ssid(WIFI_SSID).with_password(WIFI_PASSWORD.into()),
        ))
        .expect("wifi config");
    let (stack, runner) = embassy_net::new(
        Interface::station(),
        NetConfig::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );
    spawner.spawn(wifi_task(controller).expect("spawn wifi_task"));
    spawner.spawn(net_task(runner).expect("spawn net_task"));
    spawner.spawn(command_server(stack).expect("spawn command_server"));
    Network { stack, own_network: None }
}

/// Keeps the access point up (it stops when the controller is dropped) and logs who joins.
#[embassy_executor::task]
async fn access_point_task(controller: WifiController<'static>) {
    loop {
        match controller.wait_for_access_point_connected_event_async().await {
            Ok(event) => println!("wifi: {event:?}"),
            Err(err) => {
                println!("wifi: access point event error: {err:?}");
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}

/// Hands out addresses to whoever joins the board's network.
#[embassy_executor::task]
async fn dhcp_server(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buffer = [0u8; 1024];
    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(SERVER_PORT).expect("bind the DHCP server port");

    let mut server = DhcpServer::new(AP_ADDRESS.octets());
    let mut request = [0u8; 576];
    let mut reply = [0u8; REPLY_LEN];
    loop {
        let Ok((length, _)) = socket.recv_from(&mut request).await else { continue };
        let Some(reply_length) = server.handle(&request[..length], &mut reply) else { continue };
        // The client has no address yet, so the answer goes to everyone.
        match socket.send_to(&reply[..reply_length], (Ipv4Address::BROADCAST, CLIENT_PORT)).await {
            Ok(()) => println!("dhcp: answered a client"),
            Err(err) => println!("dhcp: send failed: {err:?}"),
        }
    }
}

/// Keeps the station associated, reconnecting whenever it drops.
#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>) {
    loop {
        match controller.connect_async().await {
            Ok(_) => {
                println!("wifi: connected to {WIFI_SSID}");
                let _ = controller.wait_for_disconnect_async().await;
                println!("wifi: disconnected");
            }
            Err(err) => println!("wifi: connect failed: {err:?}"),
        }
        Timer::after(Duration::from_secs(3)).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await;
}

/// Accepts one controller at a time and turns its bytes into queued key events.
#[embassy_executor::task]
async fn command_server(stack: Stack<'static>) {
    let mut rx_buffer = [0u8; 64];
    let mut tx_buffer = [0u8; 64];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        // A controller that vanishes without closing (laptop lid, dead Wi-Fi) must not hold the
        // only connection slot forever.
        socket.set_keep_alive(Some(Duration::from_secs(5)));
        socket.set_timeout(Some(Duration::from_secs(20)));
        if let Err(err) = socket.accept(DEFAULT_PORT).await {
            println!("command server: accept failed: {err:?}");
            continue;
        }
        println!("controller connected: {:?}", socket.remote_endpoint());

        let mut held = HeldKeys::default();
        let mut bytes = [0u8; 32];
        loop {
            match socket.read(&mut bytes).await {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    for &byte in &bytes[..count] {
                        let Some(event) = KeyEvent::decode(byte) else { continue };
                        println!("key {event:?}");
                        held.update(event);
                        queue(event);
                    }
                }
            }
        }
        for release in held.take_releases() {
            queue(release);
        }
        println!("controller disconnected");
    }
}
