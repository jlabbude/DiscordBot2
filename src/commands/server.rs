use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::Read;
use std::net::{Ipv4Addr, TcpStream};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use pcap::{Capture, Device};
use regex::Regex;
use serenity::all::{ActivityData, CommandOptionType, CreateCommand, CreateCommandOption, Member, ResolvedOption, ResolvedValue, UserId};
use serenity::client::Context;
use crate::{DISCORD_ID_LH, DISCORD_ID_PE};

#[derive(strum_macros::EnumString, strum_macros::Display)]
#[allow(non_camel_case_types)]
enum Options {
    check,
    start,
    stop,
    ip,
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
                Ok(match ips.len() {
                    0 => "O servidor est\u{00E1} **aberto** sem jogadores.".to_string(),
                    1 => {
                        format!(
                            "O servidor est\u{00E1} **aberto** com o jogador:\n- {}",
                            get_ign(ips).map_err(|e| e.to_string())?[0]
                        )
                    }
                    _ => {
                        format!(
                            "O servidor est\u{00E1} **aberto** com os jogadores:\n- {}",
                            get_ign(ips).map_err(|e| e.to_string())?.join("\n- ")
                        )
                    }
                })
            }
            Err(e) => Err(format!("O servidor est\u{00E1} **aberto**. {}", e)),
        },
        Err(_) => Err("O servidor est\u{00E1} **fechado**.".to_string()),
    }
}

pub fn is_process_running(process_name: &str, arg: &str) -> bool {
    let output = Command::new("pgrep")
        .arg("-afl")
        .arg(process_name)
        .output()
        .expect("Failed to execute pgrep");

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.contains(arg))
}

pub fn start(ctx: &Context) -> Result<String, String> {
    if is_process_running("java", "craftbukkit-1.21.jar") {
        return Err("Somente uma inst\u{00E2}ncia do servidor \u{00E9} permitida.".to_string());
    }

    Command::new("zellij")
        .arg("attach")
        .arg("--create-background")
        .arg("servermine")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?
        .wait()
        .map_err(|e| e.to_string())?;
    Command::new("zellij")
        .arg("--session")
        .arg("servermine")
        .arg("run")
        .arg("--in-place")
        .arg("--")
        .arg("sh")
        .arg("/home/lucas/Desktop/testetetete/run.sh")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?
        .wait()
        .map_err(|e| e.to_string())?;

        ctx.shard.set_activity(Some(ActivityData::playing(
            "Servidor aberto".to_string()
        )));

    Ok("Servidor iniciado".into())
}

fn get_ign(ips: HashSet<Ipv4Addr>) -> Result<Vec<String>, String> {
    let mut contents = String::new();
    File::open("/home/lucas/Desktop/testetetete/logs/latest.log")
        .map_err(|e| e.to_string())?
        .read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    let mut ign_ip: HashMap<Ipv4Addr, String> = HashMap::new();
    Regex::new(r"(\w+)\[/(\d+\.\d+\.\d+\.\d+):\d+] logged in with entity id")
        .unwrap()
        .captures_iter(contents.as_str())
        .for_each(|matches| {
            let ign = matches.get(1).unwrap().as_str();
            let ip = matches.get(2).unwrap().as_str();
            ign_ip.insert(Ipv4Addr::from_str(ip).unwrap(), ign.to_string());
        });

    let mut igns: Vec<String> = Vec::new();
    ips.iter().for_each(|ip| {
        igns.push(ign_ip.get(ip).unwrap().to_owned());
    });

    Ok(igns)
}

async fn get_server_ip(user: UserId) -> Result<String, String>{
    const FUWAMOCO: &str = "https://tenor.com/view/fuwamoco-fuwawa-mococo-%E3%83%95%E3%83%AF%E3%83%AF-%E3%83%A2%E3%82%B3%E3%82%B3-gif-17545042085204053426";
    let ip = match public_ip::addr_v4().await {
        None => format!("Erro retornando IP. {FUWAMOCO}"),
        Some(ip) => format!("IP do servidor: `{ip}`")
    };
    
    if user == DISCORD_ID_LH || user == DISCORD_ID_PE {
        Ok(ip)
    } else {
        Err(format!("Sem permiss\u{00E3}o seu BOSTA {FUWAMOCO}"))
    }
}

#[allow(deprecated)]
pub async fn run(ctx: &Context, options: &[ResolvedOption<'_>], member: &Option<Box<Member>>) -> Result<String, String> {
    let member = member.as_ref().unwrap().user.id;
    if let Some(ResolvedOption {
        value: ResolvedValue::String(_options),
        ..
    }) = options.first()
    {
        match *_options {
            "check" => check().await,
            "start" => start(ctx),
            "stop" => Err("Not implemented".into()),
            "ip" => get_server_ip(member).await,
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
                .required(true)
                .add_string_choice(Options::start.to_string(), Options::start.to_string())
                .add_string_choice(Options::check.to_string(), Options::check.to_string())
                .add_string_choice(Options::stop.to_string(), Options::stop.to_string())
                .add_string_choice(Options::ip.to_string(), Options::ip.to_string()),
        )
}
