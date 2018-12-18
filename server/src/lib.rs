#[macro_use]
extern crate text_io;

use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

const TIMEOUT_IN_MILLIS: u64 = 2000;

//Главная точка входа в приложение
pub fn run() {
    //Создаем сокет
    let socket = create_socket();
    //Создаем односторонний канал с одним отправителем сообщений sx и множеством получателей rx
    let (sx, rx) = mpsc::channel();
    //Запускаем рассылку сообщений всем получателям в отдельном потоке
    start_sender_thread(rx, socket.try_clone().unwrap());
    loop {
        //Читаем данные из сокета и оправляем их в поток занимающийся рассылкой сообшений клентам подключенным к серверу
        sx.send(read_data(&socket)).unwrap();
    }
}
//Метод для создания потока для рассылки сообщений клиентам
fn start_sender_thread(rx: mpsc::Receiver<(Vec<u8>, SocketAddr)>, socket: UdpSocket) {
    //Запускаем новый поток. move значит что переменные переходят во владение лямбды и потока соответсвенно
    // Конкретнее наш новый поток "поглотит" переменные rx и socket
    thread::spawn(move || {
        //Коллеция адресов подключенных к нам клиентов. Всем им мы будем разсылать наши сообщения.
        //Вообще в реальном проекте надо бы сделать обработку отключения от нас клиента и удаления его
        // адресса из этого массива.
        let mut addresses = Vec::<SocketAddr>::new();
        //запускаем бесконечный цикл
        loop {
            //Читаем данные из канала. Тут поток будет заблокирован до тех пор пока не прийдут новые данные
            let (bytes, source) = rx.recv().unwrap();
            // Если такого адреса нет в нашем массиве то добавляем его туда
            if !addresses.contains(&source) {
                println!(" {} connected to server", source);
                addresses.push(source.clone());
            }
            //Декодируем UTF8 строку из массива байт
            let result = String::from_utf8(bytes)
                .expect("can't parse to String")
                .trim()
                .to_string();
            println!("received {} from {}", result, source);
            //Создаем массив байт которые собираемся отправить всем нашим клиентам
            let message = format!("FROM: {} MESSAGE: {}", source, result);
            let data_to_send = message.as_bytes();
            //Проходим по коллецкии адресов и отправляем данные каждому.
            addresses
                .iter()
                .for_each(|s| {
                    //Операция записи в UDP сокет неблокирующая поэтому
                    //здесь метод не будет ждать пока сообщение прийдет к получателю и выполниться почти
                    //мнгновенно
                    socket
                        .send_to(data_to_send, s)
                        .expect(format!("can't send to {}", source).as_str());
                });
        }
    });
}

//Создает сокет на основе данных введенных пользователем
fn create_socket() -> UdpSocket {
    println!("Enter port to listen");
    //Считываем порт который будет слушать наш сервер и создаем на его основе адрес сервера
    let local_port: String = read!("{}\n");
    let local_address = format!("127.0.0.1:{}", local_port.trim());
    println!("server address {}", &local_address);
    //Создаем UDP сокет прослущивающий этот адрес
    let socket = UdpSocket::bind(&local_address.trim())
        .expect(format!("can't bind socket to {}", &local_address).as_str());
    //Устанавливаем таймаут для операции чтения. Операция чтения блокирующая и она заблокирует поток
    //до тех пор пока не прийдут новые данные или не наступит таймаут
    socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_IN_MILLIS)))
        .expect("can't set time out to read");
    //Возвращаем из метода созданные сокет
    socket
}

//Читает данные из сокета и возвшает их вместе с адресом оправителя
fn read_data(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
    //Буфер куда будем считывать данные
    let mut buf = [0u8; 4096];
    //Запускает цикл который будет выполняться до тех пор пока не будут считаны валидные данные
    loop {
        match socket.recv_from(&mut buf) {
            //Получаем количество считанных байт и адрес отправителя
            Ok((count, address)) => {
                //Делем срез массива от его начала до количеств считанных байт и преборазуем его в вектор байт
                return (buf[..count].into(), address);
            }
            //Если произошёл таймаут или другая ошибка то переходим к следующей итерации цикла
            Err(e) => {
                println!("Error {}", e);
                continue;
            }
        };
    }
}