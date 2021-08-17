use std::convert::AsMut;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, NewMac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

#[derive(Copy, Clone, Debug)]
pub struct D4Header {
    protocol_version: u8,
    packet_type: u8,
    uuid: [u8; 16],
    timestamp: u64,
    hmac: [u8; 32],
    size: u32,
}

impl PartialEq for D4Header {
    fn eq(&self, other: &Self) -> bool {
        (self.protocol_version == other.protocol_version)
        & (self.packet_type == other.packet_type)
        & (self.uuid == other.uuid)
        & (self.timestamp == other.timestamp)
        & (self.hmac == other.hmac)
        & (self.size == other.size)
    }
}

impl From<D4Header> for Vec<u8> {
    fn from(header: D4Header) -> Self {
        let mut to_return: Vec<u8> = Vec::with_capacity(62);
        to_return.push(header.protocol_version);
        to_return.push(header.packet_type);
        to_return.extend(&header.uuid);
        to_return.extend(&bincode::serialize(&header.timestamp).unwrap());
        to_return.extend(&header.hmac);
        to_return.extend(&bincode::serialize(&header.size).unwrap());
        to_return
    }
}

impl From<Vec<u8>> for D4Header {
    fn from(data: Vec<u8>) -> Self {
        D4Header {
            protocol_version: data[0],
            packet_type: data[1],
            uuid: clone_into_array(&data[2..18]),
            timestamp: bincode::deserialize(&data[18..26]).unwrap(),
            hmac: clone_into_array(&data[26..58]),
            size: bincode::deserialize(&data[58..62]).unwrap()
        }
    }
}


#[derive(Debug, Clone)]
pub struct D4Message {
    header: D4Header,
    body: Vec<u8>,
}

impl PartialEq for D4Message {
    fn eq(&self, other: &Self) -> bool {
        (self.header == other.header)
        & (self.body == other.body)
    }
}

impl From<D4Message> for Vec<u8> {
    fn from(message: D4Message) -> Self {
        let mut to_return: Vec<u8> = Vec::from(message.header);
        to_return.extend(&message.body.to_owned());
        to_return
    }
}

impl From<Vec<u8>> for D4Message {
    fn from(data: Vec<u8>) -> Self {
        D4Message {
            header: D4Header::from(data[0..62].to_vec()),
            body: data[62..].to_vec()
        }
    }
}

impl D4Message {
    fn compute_hmac(&mut self, secret_key: &[u8]) {
        let mut mac = HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        mac.update(Vec::from(self.to_owned()).as_slice());
        let result = mac.finalize();
        self.header.hmac = result.into_bytes().into();
    }

    pub fn new(protocol_version: u8, packet_type: u8, sensor_uuid: &[u8; 16],
               key: &[u8], message: Vec<u8>) -> Self {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let header = D4Header {
            protocol_version: protocol_version,
            packet_type: packet_type,
            uuid: *sensor_uuid,
            timestamp: time.as_secs(),
            hmac: [0; 32],
            size: message.len() as u32
        };
        let mut d4_message = D4Message {
            header: header,
            body: message
        };
        d4_message.compute_hmac(key);
        d4_message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn encode_decode() {
        let protocol_version = 1;
        let packet_type = 1;
        let sensor_uuid = Uuid::new_v4();
        let key = String::from("My Hmac key");
        let message = String::from("blah");

        let message = D4Message::new(protocol_version, packet_type,
                       sensor_uuid.as_bytes(),
                       key.as_bytes(), Vec::from(message));

        let encoded: Vec<u8> = Vec::from(message.to_owned());
        let decoded: D4Message = D4Message::from(encoded);
        assert_eq!(message, decoded);
    }
}
