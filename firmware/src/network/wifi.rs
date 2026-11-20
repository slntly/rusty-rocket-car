use defmt::{info, warn};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_radio::wifi::{
    Config, Ssid, WifiController,
    ap::{AccessPointConfig, EventInfo},
};

use crate::config::WIFI_CONFIG_CH;

/// Wi-Fi connection events reported by the access point.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum WiFiEvent {
    Connected,
    Disconnected,
}

/// Channel used to report Wi-Fi [`WifiEvent`]s to other subsystems.
pub static WIFI_EVENT_CH: Channel<CriticalSectionRawMutex, WiFiEvent, 4> = Channel::new();

#[embassy_executor::task]
pub async fn wifi_task(mut controller: WifiController<'static>) {
    let config = WIFI_CONFIG_CH.receive().await;

    let ssid: Ssid = config.ssid.try_into().expect("Invalid SSID format");

    let auth_config = if config.password.is_empty() {
        warn!("No access point password has been set; an open access point will be created");
        esp_radio::wifi::AuthenticationMethodConfig::Open
    } else {
        esp_radio::wifi::AuthenticationMethodConfig::Wpa2Personal(
            config.password.try_into().expect("Invalid password format"),
        )
    };

    let ap_config = AccessPointConfig::default()
        .with_ssid(ssid)
        .with_authentication(auth_config)
        .with_channel(config.channel)
        .with_max_connections(1);

    controller
        .set_config(&Config::AccessPoint(ap_config))
        .expect("Failed to set AP config");

    // SuperMini boards have a notoriously poor antenna layout, so cap the TX
    // power to keep Wi-Fi stable. Unit is 0.25 dBm; values above 34 are not
    // recommended for these boards.
    #[cfg(feature = "supermini")]
    controller
        .set_max_tx_power(34)
        .expect("Failed to set max TX power");

    loop {
        if let Ok(event) = controller
            .wait_for_access_point_connected_event_async()
            .await
        {
            match event {
                EventInfo::Connected(info) => wifi_event(info.mac, WiFiEvent::Connected).await,
                EventInfo::Disconnected(info) => {
                    wifi_event(info.mac, WiFiEvent::Disconnected).await
                }
            }
        }
    }
}

async fn wifi_event(mac: [u8; 6], event: WiFiEvent) {
    WIFI_EVENT_CH.send(event).await;

    info!(
        "Device [{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}] {:?}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5], event
    );
}
