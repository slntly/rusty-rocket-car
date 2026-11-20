use core::net::Ipv4Addr;
use embassy_net::Stack;
use leasehund::{DhcpConfigBuilder, DhcpServer};

use crate::config::WIFI_CONFIG_WATCH;

#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) {
    let wifi_config = WIFI_CONFIG_WATCH
        .receiver()
        .expect("Failed to subscribe to the Wi-Fi config watch")
        .get()
        .await;

    stack.wait_config_up().await;

    let ip = wifi_config.ip;

    // Calculate the client address (.1 -> .2)
    let client_ip = Ipv4Addr::from_bits(ip.to_bits() + 1);

    let config = DhcpConfigBuilder::new()
        .server_ip(ip)
        .subnet_mask(Ipv4Addr::new(255, 255, 255, 0))
        .ip_pool(client_ip, client_ip)
        .build();

    let mut server = DhcpServer::<2, 0>::with_config(config);

    server.run(stack).await.expect("Failed to run DHCP server");
}
