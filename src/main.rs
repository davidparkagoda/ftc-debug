use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::process::exit;
use std::io::ErrorKind;
use std::str;

struct Cli {
    verbose: bool,
    port: u16,
    timeout: u64,
}

macro_rules! print_table_format {
    ($name: expr, $mac_id: expr, $addr: expr, $in_use: expr, $status: expr) => (println!("{:15.15} {:18.18} {:25.25} {:25.25} {:10.10}", $name, $mac_id, $addr, $in_use, $status))
}

fn parse(string: &str) -> Option<(&str, &str, &str, &str)> {
    let mut iter = string.split("\r\n");
    let name = iter.next()?;
    let mac = iter.next()?;
    let (status, owner_ip) = iter.next()?.split_at(1);
    Some((name, mac, owner_ip, status))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    let args = Cli {
        verbose: args.contains(["-v", "--verbose"]),
        port: args.opt_value_from_str(["-p", "--port"])?.unwrap_or(30303),
        timeout: args.opt_value_from_str(["-t", "--timeout"])?.unwrap_or(1),
    };

    if args.verbose
    {
        println!("Verbose {:?}", args.verbose);
        println!("Port {:?}", args.port);
        println!("Timeout {:?}sec", args.timeout);
    }

    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0)))?;
    socket.set_broadcast(true)?;
    socket.send_to(b"D", SocketAddr::from(([255, 255, 255, 255], args.port)))?;
    socket.set_read_timeout(Some(Duration::new(args.timeout, 0)))?;

    print_table_format!("Name", "MAC ID", "Address", "In Use Address", "Status");
    let mut buf = [0; 256];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((bytes_count, source)) => {
                if let Some((name, mac, owner_ip, status)) = str::from_utf8(&buf[..bytes_count]).ok().and_then(parse) {
                    print_table_format!(name, mac, source.to_string(), owner_ip, status);
                };
            }
            Err(error) => {
                match error.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => exit(0),
                    _ => {}
                }
            }
        };
    }
}
