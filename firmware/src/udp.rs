use defmt::{debug, error, info, warn};
use embassy_net::Stack;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_time::Timer;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub struct Packet {
    pub throttle: i8,
    pub steering: i8,
}

impl Packet {
    pub const SIZE: usize = 2;

    #[inline]
    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        Self {
            throttle: bytes[0] as i8,
            steering: bytes[1] as i8,
        }
    }

    #[inline]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        [self.throttle as u8, self.steering as u8]
    }
}

#[embassy_executor::task]
pub async fn udp_task(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buffer = [0u8; 64];
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buffer = [0u8; 64];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    if let Err(e) = socket.bind(8888) {
        error!("Failed to bind UDP socket: {}", e);
        loop {}
    }
    info!("UDP socket bound to port 8888");

    loop {
        let mut rx_buf = [0u8; 64];

        match socket.recv_from(&mut rx_buf).await {
            Ok((len, src_meta)) => {
                if len == Packet::SIZE {
                    let mut buf = [0u8; Packet::SIZE];
                    buf.copy_from_slice(&rx_buf[..Packet::SIZE]);

                    let rx_pkt = Packet::from_bytes(buf);
                    debug!("Received from {}: {:?}", src_meta.endpoint, rx_pkt);
                } else {
                    warn!(
                        "Unexpected packet size from {}: {} (expected {})",
                        src_meta.endpoint,
                        len,
                        Packet::SIZE
                    );
                }
            }
            Err(e) => {
                warn!("RX error: {:?}", e);
                Timer::after_millis(10).await;
                continue;
            }
        }
    }
}
