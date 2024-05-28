use core::panic;
use std::{
    fs::{self, File},
    io::copy,
    net::TcpStream,
    process::Command,
};

use flate2::Compression;

// use std::panic;

fn main() {
    // This panic hook allows us to disable the wireless watchdog so it will not be left on if the
    // program panics.
    std::panic::set_hook(Box::new(|panic_err| {
        eprintln!("QuilLoad paniced!");
        if let Err(err) = Command::new("qndb")
            .arg("-m")
            .arg("ndbWifiKeepalive")
            .arg("true")
            .spawn()
        {
            eprintln!("Could not disable the wireless watchdog: {}", err);
        }
        eprintln!("{}", panic_err);
    }));

    // Disable the wireless watchdog
    if let Err(err) = Command::new("qndb")
        .arg("-m")
        .arg("ndbWifiKeepalive")
        .arg("true")
        .spawn()
    {
        eprintln!("Could not disable wireless watchdog: {}", err);
    }

    send_backup();

    // Re-enable the wireless watchdog
    if let Err(err) = Command::new("qndb")
        .arg("-m")
        .arg("ndbWifiKeepalive")
        .arg("false")
        .spawn()
    {
        eprintln!("Could not disable wireless watchdog: {}", err);
    }
}
fn send_backup() {
    let mut quillwrite_ip_address =
        fs::read_to_string("/mnt/onboard/.adds/quillconfig/ip_address.conf")
            .unwrap_or(String::from("0.0.0.0"));
    quillwrite_ip_address.push_str(":3333");
    match TcpStream::connect(quillwrite_ip_address) {
        Ok(mut stream) => {
            println!("Successfully connected to server on port 3333");
            if let Ok(file) = File::open("/dev/mmcblk0") {
                let mut compressed_file = flate2::read::GzEncoder::new(file, Compression::fast());
                if let Err(err) = copy(&mut compressed_file, &mut stream) {
                    eprintln!("Could not send file to kobo: {}", err)
                }
            } else {
                eprintln!("Could not open /dev/mmcblk0");
            }

            // let mut encoder = zstd::stream::Encoder::new(&mut stream, 3).unwrap();
            // let mut encoder = EncoderBuilder::new().level(4).build(&mut stream).unwrap();
            // copy(&mut file, &mut encoder).unwrap();
            // let (_output, result) = encoder.finish();
            // result.unwrap();
            // zstd::stream::copy_encode(&mut file, &mut stream, 3).unwrap();
        }
        Err(err) => {
            eprintln!("Failed to connect to server: {}", err);
        }
    }
}
