use std::net::{UdpSocket, SocketAddr};

pub fn hello_server() {
    use std::io::ErrorKind;
    use std::thread;
    use std::time::Duration;

    println!("Enter port to listen");
    let local_port: String = read!("{}\n");
    let local_address = format!("127.0.0.1:{}", local_port.trim());
    println!("server addres {}", &local_address);
    let socket = UdpSocket::bind(&local_address.trim())
        .expect(format!("can't bind socket to {}", &local_address).as_str());
    socket.set_read_timeout(Some(Duration::from_millis(100)))
        .expect("can't set time out to read");
    let mut buf = [0u8; 4096];
    loop {
        let (count, source) = read_data(&socket, &mut buf);
        let result = String::from_utf8(buf[..count].into())
            .expect("can't parse to String");
        println!("received {} from {}", result, source);
        socket.send_to("Hello from server".as_bytes(), source)
            .expect(format!("can't send to {}", source).as_str());
    }
}

fn read_data(socket: &UdpSocket, buf: &mut [u8]) -> (usize, SocketAddr) {
    loop {
        match socket.recv_from(buf) {
            Ok((count, address)) => {
                return (count, address);
            }
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
    }
}