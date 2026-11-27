use core::mem;
use defmt::{info, warn};
use embassy_net::Stack;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::watch::Watch;
use embassy_time::Timer;

pub static THROTTLE_WATCH: Watch<CriticalSectionRawMutex, i8, 1> = Watch::new();
pub static STEERING_WATCH: Watch<CriticalSectionRawMutex, i8, 1> = Watch::new();

/// Notifies the failsafe task that the control stream is still alive.
pub static PACKET_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

const UDP_PORT: u16 = 8888;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
struct Packet {
    throttle: i8,
    steering: i8,
}

enum PacketError {
    InvalidMagic(u16),
    InvalidSize(usize),
}

impl Packet {
    /// ASCII "RC", big-endian.
    const MAGIC: u16 = 0x5243;
    const SIZE: usize = mem::size_of_val(&Self::MAGIC) + mem::size_of::<Self>();

    #[inline]
    fn from_slice(bytes: &[u8]) -> Result<Self, PacketError> {
        let len = bytes.len();
        if len != Self::SIZE {
            return Err(PacketError::InvalidSize(len));
        }

        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);
        if magic != Self::MAGIC {
            return Err(PacketError::InvalidMagic(magic));
        }

        Ok(Self {
            throttle: (bytes[2] as i8).clamp(-100, 100),
            steering: (bytes[3] as i8).clamp(-100, 100),
        })
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

    socket.bind(UDP_PORT).expect("Failed to bind UDP socket");
    info!("UDP socket bound to port 8888");

    let throttle_sender = THROTTLE_WATCH.sender();
    let steering_sender = STEERING_WATCH.sender();

    let mut rx_buf = [0u8; 64];
    let mut last_pkt: Option<Packet> = None;

    loop {
        match socket.recv_from(&mut rx_buf).await {
            Ok((len, src_meta)) => {
                let rx_pkt = match Packet::from_slice(&rx_buf[..len]) {
                    Ok(pkt) => pkt,
                    Err(err) => {
                        warn!("Invalid packet from {}: {:?}", src_meta.endpoint, err);
                        continue;
                    }
                };

                // Log only changes to keep the monitor readable while the
                // controller repeats the same command at a high rate.
                if last_pkt != Some(rx_pkt) {
                    info!("Received from {}: {:?}", src_meta.endpoint, rx_pkt);
                    last_pkt = Some(rx_pkt);
                }

                throttle_sender.send(rx_pkt.throttle);
                steering_sender.send(rx_pkt.steering);
                PACKET_SIGNAL.signal(());
            }
            Err(e) => {
                warn!("RX error: {:?}", e);
                Timer::after_millis(10).await;
            }
        }
    }
}
