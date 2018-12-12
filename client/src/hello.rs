#![windows_subsystem = "windows"]

use std::net::UdpSocket;

use azul;

const CUSTOM_CSS: &str = "
.row { height: 50px; }
.orange {
    background: linear-gradient(to bottom, #f69135, #f37335);
    font-color: white;
    border-bottom: 1px solid #8d8d8d;
}";

#[derive(Debug)]
struct ChatDataModel {
    logged_in: bool,
    text_input_state: azul::widgets::text_input::TextInputState,
    messages: Vec<String>,
    login_model: LoginDataModel,
    socket: Option<UdpSocket>,
}

#[derive(Debug, Default)]
struct LoginDataModel {
    port_input: azul::widgets::text_input::TextInputState,
    address_input: azul::widgets::text_input::TextInputState,
}

impl ChatDataModel {
    fn chat_form(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        let button = azul::widgets::button::Button::with_label("Send")
            .dom()
            .with_class("row")
            .with_class("orange")
            .with_callback(azul::prelude::On::MouseUp, azul::prelude::Callback(send_pressed));
        let text = azul::widgets::text_input::TextInput::new()
            .bind(info.window, &self.text_input_state, &self)
            .dom(&self.text_input_state)
            .with_class("row");
        let mut dom = azul::prelude::Dom::new(azul::prelude::NodeType::Div)
            .with_child(text)
            .with_child(button);
        for i in &self.messages {
            dom.add_child(azul::widgets::label::Label::new(i.clone()).dom().with_class("row"));
        }
        dom
    }

    fn login_form(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        let button = azul::widgets::button::Button::with_label("Login")
            .dom()
            .with_class("row")
            .with_class("orange")
            .with_callback(azul::prelude::On::MouseUp, azul::prelude::Callback(login_pressed));

        let port_label = azul::widgets::label::Label::new("Enter port to listen:")
            .dom()
            .with_class("row");

        let port = azul::widgets::text_input::TextInput::new()
            .bind(info.window, &self.login_model.port_input, &self)
            .dom(&self.login_model.port_input)
            .with_class("row");

        let address_label = azul::widgets::label::Label::new("Enter server address:")
            .dom()
            .with_class("row");

        let address = azul::widgets::text_input::TextInput::new()
            .bind(info.window, &self.login_model.address_input, &self)
            .dom(&self.login_model.address_input)
            .with_class("row");

        azul::prelude::Dom::new(azul::prelude::NodeType::Div)
            .with_child(port_label)
            .with_child(port)
            .with_child(address_label)
            .with_child(address)
            .with_child(button)
    }
}

impl azul::prelude::Layout for ChatDataModel {
    fn layout(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        if self.logged_in {
            self.chat_form(info)
        } else {
            self.login_form(info)
        }
    }
}

fn send_pressed(app_state: &mut azul::prelude::AppState<ChatDataModel>, event: azul::prelude::WindowEvent<ChatDataModel>) -> azul::prelude::UpdateScreen {
    let mut data = app_state.data.lock().unwrap();
    let message = data.text_input_state.text.clone();
    data.messages.push(message);
    data.text_input_state.text = "".into();
    azul::prelude::UpdateScreen::Redraw
}

fn login_pressed(app_state: &mut azul::prelude::AppState<ChatDataModel>, event: azul::prelude::WindowEvent<ChatDataModel>) -> azul::prelude::UpdateScreen {
    use std::io::ErrorKind;
    use std::thread;
    use std::time::Duration;
    let mut data = app_state.data.lock().unwrap();
    let local_address = format!("127.0.0.1:{}", data.login_model.port_input.text.clole().trim());
    let socket = UdpSocket::bind(&local_address)
        .expect(format!("can't bind socket to {}", local_address).as_str());
    let remote_address = data.login_model.address_input.text.clone().trim();
    socket.connect(remote_address)
        .expect(format!("can't connect to {}", &remote_address).as_str());
    socket.set_read_timeout(Some(Duration::from_millis(100)))
        .expect("can't set time out to read");
    data.logged_in = true;
    data.socket = Option::Some(socket);
    azul::prelude::UpdateScreen::Redraw
}

pub fn hello_client() {
//    use std::io::ErrorKind;
//    use std::thread;
//    use std::time::Duration;
//    println!("Enter port to listen");
//    let local_port: String = read!("{}\n");
//    let local_address = format!("127.0.0.1:{}", local_port.trim());
//    let socket = UdpSocket::bind(&local_address)
//        .expect(format!("can't bind socket to {}", local_address).as_str());
//    println!("Enter server address");
//    let remote_address: String = read!("{}\n");
//    println!("server: {}", &remote_address);
//    socket.connect(remote_address.clone().trim())
//        .expect(format!("can't connect to {}", &remote_address).as_str());
//    socket.set_read_timeout(Some(Duration::from_millis(100)))
//        .expect("can't set time out to read");
//    let mut buf = [0u8; 4096];
//    loop {
//        let message: String = read!("{}\n");
//        socket.send(message.as_bytes())
//            .expect("can't send");
//        read_data(&socket, &mut buf, &remote_address);
//    }

    let app = azul::prelude::App::new(ChatDataModel {
        logged_in: false,
        text_input_state: azul::widgets::text_input::TextInputState::new(""),
        messages: Vec::new(),
        login_model: LoginDataModel::default(),
        socket: Option::None
    }, azul::prelude::AppConfig::default());
    let mut style = azul::prelude::css::native();
    style.merge(azul::prelude::css::from_str(CUSTOM_CSS).unwrap());
    let window = azul::prelude::Window::new(azul::prelude::WindowCreateOptions::default(), style).unwrap();
    app.run(window).unwrap();
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

