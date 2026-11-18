use defmt::info;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_radio::wifi::{Interface, WifiController, ap::EventInfo};

/// Wi-Fi connection events reported by the access point.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum WifiEvent {
    Connected,
    Disconnected,
}

/// Channel used to report Wi-Fi [`WifiEvent`]s to other subsystems.
pub static WIFI_EVENT_CH: Channel<CriticalSectionRawMutex, WifiEvent, 4> = Channel::new();

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, Interface>) {
    runner.run().await;
}

#[embassy_executor::task]
pub async fn wifi_monitor_task(controller: WifiController<'static>) {
    loop {
        if let Ok(event) = controller
            .wait_for_access_point_connected_event_async()
            .await
        {
            match event {
                EventInfo::Connected(info) => {
                    let mac = info.mac;
                    info!(
                        "Device [{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}] connected",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                    );

                    WIFI_EVENT_CH.send(WifiEvent::Connected).await;
                }
                EventInfo::Disconnected(info) => {
                    let mac = info.mac;
                    info!(
                        "Device [{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}] disconnected",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                    );

                    WIFI_EVENT_CH.send(WifiEvent::Disconnected).await;
                }
            }
        }
    }
}
