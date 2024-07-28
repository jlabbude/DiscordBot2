use std::net::TcpListener;
use serenity::all::{CreateCommand, ResolvedOption};

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

fn is_port_open(port: u16) -> std::io::Result<TcpListener> {
    let address = format!("{}:{}", get_ip().unwrap(), port);
    TcpListener::bind(&address)
}

pub fn run(_options: &[ResolvedOption]) -> String {
    String::from(match is_port_open(25565) {
        // Yes this is correct
        Err(_) => "O servidor est\u{00E1} **aberto**.",
        Ok(_) => "O servidor est\u{00E1} **fechado**.",
    })
}

pub fn register() -> CreateCommand {
    CreateCommand::new("servidor").description("A ping command")
}