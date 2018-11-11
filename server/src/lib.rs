pub mod hello {
    pub fn hello_server() {
        use std::net::UdpSocket;
        let socket = UdpSocket::bind("127.0.0.1:8877")
            .expect("can't bind socket to 127.0.0.1:8877");
      //  socket.set_nonblocking(true)
        //    .expect("can't set non blocking");
        let mut buf = [0u8; 4096];
        let (count, source) = socket.recv_from(&mut buf)
            .expect("can't receive");
        let result = String::from_utf8(buf[..count].into())
            .expect("can't parse to String");
        println!("received {} from {}", result, source);
        socket.send_to("Hello from server".as_bytes(),source)
            .expect(format!("can't send to {}", source).as_str());
    }
}