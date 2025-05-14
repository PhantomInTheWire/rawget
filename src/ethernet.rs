use rand::RngCore;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress {
    octets: [u8; 6],
}

impl MacAddress {
    pub fn generate() -> Self {
        let mut octets = [0u8; 6];
        rand::rng().fill_bytes(&mut octets);

        // Set locally administered bit (bit 1), clear multicast bit (bit 0).
        octets[0] = (octets[0] | 0b0000_0010) & 0b1111_1110;

        Self { octets }

    }

    pub fn is_local(&self) -> bool {
        self.octets[0] & 0b0000_0010 != 0
    }

    pub fn is_unicast(&self) -> bool {
        self.octets[0] & 0b0000_0001 == 0
    }

    pub fn octets(&self) -> [u8; 6] {
        self.octets
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, f_] = self.octets;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            a, b, c, d, e, f_
        )
    }
}
