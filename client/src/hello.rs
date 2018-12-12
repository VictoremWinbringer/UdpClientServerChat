use std::net::UdpSocket;

pub fn hello_client() {
    use std::io::ErrorKind;
    use std::thread;
    use std::time::Duration;
    println!("Enter port to listen");
    let local_port: String = read!("{}\n");
    let local_address = format!("127.0.0.1:{}", local_port.trim());
    let socket = UdpSocket::bind(&local_address)
        .expect(format!("can't bind socket to {}", local_address).as_str());
    println!("Enter server address");
    let remote_address: String = read!("{}\n");
    println!("server: {}", &remote_address);
    socket.connect(remote_address.clone().trim())
        .expect(format!("can't connect to {}", &remote_address).as_str());
    socket.set_read_timeout(Some(Duration::from_millis(100)))
        .expect("can't set time out to read");
    let mut buf = [0u8; 4096];
    loop {
        let message: String = read!("{}\n");
        socket.send(message.as_bytes())
            .expect("can't send");
        read_data(&socket, &mut buf, &remote_address);
    }
}

fn read_data(socket: &UdpSocket, buf: &mut [u8], remote_address: &String) {
    match socket.recv(buf) {
        Ok(count) => {
            let result = String::from_utf8(buf[..count].into())
                .expect("can't parse to String");
            println!("{}", result)
        }
        Err(e) => {
            println!("Error {}", e)
        }
    }
}

