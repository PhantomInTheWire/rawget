use std::net::{Ipv4Addr, UdpSocket};
use std::str::FromStr;
use hickory_proto::op::{Message, MessageType, OpCode, Query};
use hickory_proto::rr::{Name, RecordType};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable, BinEncoder};
use hickory_proto::rr::RData;
use rand;
use hickory_proto::ProtoError;
use std::fmt;

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnsError::ParseDomainName(e) => write!(f, "Invalid domain name: {}", e),
            DnsError::ParseDnsServerAddress(e) => write!(f, "Invalid DNS server address: {}", e),
            DnsError::Encoding(e) => write!(f, "Failed to encode DNS request: {}", e),
            DnsError::Decoding(e) => write!(f, "Failed to decode DNS response: {}", e),
            DnsError::Network(e) => write!(f, "Network error: {}", e),
            DnsError::Sending(e) => write!(f, "Failed to send DNS query: {}", e),
            DnsError::Receiving(e) => write!(f, "Failed to receive DNS response: {}", e),
            DnsError::NoSuchDomain => write!(f, "Domain does not exist"),
        }
    }
}


#[derive(Debug)]
pub enum DnsError {
    ParseDomainName(ProtoError),
    ParseDnsServerAddress(std::net::AddrParseError),
    Encoding(ProtoError),
    Decoding(ProtoError),
    Network(std::io::Error),
    Sending(std::io::Error),
    Receiving(std::io::Error),
    NoSuchDomain,
}


pub fn resolve(domain: &str, dns_server_addr: &Ipv4Addr) -> Result<Vec<Ipv4Addr>, DnsError> {

    // Construct DNS message
    let mut message = Message::new();
    message
        .set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    let name = Name::from_str(&domain).map_err(DnsError::ParseDomainName)?;
    let query = Query::query(name, RecordType::A);
    message.add_query(query);

    let mut buf = Vec::with_capacity(512);
    {
        let mut encoder = BinEncoder::new(&mut buf);
        message.emit(&mut encoder).map_err(DnsError::Encoding)?;
    }

    // Send UDP packet to DNS server
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(DnsError::Sending)?;
    socket
        .send_to(&buf, (dns_server_addr.to_string(), 53))
        .map_err(DnsError::Sending)?;

    let mut response_buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut response_buf).map_err(DnsError::Receiving)?;

    let response = Message::from_bytes(&response_buf[..len]).map_err(DnsError::Decoding)?;
    let mut ips = Vec::new();
    for answer in response.answers() {
        if let RData::A(ipv4) = answer.data() {
            ips.push(ipv4.0);
        }
    }
    Ok(ips)
}
