#![windows_subsystem = "windows"]

use std::net::UdpSocket;
use azul;
use std::sync::Mutex;
use std::sync::Arc;
use azul::traits::*;

// MODEL ---------------------------------------------------------------------------------------------------------------------------
#[derive(Debug)]
struct ChatDataModel {
    logged_in: bool,
    messaging_model: MessagingDataModel,
    login_model: LoginDataModel,
}

#[derive(Debug, Default)]
struct LoginDataModel {
    port_input: azul::widgets::text_input::TextInputState,
    address_input: azul::widgets::text_input::TextInputState,
}

#[derive(Debug)]
struct MessagingDataModel {
    text_input_state: azul::widgets::text_input::TextInputState,
    messages: Vec<String>,
    socket: Option<UdpSocket>,
    has_new_message: bool,
}

//VIEW -------------------------------------------------------------------------------------------------------------------------------
const CUSTOM_CSS: &str = "
.row { height: 50px; }
.orange {
    background: linear-gradient(to bottom, #f69135, #f37335);
    font-color: white;
    border-bottom: 1px solid #8d8d8d;
}";

impl ChatDataModel {
    fn chat_form(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        let button = azul::widgets::button::Button::with_label("Send")
            .dom()
            .with_class("row")
            .with_class("orange")
            .with_callback(azul::prelude::On::MouseUp, azul::prelude::Callback(Controller::send_pressed));
        let text = azul::widgets::text_input::TextInput::new()
            .bind(info.window, &self.messaging_model.text_input_state, &self)
            .dom(&self.messaging_model.text_input_state)
            .with_class("row");
        let mut dom = azul::prelude::Dom::new(azul::prelude::NodeType::Div)
            .with_child(text)
            .with_child(button);
        for i in &self.messaging_model.messages {
            dom.add_child(azul::widgets::label::Label::new(i.clone()).dom().with_class("row"));
        }
        dom
    }

    fn login_form(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        let button = azul::widgets::button::Button::with_label("Login")
            .dom()
            .with_class("row")
            .with_class("orange")
            .with_callback(
                azul::prelude::On::MouseUp,
                azul::prelude::Callback(Controller::login_pressed));

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

pub fn run() {
    let app = azul::prelude::App::new(ChatDataModel {
        logged_in: false,
        messaging_model: MessagingDataModel {
            text_input_state: azul::widgets::text_input::TextInputState::new(""),
            messages: Vec::new(),
            socket: None,
            has_new_message: false,
        },
        login_model: LoginDataModel::default(),
    }, azul::prelude::AppConfig::default());
    let mut style = azul::prelude::css::native();
    style.merge(azul::prelude::css::from_str(CUSTOM_CSS).unwrap());
    let window = azul::prelude::Window::new(azul::prelude::WindowCreateOptions::default(), style).unwrap();
    app.run(window).unwrap();
}

//CONTROLLER -------------------------------------------------------------------------------------------------------------------------------------------------
struct Controller {}

const TIMEOUT_IN_MILLIS: u64 = 2000;

//Надо бы авто обновление интерфейса сделать при получении нового месседжа с сервера
//Думаю сделать это через демон который будет при добавлении нового сообдения отправлять комманду на перерисовку.
impl Controller {
    fn send_pressed(app_state: &mut azul::prelude::AppState<ChatDataModel>, _event: azul::prelude::WindowEvent<ChatDataModel>) -> azul::prelude::UpdateScreen {
        let mut data = app_state.data.lock().unwrap();
        let message = data.messaging_model.text_input_state.text.clone();
        data.logged_in = true;
        ChatService::send_to_socket(message, &data.messaging_model.socket);
        azul::prelude::UpdateScreen::Redraw
    }

    fn login_pressed(app_state: &mut azul::prelude::AppState<ChatDataModel>, _event: azul::prelude::WindowEvent<ChatDataModel>) -> azul::prelude::UpdateScreen {
        use std::time::Duration;
        if let Some(ref _s) = app_state.data.clone().lock().unwrap().messaging_model.socket {
            return azul::prelude::UpdateScreen::DontRedraw;
        }

        app_state.add_task(Controller::read_from_socket_async, &[]);
        app_state.add_daemon(azul::prelude::Daemon::unique(azul::prelude::DaemonCallback(Controller::redraw_daemon)));
        let mut data = app_state.data.lock().unwrap();
        let local_address = format!("127.0.0.1:{}", data.login_model.port_input.text.clone().trim());
        let socket = UdpSocket::bind(&local_address)
            .expect(format!("can't bind socket to {}", local_address).as_str());
        let remote_address = data.login_model.address_input.text.clone().trim().to_string();
        socket.connect(&remote_address)
            .expect(format!("can't connect to {}", &remote_address).as_str());
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_IN_MILLIS)))
            .expect("can't set time out to read");
        data.logged_in = true;
        data.messaging_model.socket = Option::Some(socket);
        azul::prelude::UpdateScreen::Redraw
    }

    fn read_from_socket_async(app_data: Arc<Mutex<ChatDataModel>>, _: Arc<()>) {
        let socket = Controller::get_socket(app_data.clone());
        loop {
            if let Some(message) = ChatService::read_data(&socket) {
                app_data.modify(|state| {
                    state.messaging_model.has_new_message = true;
                    state.messaging_model.messages.push(message);
                });
            }
        }
    }

    fn redraw_daemon(state: &mut ChatDataModel, _resources: &mut azul::prelude::AppResources) -> (azul::prelude::UpdateScreen, azul::prelude::TerminateDaemon) {
        if state.messaging_model.has_new_message {
            state.messaging_model.has_new_message = true;
            (azul::prelude::UpdateScreen::Redraw, azul::prelude::TerminateDaemon::Continue)
        } else {
            (azul::prelude::UpdateScreen::DontRedraw, azul::prelude::TerminateDaemon::Continue)
        }
    }


    fn get_socket(app_data: Arc<Mutex<ChatDataModel>>) -> Option<UdpSocket> {
        let model = app_data.clone();
        let ref_model = &(model.lock().unwrap().messaging_model.socket);
        let socket = match ref_model {
            Some(s) => Some(s.try_clone().unwrap()),
            _ => None
        };
        std::mem::drop(app_data);
        socket
    }
}

//Services -------------------------------------------------------------------------------------------------
struct ChatService {}

impl ChatService {
    fn read_data(socket: &Option<UdpSocket>) -> Option<String> {
        let mut buf = [0u8; 4096];
        match socket {
            Some(s) => {
                match s.recv(&mut buf) {
                    Ok(count) => Some(String::from_utf8(buf[..count].into())
                        .expect("can't parse to String")),
                    Err(e) => {
                        println!("Error {}", e);
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn send_to_socket(message: String, socket: &Option<UdpSocket>) {
        match socket {
            Some(s) => { s.send(message.as_bytes()).expect("can't send"); }
            _ => return,
        }
    }
}

/*
use azul::{
    prelude::*,
    widgets::{button::Button, label::Label},
};
use std::{
    thread,
    time::{Duration, Instant},
    sync::{Arc, Mutex},
};

struct MyDataModel {
    counter: usize,
}

impl Layout for MyDataModel {
        fn layout(&self, _info: WindowInfo<Self>) -> Dom<Self> {
            let label = Label::new(format!("{}", self.counter)).dom();
            let button = Button::with_label("Update counter").dom()
                .with_callback(On::MouseUp, Callback(update_counter));
            let async_task_button = Button::with_label("Start async").dom()
                .with_callback(On::MouseUp, Callback(start_connection));

            Dom::new(NodeType::Div)
                .with_child(label)
                .with_child(button)
                .with_child(async_task_button)
    }
}

fn update_counter(app_state: &mut AppState<MyDataModel>, _event: WindowEvent<MyDataModel>) -> UpdateScreen {
    app_state.data.modify(|state| state.counter += 1);
    UpdateScreen::Redraw
}

// Problem - blocks UI :(
fn start_connection(app_state: &mut AppState<MyDataModel>, _event: WindowEvent<MyDataModel>) -> UpdateScreen {
    app_state.add_task(start_async_task, &[]);
    app_state.add_daemon(Daemon::unique(DaemonCallback(start_daemon)));
    UpdateScreen::Redraw
}

fn start_daemon(state: &mut MyDataModel, _resources: &mut AppResources) -> (UpdateScreen, TerminateDaemon) {
    thread::sleep(Duration::from_secs(10));
        state.counter += 10000;
        (UpdateScreen::Redraw, TerminateDaemon::Continue)
}

fn start_async_task(app_data: Arc<Mutex<MyDataModel>>, _: Arc<()>) {
     // simulate slow load
    app_data.modify(|state| {
        thread::sleep(Duration::from_secs(10));
        state.counter += 10000;
    });
}

pub fn run() {
    let model = MyDataModel { counter:0 };
    let app = App::new(model, AppConfig::default());
    app.run(Window::new(WindowCreateOptions::default(), css::native()).unwrap()).unwrap();
}
*/