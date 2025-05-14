use std::collections::BTreeMap;
use std::fmt;
use std::net::IpAddr;
use std::os::unix::io::AsRawFd;
use std::str;
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{wait as phy_wait, Device, Medium, TunTapInterface};
use smoltcp::socket::{tcp::Socket, tcp::SocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use url::Url;

#[derive(Debug)]
enum HttpState {
    Connect,
    Request,
    Response,
}

#[derive(Debug)]
pub enum UpstreamError {
    Network(smoltcp::socket::tcp::ConnectError),
    InvalidUrl,
    Content(std::str::Utf8Error),
}

impl fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<smoltcp::socket::tcp::ConnectError> for UpstreamError {
    fn from(error: smoltcp::socket::tcp::ConnectError) -> Self {
        UpstreamError::Network(error)
    }
}

impl From<std::str::Utf8Error> for UpstreamError {
    fn from(error: std::str::Utf8Error) -> Self {
        UpstreamError::Content(error)
    }
}

fn random_port() -> u16 {
    49152 + rand::random::<u16>() % 16384
}

pub fn get(
    mut device: TunTapInterface,
    mac: EthernetAddress,
    addr: IpAddr,
    url: Url,
) -> Result<(), UpstreamError> {
    let domain_name = url.host_str().ok_or(UpstreamError::InvalidUrl)?;
    let fd = device.as_raw_fd();

    // Configure interface
    let mut config = match device.capabilities().medium {
        Medium::Ethernet => Config::new(mac.into()),
        Medium::Ip => Config::new(HardwareAddress::Ip),
        Medium::Ieee802154 => todo!(),
    };
    config.random_seed = rand::random();

    let mut iface = Interface::new(config, &mut device, Instant::now());

    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::v4(192, 168, 42, 1), 24))
            .unwrap();
    });

    iface
        .routes_mut()
        .add_default_ipv4_route(Ipv4Address::new(192, 168, 42, 100))
        .unwrap();

    // Create TCP socket
    let tcp_rx_buffer = SocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = SocketBuffer::new(vec![0; 1024]);
    let tcp_socket = Socket::new(tcp_rx_buffer, tcp_tx_buffer);
    let mut sockets = SocketSet::new(vec![]);
    let tcp_handle = sockets.add(tcp_socket);

    let http_header = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        url.path(),
        domain_name
    );

    let mut state = HttpState::Connect;

    loop {
        let timestamp = Instant::now();
        iface.poll(timestamp, &mut device, &mut sockets)?;

        let socket = sockets.get_mut::<Socket>(tcp_handle);
        let cx = iface.context();

        state = match state {
            HttpState::Connect if !socket.is_active() => {
                socket.connect(cx, (addr, url.port().unwrap_or(80)), random_port())?;
                HttpState::Request
            }

            HttpState::Request if socket.may_send() => {
                socket.send_slice(http_header.as_ref())?;
                HttpState::Response
            }

            HttpState::Response if socket.can_recv() => {
                socket.recv(|data| {
                    println!("{}", str::from_utf8(data).unwrap_or("(invalid utf8)"));
                    (data.len(), ())
                })?;
                HttpState::Response
            }

            HttpState::Response if !socket.may_recv() => {
                break;
            }

            _ => state,
        };

        phy_wait(fd, iface.poll_delay(timestamp, &sockets))?;
    }

    Ok(())
}
