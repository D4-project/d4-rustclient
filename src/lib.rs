use std::convert::AsMut;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, NewMac, Mac};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
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

#[pyclass]
#[derive(Copy, Clone, Debug)]
pub struct D4Header {
    #[pyo3(get)]
    protocol_version: u8,
    #[pyo3(get)]
    packet_type: u8,
    #[pyo3(get)]
    uuid: [u8; 16],
    #[pyo3(get)]
    timestamp: u64,
    #[pyo3(get)]
    hmac: [u8; 32],
    #[pyo3(get)]
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

impl From<D4Header> for [u8; 62] {
    fn from(header: D4Header) -> Self {
        let mut to_return = [0; 62];
        to_return[0] = header.protocol_version;
        to_return[1] = header.packet_type;
        to_return[2..18].clone_from_slice(&header.uuid);
        to_return[18..26].clone_from_slice(&header.timestamp.to_le_bytes());
        to_return[26..58].clone_from_slice(&header.hmac);
        to_return[58..62].clone_from_slice(&header.size.to_le_bytes());
        to_return
    }
}

impl From<&[u8]> for D4Header {
    fn from(data: &[u8]) -> Self {
        D4Header {
            protocol_version: data[0],
            packet_type: data[1],
            uuid: clone_into_array(&data[2..18]),
            timestamp: u64::from_le_bytes(clone_into_array(&data[18..26])),
            hmac: clone_into_array(&data[26..58]),
            size: u32::from_le_bytes(clone_into_array(&data[58..62]))
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct D4Message {
    #[pyo3(get)]
    header: D4Header,
    #[pyo3(get)]
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
        let mut to_return: Vec<u8> = Into::<[u8; 62]>::into(message.header).to_vec();
        to_return.extend(&message.body.to_owned());
        to_return
    }
}

impl From<Vec<u8>> for D4Message {
    fn from(data: Vec<u8>) -> Self {
        D4Message::from(data.as_slice())
    }
}

impl From<&[u8]> for D4Message {
    fn from(data: &[u8]) -> Self {
        let header = D4Header::from(&data[0..62]);
        D4Message {
            header: header,
            body: data[62..62 + header.size as usize].to_vec()
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
}

#[pymethods]
impl D4Message {
    pub fn validate_hmac(&mut self, secret_key: &[u8]) -> bool {
        let mut mac = HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        let mut message = self.to_owned();
        let code_bytes: [u8; 32] = message.header.hmac;
        message.header.hmac = [0; 32];
        mac.update(Vec::from(message).as_slice());
        match mac.verify(&code_bytes) {
			Ok(_) => true,
			Err(_) => false
		}
    }

    #[new]
    pub fn new(protocol_version: u8, packet_type: u8, sensor_uuid: &[u8],
               key: &[u8], message: Vec<u8>) -> Self {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let header = D4Header {
            protocol_version: protocol_version,
            packet_type: packet_type,
            uuid: clone_into_array(&sensor_uuid[..16]),
            timestamp: time.as_secs(),
            hmac: [0; 32],
            size: message.len() as u32
        };
        let mut d4_message = D4Message {
            header: header,
            body: message
        };
        d4_message.compute_hmac(&key);
        d4_message
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        Vec::from(self.to_owned())
    }

}

#[pyfunction]
fn from_bytes(message: &[u8]) -> D4Message{
    D4Message::from(message)
}

#[pymodule]
fn d4message(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<D4Message>()?;
    m.add_function(wrap_pyfunction!(from_bytes, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn validate_error() {
        let protocol_version = 1;
        let packet_type = 1;
        let sensor_uuid = Uuid::new_v4();
        let key = String::from("My Hmac key");
        let message = String::from("blah");

        let mut message = D4Message::new(protocol_version, packet_type,
                                     sensor_uuid.as_bytes(),
                                     key.as_bytes(), Vec::from(message));
        message.body.push(1);
        assert_eq!(message.validate_hmac(key.as_bytes()), false);
    }

    #[test]
    fn validate() {
        let protocol_version = 1;
        let packet_type = 1;
        let sensor_uuid = Uuid::new_v4();
        let key = String::from("My Hmac key");
        let message = String::from("blah");

        let mut message = D4Message::new(protocol_version, packet_type,
                                     sensor_uuid.as_bytes(),
                                     key.as_bytes(), Vec::from(message));
        assert!(message.validate_hmac(key.as_bytes()));
        let encoded: Vec<u8> = Vec::from(message.to_owned());
        let mut decoded: D4Message = D4Message::from(encoded);
        assert!(decoded.validate_hmac(key.as_bytes()));
    }

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
