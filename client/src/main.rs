//extern crate client;
//
//fn main(){
//    client::hello::hello_client();
//}

#![windows_subsystem = "windows"]

use azul;

const CUSTOM_CSS: &str = "
.row { height: 50px; }
.orange {
    background: linear-gradient(to bottom, #f69135, #f37335);
    font-color: white;
    border-bottom: 1px solid #8d8d8d;
}";

#[derive(Debug)]
struct MyDataModel {
    text_input_state: azul::widgets::text_input::TextInputState
}

impl azul::prelude::Layout for MyDataModel {
    fn layout(&self, info: azul::prelude::WindowInfo<Self>) -> azul::prelude::Dom<Self> {
        let button = azul::widgets::button::Button::with_label("Send")
            .dom()
            .with_class("row")
            .with_class("orange")
            .with_callback(azul::prelude::On::MouseUp,azul::prelude::Callback(send_pressed));
        let text = azul::widgets::text_input::TextInput::new()
            .bind(info.window,&self.text_input_state,&self)
            .dom(&self.text_input_state)
            .with_class("row");
        let mut dom = azul::prelude::Dom::new(azul::prelude::NodeType::Div)
            .with_class("row")
            .with_child(text)
            .with_child(button);
        for i in 1..100 {
            dom.add_child(azul::widgets::label::Label::new(format!("Empty {}", i)).dom().with_class("row"));
        }
        dom
    }
}

fn send_pressed(app_state: &mut azul::prelude::AppState<MyDataModel>, event: azul::prelude::WindowEvent<MyDataModel>) -> azul::prelude::UpdateScreen {
    println!("send_pressed event: {:?}", event.hit_dom_node);
    azul::prelude::UpdateScreen::Redraw
}

fn main() {
    let app = azul::prelude::App::new(MyDataModel { text_input_state: azul::widgets::text_input::TextInputState::new("") }, azul::prelude::AppConfig::default());
    let mut style = azul::prelude::css::native();
    style.merge(azul::prelude::css::from_str(CUSTOM_CSS).unwrap());
    let window = azul::prelude::Window::new(azul::prelude::WindowCreateOptions::default(), style).unwrap();
    app.run(window).unwrap();
}