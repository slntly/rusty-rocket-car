use defmt::{info, warn};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;
use embassy_time::{Duration, Instant, Ticker};

use crate::network::udp::PACKET_SIGNAL;
use crate::network::wifi::{WIFI_EVENT_CH, WiFiEvent};

/// Interval at which the failsafe re-evaluates its inputs.
const TICK_MS: u64 = 20;

/// Broadcast stop signal. `true` means the actuators must stay at safe values.
static STOP: Watch<CriticalSectionRawMutex, bool, 1> = Watch::new_with(true);

/// Whether the failsafe currently requires the actuators to stop.
#[inline]
pub fn is_stopped() -> bool {
    STOP.try_get().unwrap_or(true)
}

/// Task that decides whether the actuators are allowed to run.
///
/// The failsafe engages when the Wi-Fi link drops or when no control packet
/// has arrived for `packet_timeout`, and releases once packets are flowing
/// again. The drivers only see the stop signal and know nothing about Wi-Fi
/// or command timeouts.
#[embassy_executor::task]
pub async fn failsafe_task(packet_timeout: Duration) {
    let wifi_events = WIFI_EVENT_CH.receiver();
    let stop = STOP.sender();
    let mut ticker = Ticker::every(Duration::from_millis(TICK_MS));

    let mut link_up = false;
    let mut last_packet: Option<Instant> = None;
    let mut stopped = true;

    loop {
        while let Ok(event) = wifi_events.try_receive() {
            link_up = matches!(event, WiFiEvent::Connected);

            if !link_up {
                last_packet = None;
            }
        }

        if PACKET_SIGNAL.try_take().is_some() && link_up {
            last_packet = Some(Instant::now());
        }

        let next = !link_up || last_packet.is_none_or(|at| at.elapsed() >= packet_timeout);

        if next != stopped {
            stopped = next;

            if stopped {
                if link_up {
                    warn!(
                        "Failsafe engaged: no commands for {} ms",
                        packet_timeout.as_millis()
                    );
                } else {
                    warn!("Failsafe engaged: Wi-Fi disconnected");
                }
            } else {
                info!("Failsafe released: commands resumed");
            }

            stop.send(stopped);
        }

        ticker.next().await;
    }
}
