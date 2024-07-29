use std::collections::HashSet;
use std::io;
use std::net::{Ipv4Addr, TcpStream};
use std::os::unix::prelude::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use pcap::{Capture, Device};
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};

enum Options {
    Check,
    Start,
}

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

fn is_port_open(port: u16) -> io::Result<TcpStream> {
    let address = format!("{}:{}", get_ip().unwrap(), port);
    TcpStream::connect(address)
}

async fn get_ips(target_port: u16) -> Result<HashSet<Ipv4Addr>, String> {
    let mut unique_ips: HashSet<Ipv4Addr> = HashSet::new();
    let device = Device::lookup().unwrap().unwrap();
    let mut capture = Capture::from_device(device)
        .unwrap()
        .promisc(true)
        .snaplen(5000)
        .timeout(100)
        .open()
        .expect("Not ran with sudo")
        .setnonblock()
        .map_err(|e| e.to_string())?;

    capture
        .filter(&format!("tcp dst port {}", target_port), true)
        .map_err(|e| e.to_string())?;
    let now = SystemTime::now();
    println!("Listening...");
    while now.elapsed().unwrap() < Duration::from_millis(800) {
        if let Ok(packet) = capture.next_packet() {
            let ip_header = &packet[14..34];
            let src_ip = Ipv4Addr::new(ip_header[12], ip_header[13], ip_header[14], ip_header[15]);
            unique_ips.insert(src_ip);
        }
    }
    println!("Done.");
    Ok(unique_ips)
}

pub async fn check() -> Result<String, String> {
    const PORT: u16 = 25565;
    match is_port_open(PORT) {
        Ok(_) => match get_ips(PORT).await {
            Ok(ips) => {
                println!("{:?}", ips.iter());
                Ok(format!(
                    "O servidor est\u{00E1} **aberto** com {:?} jogadores.",
                    ips.len()
                ))
            }
            Err(e) => Err(format!("O servidor est\u{00E1} **aberto**. {}", e)),
        },
        Err(_) => Err("O servidor est\u{00E1} **fechado**.".to_string()),
    }
}

fn is_process_running(process_name: &str, arg: &str) -> bool {
    let output = Command::new("pgrep")
        .arg("-afl")
        .arg(process_name)
        .output()
        .expect("Failed to execute pgrep");

    println!("{:?}", output);
    println!("{}", !output.status.success());
    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.contains(arg))
}

pub fn start() -> Result<String, String> {
    if is_process_running("java", "craftbukkit-1.21.jar") {
        return Err("Somente uma inst\u{00E2}ncia do servidor \u{00E9} permitida.".to_string());
    }

    unsafe {
        Command::new("sh")
            .arg("-c")
            .arg("/home/lucas/Desktop/testetetete/run.sh")
            .pre_exec(|| {
                libc::setsid();
                Ok(())
            })
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok("Servidor iniciado".into())
}

#[allow(deprecated)]
pub async fn run(options: &[ResolvedOption<'_>]) -> Result<String, String> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(ref _options),
        ..
    }) = options.first()
    {
        match _options {
            &"check" => check().await,
            &"start" => start(),
            _ => Err("Invalid option".into()),
        }
    } else {
        Err("No option".into())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("servidor")
        .description("Checar status do servidor")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "action", "action")
                .name("action")
                .description("action related to the server")
                .kind(CommandOptionType::String)
                .required(true),
        )
}
