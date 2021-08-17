use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use clap::{Arg, App};
use uuid::Uuid;

use d4message::D4Message;

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

    let message = D4Message::new(protocol_version, packet_type,
                                 sensor_uuid.as_bytes(),
                                 key.as_bytes(), stdin_message);
    let encoded: Vec<u8> = Vec::from(message);


    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(encoded.as_slice())?;

    Ok(())
}
