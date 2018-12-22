#![windows_subsystem = "windows"]

use std::net::UdpSocket;
use azul;
use std::sync::Mutex;
use std::sync::Arc;
use azul::traits::*;

// MODEL ---------------------------------------------------------------------------------------------------------------------------
//Это позволит отображать нашут структуру в виде строки в шаблоне вида {:?} например println!("{:?}",model)
#[derive(Debug)]
//Наша модель данных
//Для того чтобы ее можно было использовать в Azul она обязательно должна реальизовать трейт Layout
struct ChatDataModel {
    //Флаг для проверки того подключен ли пользователь к серверу или нет
    logged_in: bool,
    //Модель для отображения формы для отправки сообщений на сервер и сохранения полученных с сервера сообщений
    messaging_model: MessagingDataModel,
    //Модель для отображения формы для подключения к серверу
    login_model: LoginDataModel,
}

#[derive(Debug, Default)]
struct LoginDataModel {
    //Порт который ввел пользователь. Мы будем его прослушивать нашим сокетом.
    port_input: azul::widgets::text_input::TextInputState,
    //Адрес сервера котовый ввел пользователь. Мы будем к нему подключаться
    address_input: azul::widgets::text_input::TextInputState,
}

#[derive(Debug)]
struct MessagingDataModel {
    //Сообщение пользователя. Мы его отправим на сервер
    text_input_state: azul::widgets::text_input::TextInputState,
    //Массив сообщений которые пришли с сервера
    messages: Vec<String>,
    //Сокет через который мы общаемся с сервером.
    socket: Option<UdpSocket>,
    //Флаг для проверки того, пришло ли нам новое сообщение от сервера
    has_new_message: bool,
}

//VIEW -------------------------------------------------------------------------------------------------------------------------------

//css стили для нашего DOM
const CUSTOM_CSS: &str = "
.row { height: 50px; }
.orange {
    background: linear-gradient(to bottom, #f69135, #f37335);
    font-color: white;
    border-bottom: 1px solid #8d8d8d;
}";


//Трейт для элементов потомков корневого DataModel
trait Layout<T> {
    //Создает елемент DOM на основе данных потомка с типом Т родителя.
    fn layout(&self, info: azul::prelude::WindowInfo<T>, root: &T) -> azul::prelude::Dom<T> where T: Sized + azul::prelude::Layout;
}

impl Layout<ChatDataModel> for LoginDataModel {
    //Создает форму для ввода данных необходимых для подключения к серверу.
    fn layout(&self, info: azul::prelude::WindowInfo<ChatDataModel>, root: &ChatDataModel) -> azul::prelude::Dom<ChatDataModel> {
        //Создаем кнопку с текстовой надписью Login
        let button = azul::widgets::button::Button::with_label("Login")
            //Преобразуем ее в обьект DOM
            .dom()
            //Добавляем ей класс row
            .with_class("row")
            //Добавляем ей css класс orange
            .with_class("orange")
            //Добавляем обработчик события для нажатия на кнопку
            .with_callback(
                azul::prelude::On::MouseUp,
                azul::prelude::Callback(LoginController::login_pressed));

        //Создаем текстовую метку с тектом Enter port to listen и css классом row
        let port_label = azul::widgets::label::Label::new("Enter port to listen:")
            .dom()
            .with_class("row");
        //Создаем текстовое поле для ввода текста с текстом из свойства нашей модели и css классом row
        let port = azul::widgets::text_input::TextInput::new()
            //Привязываем текстовое поле к свойству нашей DataModel
            // Это двухсторонняя привязка. Теперь редактирование TextInput автоматически изменяет
            // текст в свойстве нашей модели и обратное тоже верно. Если мы изменим текст в нашей модели то измениться текст в TextInput
            .bind(info.window, &self.port_input, root)
            .dom(&self.port_input)
            .with_class("row");

        // Тоже что и для port_label
        let address_label = azul::widgets::label::Label::new("Enter server address:")
            .dom()
            .with_class("row");

        //то же что и для port. Двухсторонняя привязка
        let address = azul::widgets::text_input::TextInput::new()
            .bind(info.window, &self.address_input, root)
            .dom(&self.address_input)
            .with_class("row");

        //Создаем корневой DOM элемент в который помещяем наши UI элементы
        azul::prelude::Dom::new(azul::prelude::NodeType::Div)
            .with_child(port_label)
            .with_child(port)
            .with_child(address_label)
            .with_child(address)
            .with_child(button)
    }
}

impl Layout<ChatDataModel> for MessagingDataModel {
    //Создает форму для отправки и чтения сообдений
    fn layout(&self, info: azul::prelude::WindowInfo<ChatDataModel>, root: &ChatDataModel) -> azul::prelude::Dom<ChatDataModel> {
        //Создаем кнопку с тектом Send css классами row, orange и обработчиком события при ее нажатии
        let button = azul::widgets::button::Button::with_label("Send")
            .dom()
            .with_class("row")
            .with_class("orange")
            .with_callback(azul::prelude::On::MouseUp, azul::prelude::Callback(MessagingController::send_pressed));
        //Создаем поле для ввода текста с двухсторонней привязкой с свойству модели self.messaging_model.text_input_state
        // и css классом row
        let text = azul::widgets::text_input::TextInput::new()
            .bind(info.window, &self.text_input_state, root)
            .dom(&self.text_input_state)
            .with_class("row");
        //Создаем корневой дом элемент и помещяем в него наши UI элементы
        let mut dom = azul::prelude::Dom::new(azul::prelude::NodeType::Div)
            .with_child(text)
            .with_child(button);
        //Добавляем тестовые метки которые отображают сообщения которые были написаны в чате
        for i in &self.messages {
            dom.add_child(azul::widgets::label::Label::new(i.clone()).dom().with_class("row"));
        }
        dom
    }
}

impl azul::prelude::Layout for ChatDataModel {
    //Метод который создает конечный DOM и вызваеться каждый раз кода нужно перерисовать интерфейс
    fn layout(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        //Если мы уже подключены к серверу то показываем форму для отправки и чтения сообщений
        //иначе отображаем форму для подключения к серверу
        if self.logged_in {
            self.messaging_model.layout(info, self)
        } else {
            self.login_model.layout(info, self)
        }
    }
}

//Запускает цикл отрисовки GUI и обработки ввода пользователя
pub fn run() {
    //Создаем приложение со стартовыми данными
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
    //Стили используемые приложением по умолчанию
    let mut style = azul::prelude::css::native();
    //Добавляем к ним наши собственные стили
    style.merge(azul::prelude::css::from_str(CUSTOM_CSS).unwrap());
    //Создаем окно в котором будет отображать наше приложение
    let window = azul::prelude::Window::new(azul::prelude::WindowCreateOptions::default(), style).unwrap();
    //Запускаем приложение в этом окне
    app.run(window).unwrap();
}

//CONTROLLER -------------------------------------------------------------------------------------------------------------------------------------------------
struct MessagingController {}

struct LoginController {}

//Таймату в милисекундах после которого будет прервана блокирующая операция чтения из сокета
const TIMEOUT_IN_MILLIS: u64 = 2000;

impl MessagingController {
    //Метод отрабатывает когда пользователь
    // хочет оправить новое сообщение на сервер.
    fn send_pressed(app_state: &mut azul::prelude::AppState<ChatDataModel>, _event: azul::prelude::WindowEvent<ChatDataModel>) -> azul::prelude::UpdateScreen {
        //Получаем во владение мутекс с нашей моделью данных.
        // Это блокирует поток отрисовки интерфейса до тех пор пока мютекс не будет освобожден.
        let mut data = app_state.data.lock().unwrap();
        //Делаем копию введенного пользователем текста
        let message = data.messaging_model.text_input_state.text.clone();
        //Очищаем поле ввода.
        data.messaging_model.text_input_state.text = "".into();
        //Шана функция для отправки сообщения в сокет
        SocketService::send_to_socket(message, &data.messaging_model.socket);
        //Сообщаем фреймворку что после обработки этого события нужно перерисовать интерфейс.
        azul::prelude::UpdateScreen::Redraw
    }
}

impl LoginController {
    //Метод отрабатывает когда пользователь хочет подключиться к серверу
    fn login_pressed(app_state: &mut azul::prelude::AppState<ChatDataModel>, _event: azul::prelude::WindowEvent<ChatDataModel>) -> azul::prelude::UpdateScreen {
        //Если мы уже подключены к серверу то прерываем выполнение метода сообщаем фреймворку
        // что нет необходимости перерисовывать интерфейс.
        if let Some(ref _s) = app_state.data.clone().lock().unwrap().messaging_model.socket {
            return azul::prelude::UpdateScreen::DontRedraw;
        }
        let temp = app_state.data.clone();
        //Получаем во владение мьютекс
        let mut data = temp.lock().unwrap();
        //Создаем сокет
        let socket = SocketService::create_socket(data.login_model.port_input.text.as_str(), data.login_model.address_input.text.as_str());
// Утанавливаем флаг на то что пользователь уже подключился к серверу
        data.logged_in = true;
// Передаем в модель данных созданный сокет
        data.messaging_model.socket = Option::Some(socket);
        //Добавляем задачу которая будет выполняться асинхронно в потоке из пула потоков фреймворка Azul
        //Обращение к мютексу с моделью данных блокриуте обновление UI до тех пор пока мюьютекс не освободиться
        app_state.add_task(TasksService::read_from_socket_async, &[]);
        //Добавляем повторяющуюся задачу которая выполеться в основном потоке.
        // Любые длительные вычисления в этом демоне блокирует обновление интерфейса
        app_state.add_daemon(azul::prelude::Daemon::unique(azul::prelude::DaemonCallback(DaemonService::redraw_daemon)));
        //Сообщаем фреймворку что после обработки этого события нужно перерисовать интерфейс
        azul::prelude::UpdateScreen::Redraw
    }
}

//Services -------------------------------------------------------------------------------------------------
struct TasksService {}

impl TasksService {
    //Асинхронная операция выполняющаяся в пуле потоков фреймворка azul
    fn read_from_socket_async(app_data: Arc<Mutex<ChatDataModel>>, _: Arc<()>) {
        let temp = app_data.clone();
        //Лочим мьютекс и получаем ссылку на сокет
        //Получаем копию сокета из нашей модели данных
        let socket = SocketService::clone_socket(&(temp.lock().unwrap().messaging_model.socket));
        drop(temp);
        loop {
            //Пытаемся прочитать данные из сокета.
            //Если не сделать копию сокета и напрямую ждать тут пока прийдет сообщение из сокета
            // который в мьютексе в нашей модели денных
            // то весь интерфейс переснанет обновляться до тех пор пока мы не освободим мьютекс
            //Если нам прило какоте то сообшение то изменяем нашу модель данных
            // modify делает то же что и .lock().unwrap() с передачей результата в лямбду
            // и освобождением мьютекса после того как закончиться код лямбды
            SocketService::read_data(&socket).map(|message| {
                app_data.modify(|state| {
                    //Устанавливаем флаг на то что у нас новое сообдение
                    state.messaging_model.has_new_message = true;
                    //Добавляем сообщение в массив всех сообщения чата
                    state.messaging_model.messages.push(message);
                });
            });
        }
    }
}

struct DaemonService {}

impl DaemonService {
    //Повторяющаяся синхронная операция выполняющая в основном потоке
    fn redraw_daemon(state: &mut ChatDataModel, _resources: &mut azul::prelude::AppResources) -> (azul::prelude::UpdateScreen, azul::prelude::TerminateDaemon) {
        //Если у нас есть новое сообщение то сообщаем фреймворку что нужно перерисовать
        //интерфейс с нуля и продолжить работу этого демона
        //иначе не рисуем интерфейс с начала но все равно вызываем этот метод в следующем цикле.
        if state.messaging_model.has_new_message {
            state.messaging_model.has_new_message = false;
            (azul::prelude::UpdateScreen::Redraw, azul::prelude::TerminateDaemon::Continue)
        } else {
            (azul::prelude::UpdateScreen::DontRedraw, azul::prelude::TerminateDaemon::Continue)
        }
    }
}

struct SocketService {}

impl SocketService {
    //Читаем денные из сокета
    fn read_data(socket: &Option<UdpSocket>) -> Option<String> {
        //Буффер для данных которые будем считывать из сокета.
        let mut buf = [0u8; 4096];
        socket.as_ref()
            //Блокирующий вызов. Здесь поток выполнения останавливаеться до тех пор пока
            // не будут считанные данные или произойдет таймаут.
            .map(|s| s.recv(&mut buf))
            .and_then(|r|
                r.map_err(|e| println!("Error can't send {}", e))
                    .ok()
            )
            //Получаем строку из массива байт в кодировке UTF8
            .map(|count| String::from_utf8(buf[..count].into()))
            .and_then(|r|
                r.map_err(|e| println!("Error can't send {}", e))
                    .ok()
            )
    }

    //Отправляем строку в сокет
    fn send_to_socket(message: String, socket: &Option<UdpSocket>) {
        //Преобразуем строку в байты в кодировке UTF8
        // и отправляем данные в сокет
        //Запись данных в сокент не блокирующая т.е. поток выполнения продолжит свою работу.
        let _ = socket.as_ref()
            .map(|s| s.send(message.as_bytes()))
            .map(|r| r.map_err(|e| println!("Error can't send {}", e)));
    }

    fn create_socket(port: &str, server_address: &str) -> UdpSocket {
        // Подключаем структуру для представления отрезка времени из стандартной библиотеки
        use std::time::Duration;
        //Считываем введенный пользователем порт и создаем на основе него локальный адресс
// будем прослушивать
        let local_address = format!("127.0.0.1:{}", port.trim());
//Создаем UDP сокет который считывает пакеты приходящие на локальный адресс.
        let socket = UdpSocket::bind(&local_address)
            .expect(format!("can't bind socket to {}", local_address).as_str());
//Считываем введенный пользователем адрес сервера
        let remote_address = server_address.trim();
//Говорим нашему UDP сокету читать пакеты только от этого сервера
        socket.connect(remote_address)
            .expect(format!("can't connect to {}", remote_address).as_str());
//Устанавливаем таймаут для операции чтения из сокета.
//Запись в сокет происходит без ожидания т. е. мы просто пишем данные и не ждем ничего
// а операция чтения из сокета блокирует поток и ждет пока не прийдут данные которые можно считать.
// Если не установить таймаут то операция чтения из сокета будет ждать бесконечно.
        socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_IN_MILLIS)))
            .expect("can't set time out to read");
        socket
    }

    //Создает копию нашего сокета для того чтобы не держать заблокированным
    //Мьютекс с нашей моделью данных
    fn clone_socket(socket: &Option<UdpSocket>) -> Option<UdpSocket> {
        socket.as_ref()
            .map(|s| s.try_clone())
            .and_then(|r|
                r.map_err(|e| println!("Error {}", e))
                    .ok()
            )
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
    //Добавляем асинхроную задачу
    app_state.add_task(start_async_task, &[]);
    //Добавляем демон
    app_state.add_daemon(Daemon::unique(DaemonCallback(start_daemon)));
    UpdateScreen::Redraw
}

fn start_daemon(state: &mut MyDataModel, _resources: &mut AppResources) -> (UpdateScreen, TerminateDaemon) {
    //Блокирует UI на десять секунд
    thread::sleep(Duration::from_secs(10));
        state.counter += 10000;
        (UpdateScreen::Redraw, TerminateDaemon::Continue)
}

fn start_async_task(app_data: Arc<Mutex<MyDataModel>>, _: Arc<()>) {
     // simulate slow load
    app_data.modify(|state| {
        //Блокирует UI на десять секунд
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