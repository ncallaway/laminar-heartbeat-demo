//! Note that the terms "client" and "server" here are purely what we logically associate with them.
//! Technically, they both work the same.
//! Note that in practice you don't want to implement a chat client using UDP.
use std::net::ToSocketAddrs;
use std::thread;
use std::time::{Duration, Instant};

use laminar::{Config, Packet, Socket, SocketEvent};

use crossbeam_channel::{Receiver, Sender};

const SERVER: &str = "127.0.0.1:12350";
const CLIENT: &str = "127.0.0.1:12355";

fn laminar_config() -> Config {
    Config {
        idle_connection_timeout: Duration::from_secs(5),
        heartbeat_interval: Some(Duration::from_secs(2)),
        ..Default::default()
    }
}

fn server(ci: ConnectionInfo) {
    let start = Instant::now();
    let (mut socket, send, receive) = bind(SERVER, start);
    thread::spawn(move || socket.start_polling());
    listen_loop(ci, send, receive, start);
}

fn client(ci: ConnectionInfo) {
    let start = Instant::now();
    let server = SERVER.parse().unwrap();

    let (mut socket, send, receive) = bind(CLIENT, start);

    thread::spawn(move || socket.start_polling());

    send.send(Packet::reliable_unordered(server, b"Ping".to_vec()))
        .unwrap();
    println!("\t[{:.2?}] Sent: Ping", start.elapsed());

    listen_loop(ci, send, receive, start);
}

fn bind<A: ToSocketAddrs + std::fmt::Display>(
    addresses: A,
    start: Instant,
) -> (Socket, Sender<Packet>, Receiver<SocketEvent>) {
    let mut socket = Socket::bind_with_config(&addresses, laminar_config()).unwrap();
    println!("[{:.1?}] Bound to {}", start.elapsed(), &addresses);
    let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());

    return (socket, sender, receiver);
}

fn listen_loop(
    ci: ConnectionInfo,
    sender: Sender<Packet>,
    receiver: Receiver<SocketEvent>,
    start: Instant,
) {
    loop {
        if let Ok(event) = receiver.recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let msg = String::from_utf8_lossy(packet.payload());
                    let ip = packet.addr().ip();

                    println!("[{:.2?}] Received {:?} from {:?}", start.elapsed(), msg, ip);

                    if let ConnectionInfo::Server { pong } = ci {
                        if pong {
                            sender
                                .send(Packet::reliable_unordered(packet.addr(), b"pong".to_vec()))
                                .unwrap();
                            println!("\t[{:.2?}] Sent: pong", start.elapsed());
                        }
                    }
                }
                SocketEvent::Timeout(address) => {
                    println!("\t[{:.2?}] Timeout from: {}", start.elapsed(), address);
                }
                SocketEvent::Connect(address) => {
                    println!("\t[{:.2?}] Connection from: {}", start.elapsed(), address);
                }
            }
        }
    }
}

fn main() {
    let ci = parse_args();

    if ci.is_server() {
        server(ci);
    } else {
        client(ci);
    }
}
pub enum ConnectionInfo {
    Server { pong: bool },
    Client,
}

impl ConnectionInfo {
    fn is_server(&self) -> bool {
        return match self {
            ConnectionInfo::Server { .. } => true,
            _ => false,
        };
    }
}

fn parse_args() -> ConnectionInfo {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Need to select to run as either a server (--server) or a client (--client).");
    }

    let connection_type = &args[1];

    let is_server = match connection_type.as_str() {
        "--server" | "-s" => true,
        "--client" | "-c" => false,
        _ => panic!("Need to select to run as either a server (--server) or a client (--client)."),
    };

    if !is_server {
        return ConnectionInfo::Client;
    }

    let mut pong = false;
    if args.len() >= 3 {
        pong = match args[2].as_str() {
            "--pong" | "-p" => true,
            _ => false,
        }
    }

    return ConnectionInfo::Server { pong };
}
