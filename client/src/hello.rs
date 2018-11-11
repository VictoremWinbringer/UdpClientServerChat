pub fn hello_client() {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("127.0.0.1:7788")
        .expect("can't bind socket to 127.0.0.1:7788");
 //   socket.set_nonblocking(true)
    //    .expect("can't set non blocking");
    socket.connect("127.0.0.1:8877")
        .expect("can't connect to 127.0.0.1:8877");
    socket.send("Hello from client!".as_bytes()).expect("can't send");
    let mut buf = [0u8; 4096];
    let (count, source) = socket.recv_from(&mut buf)
        .expect("can't receive");
    let result = String::from_utf8(buf[..count].into())
        .expect("can't parse to String");
    println!("received {} from {}", result, source);
}
