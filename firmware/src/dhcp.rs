use core::net::Ipv4Addr;
use defmt::info;
use embassy_net::Stack;
use leasehund::{DhcpConfigBuilder, DhcpServer};

use crate::AP_IP;

const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
const IP_POOL_START: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 100);
const IP_POOL_END: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 100);

const MAX_CLIENTS: usize = 2; // maximum number of concurrent clients
const MAX_DNS: usize = 1; // maximum number of DNS servers provided to clients

const LEASE_TIME: u32 = 10; // seconds

#[embassy_executor::task]
pub async fn dhcp_server_task(stack: Stack<'static>) {
    stack.wait_config_up().await;

    let config = DhcpConfigBuilder::new()
        .server_ip(AP_IP)
        .subnet_mask(SUBNET_MASK)
        .router(AP_IP)
        .add_dns_server(AP_IP)
        .ip_pool(IP_POOL_START, IP_POOL_END)
        .lease_time(LEASE_TIME)
        .build();

    let mut server = DhcpServer::<MAX_CLIENTS, MAX_DNS>::with_config(config);

    // Run the DHCP server
    server.run(stack).await.unwrap();
}
