//! Wi-Fi station plus a TCP command server. Runs on core 0 under the esp-rtos embassy executor;
//! the game on core 1 picks the decoded events up with [`next_key_event`].
//!
//! Every byte the controller sends is one `core_s3_protocol` command (see that crate).

use core_s3_protocol::{HeldKeys, KeyEvent, DEFAULT_PORT};
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, Config as NetConfig, Runner, Stack, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_println::println;
use esp_radio::wifi::{sta::StationConfig, Config, ControllerConfig, Interface, WifiController};
use static_cell::StaticCell;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

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

/// Starts Wi-Fi and the command server. Returns the network stack (so the caller can wait for an
/// address), or `None` when no credentials were compiled in.
pub fn start(spawner: Spawner, wifi: WIFI<'static>) -> Option<Stack<'static>> {
    if WIFI_SSID.is_empty() {
        println!("wifi: no credentials (copy wifi.env.example to wifi.env); network input is off");
        return None;
    }

    let mut controller =
        WifiController::new(wifi, ControllerConfig::default()).expect("wifi controller");
    controller
        .set_config(&Config::Station(
            StationConfig::default().with_ssid(WIFI_SSID).with_password(WIFI_PASSWORD.into()),
        ))
        .expect("wifi config");

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let rng = Rng::new();
    let seed = u64::from(rng.random()) << 32 | u64::from(rng.random());
    let (stack, runner) = embassy_net::new(
        Interface::station(),
        NetConfig::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(wifi_task(controller).expect("spawn wifi_task"));
    spawner.spawn(net_task(runner).expect("spawn net_task"));
    spawner.spawn(command_server(stack).expect("spawn command_server"));
    Some(stack)
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
