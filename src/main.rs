use std::convert::AsMut;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH, SystemTimeError};

use clap::{Arg, App};
use hmac::{Hmac, NewMac, Mac};
use sha2::Sha256;
use uuid::Uuid;

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
struct D4Header {
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
struct D4Message {
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
        body: message
    };
    d4_message.compute_hmac(key);
    Ok(d4_message)
}

fn read_config_file(path: &Path) -> String {
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", path.display(), why),
        Ok(_) => (),
    }
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let protocol_version = 1;
        let packet_type = 1;
        let sensor_uuid = Uuid::new_v4();
        let key = String::from("My Hmac key");
        let message = String::from("blah");

        match create_message(protocol_version, packet_type,
                                     sensor_uuid.as_bytes(),
                                     key.as_bytes(), Vec::from(message)){
            Err(why) => panic!("{:?}", why),
            Ok(message) => {
                let encoded: Vec<u8> = Vec::from(message.to_owned());
                let decoded: D4Message = D4Message::from(encoded);
                assert_eq!(message, decoded);
            }
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {

    let matches = App::new("d4 - d4 client")
        .version("0.1.0")
        .author("D4 team")
        .about("Read data from <stdin> and send it to <stdout>")
        .arg(Arg::with_name("config-directory")
                 .short("c")
                 .long("config-directory")
                 .takes_value(true)
                 .help("The configuration directory"))
        .get_matches();

    let config_dir = matches.value_of("config-directory").unwrap_or("conf.sample");
    let config_dir_path = Path::new(config_dir);

    let u = read_config_file(&config_dir_path.join("uuid"));
    let sensor_uuid = Uuid::parse_str(&u)?;

    let key = read_config_file(&config_dir_path.join("key"));

    let v = read_config_file(&config_dir_path.join("version"));
    let protocol_version = v.parse::<u8>().unwrap();

    let t = read_config_file(&config_dir_path.join("type"));
    let packet_type = t.parse::<u8>().unwrap();


    let mut stdin_message = Vec::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    handle.read_to_end(&mut stdin_message)?;

    let message = create_message(protocol_version, packet_type,
                                 sensor_uuid.as_bytes(),
                                 key.as_bytes(), stdin_message)?;
    let encoded: Vec<u8> = Vec::from(message);


    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(encoded.as_slice())?;

    Ok(())
}
