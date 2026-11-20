use defmt::warn;
use esp_radio::wifi::{Config, Ssid, WifiController, ap::AccessPointConfig};

use crate::config::WIFI_CONFIG_CH;

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

    loop {}
}
