use pcap::{Capture, Device};
use serenity::all::{CreateCommand, ResolvedOption};
use std::collections::HashSet;
use std::net::{Ipv4Addr, TcpStream};
use std::time::{Duration, SystemTime};

fn get_ip() -> Option<String> {
    get_if_addrs::get_if_addrs()
        .unwrap()
        .into_iter()
        .filter(|interface| !interface.is_loopback() && interface.ip().is_ipv4())
        .map(|interface| interface.ip().to_string())
        .collect::<Vec<String>>()
        .first()
        .cloned()
}

fn is_port_open(port: u16) -> std::io::Result<TcpStream> {
    let address = format!("{}:{}", get_ip().unwrap(), port);
    TcpStream::connect(address)
}

fn get_ips(target_port: u16) -> Result<HashSet<Ipv4Addr>, String> {
    let mut unique_ips: HashSet<Ipv4Addr> = HashSet::new();
    let device = Device::lookup().unwrap().unwrap();
    let capture = Capture::from_device(device)
        .unwrap()
        .promisc(true)
        .snaplen(5000)
        .timeout(100)
        .open()
        .unwrap()
        .setnonblock();

    match capture {
        Ok(mut cap) => {
            cap.filter(&format!("tcp dst port {}", target_port), true)
                .unwrap();
            let now = SystemTime::now();
            println!("Listening...");
            while now.elapsed().unwrap() < Duration::from_millis(800) {
                if let Ok(packet) = cap.next_packet() {
                    let ip_header = &packet[14..34];
                    let src_ip =
                        Ipv4Addr::new(ip_header[12], ip_header[13], ip_header[14], ip_header[15]);
                    unique_ips.insert(src_ip);
                }
            }
            println!("Done.");
            Ok(unique_ips)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn run(_options: &[ResolvedOption]) -> String {
    const PORT: u16 = 25565;
    match is_port_open(PORT) {
        Ok(_) => match get_ips(PORT) {
            Ok(ips) => {
                println!("{:?}", ips.iter());
                format!(
                    "O servidor est\u{00E1} **aberto** com {:?} jogadores.",
                    ips.len()
                )
            }
            Err(e) => format!("O servidor est\u{00E1} **aberto**. {}", e),
        },
        Err(_) => String::from("O servidor est\u{00E1} **fechado**."),
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("servidor").description("Checar status do servidor")
}
