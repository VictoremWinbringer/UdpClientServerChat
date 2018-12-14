#[macro_use]
extern crate text_io;

use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use std::sync::mpsc;
//  use std::io::ErrorKind;
  use std::thread;

const TIMEOUT_IN_MILLIS: u64 = 2000;

pub fn run() {
    let socket = create_socket();
    let (sx, rx) = mpsc::channel();
    let socket_ref = socket.try_clone().unwrap();
   thread::spawn( move ||{
       let mut addresses = Vec::<SocketAddr>::new();
        loop {
            let (bytes, source) = rx.recv().unwrap();
            if !addresses.contains(&source) {
                println!(" {} connected to server", source);
                addresses.push(source.clone());
            }
            let result = String::from_utf8(bytes)
                .expect("can't parse to String")
                .trim()
                .to_string();
            println!("received {} from {}", result, source);
            addresses
                .iter()
                .for_each(|s| {
                    socket_ref
                        .send_to(format!("{} : {}", source, result).as_bytes(), s)
                        .expect(format!("can't send to {}", source).as_str());
                });
        }
    });

    loop {
       sx.send(read_data(&socket)).unwrap();
    }
}

fn create_socket() -> UdpSocket {
    println!("Enter port to listen");
    let local_port: String = read!("{}\n");
    let local_address = format!("127.0.0.1:{}", local_port.trim());
    println!("server address {}", &local_address);
    let socket = UdpSocket::bind(&local_address.trim())
        .expect(format!("can't bind socket to {}", &local_address).as_str());
    socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_IN_MILLIS)))
        .expect("can't set time out to read");
    socket
}

fn read_data(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
    let mut buf = [0u8; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((count, address)) => {
                return (buf[..count].into(), address);
            }
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
    }
}