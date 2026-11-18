use esp_radio::wifi::Interface;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, Interface>) {
    runner.run().await;
}
