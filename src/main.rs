use std::time::{SystemTime, UNIX_EPOCH, SystemTimeError};
use std::io::{self, Read, Write};
use std::error::Error;

use uuid::Uuid;
use serde::{Serialize, Serializer, Deserialize};
use serde::ser::SerializeStruct;

use sha2::Sha256;
use hmac::{Hmac, NewMac, Mac};

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize, Debug)]
struct D4Header {
    protocol_version: u8,
    packet_type: u8,
    uuid: [u8; 16],
    timestamp: u64,
    hmac: [u8; 32],
    size: u32,
}

#[derive(Debug)]
struct D4Message {
    header: D4Header,
    data: Vec<u8>,
}


impl Serialize for D4Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("D4Message", 2)?;
        state.serialize_field("header", &self.header)?;
		for v in &self.data {
	        state.serialize_field("data", v)?;
		}
        state.end()
    }
}

impl D4Message {
    fn compute_hmac(&mut self, secret_key: &[u8]) {
        let mut mac = HmacSha256::new_from_slice(secret_key).expect("HMAC can take key of any size");
        mac.update(&bincode::serialize(&self).unwrap());
        let result = mac.finalize();
        self.header.hmac = result.into_bytes().into();
    }
}

fn create_message(protocol_version: u8, packet_type: u8, sensor_uuid: &[u8; 16],
                  key: &[u8], message: Vec<u8>) -> Result<D4Message, SystemTimeError>{
    let time = SystemTime::now().duration_since(UNIX_EPOCH)?;

    let header = D4Header{
        protocol_version: protocol_version,
        packet_type: packet_type,
        uuid: *sensor_uuid,
        timestamp: time.as_secs(),
        hmac: [0; 32],
        size: message.len() as u32
    };
    let mut d4_message = D4Message {
        header: header,
        data: message
    };
    d4_message.compute_hmac(key);
    Ok(d4_message)
}


fn main() -> Result<(), Box<dyn Error>> {
    let sensor_uuid = Uuid::new_v4();
    // let sensor_uuid = Uuid::parse_str("cd980d75-dcf1-47da-bb60-aabe94fea6b2")?;
    let key = b"private key to change";
    let protocol_version = 1;
    let packet_type = 1;

    let mut message = Vec::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    handle.read_to_end(&mut message)?;

    let message = create_message(protocol_version, packet_type, sensor_uuid.as_bytes(), key, message)?;
    let encoded = bincode::serialize(&message).unwrap();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(encoded.as_slice())?;

    Ok(())
}
