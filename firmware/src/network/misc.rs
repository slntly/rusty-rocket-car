use defmt::info;
use esp_radio::wifi::{Interface, WifiController, ap::EventInfo};

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
                    )
                }
                EventInfo::Disconnected(info) => {
                    let mac = info.mac;
                    info!(
                        "Device [{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}] disconnected",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                    )
                }
            }
        }
    }
}
