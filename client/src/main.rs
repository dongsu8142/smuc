use iced::{
    Element, Subscription, Task, Theme, futures::channel::mpsc, keyboard, widget, widget::container,
};
use pages::{chat::chat_page, login::login_page};
use structs::{Color, ReqLogin, Request, RequestData, ResMsg};

mod client;
mod pages;

fn main() -> iced::Result {
    iced::application(Layout::default, app_update, app_view)
        .title(Layout::title)
        .subscription(app_subscription)
        .theme(app_theme)
        .run()
}

#[derive(Debug)]
struct Layout {
    theme: Theme,
    page: Page,
    disconected: bool,
    login_field: LoginField,
    messages: Vec<ResMsg>,
    msg_input: String,
    sender: Option<mpsc::Sender<Request>>,
}

#[derive(Debug)]
enum Page {
    Login,
    Chat,
}

#[derive(Default, Debug, Clone)]
struct LoginField {
    url: String,
    name: String,
    color: Color,
}

#[derive(Debug, Clone)]
enum Message {
    Subscription(client::Event),
    LoginSubmit,
    MsgSend,
    LoginFieldChanged(String, String, Color),
    MsgFieldChanged(String),
    Keyboard(keyboard::Event),
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            page: Page::Login,
            disconected: false,
            login_field: LoginField {
                url: "".to_string(),
                name: "".to_string(),
                color: Color::default(),
            },
            messages: Vec::default(),
            msg_input: String::default(),
            sender: None,
        }
    }
}

impl Layout {
    fn title(&self) -> String {
        "smuc client".to_string()
    }

    fn subscription(&self) -> Subscription<Message> {
        let keys = iced_futures::keyboard::listen().map(Message::Keyboard);
        if self.disconected {
            // Still listen to keyboard events so Tab/Enter work on the login screen
            keys
        } else {
            let conn = client::connect(self.login_field.url.clone()).map(Message::Subscription);
            Subscription::batch(vec![conn, keys])
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Subscription(event) => match event {
                client::Event::FailConnection => {
                    self.disconected = true;
                    self.page = Page::Login;
                }
                client::Event::Connected(mut sender) => {
                    sender
                        .start_send(Request {
                            status: "LOGIN".to_string(),
                            data: RequestData::Login(ReqLogin {
                                name: self.login_field.name.clone(),
                                color: self.login_field.color.clone(),
                            }),
                        })
                        .unwrap();
                    self.page = Page::Chat;
                    self.sender = Some(sender);
                }
                client::Event::Response(res) => match res.data {
                    structs::ResponseData::Err(err) => {
                        println!("{:?}", err);
                    }
                    structs::ResponseData::Msg(msg) => {
                        self.messages.push(msg);
                    }
                },
            },
            Message::LoginSubmit => {
                self.disconected = false;
            }
            Message::MsgSend => {
                if self.msg_input.trim().is_empty() {
                    return;
                }
                self.sender
                    .as_mut()
                    .unwrap()
                    .start_send(Request {
                        status: "MSG".to_string(),
                        data: RequestData::Msg(self.msg_input.clone()),
                    })
                    .unwrap();
                self.msg_input = String::default();
            }
            Message::LoginFieldChanged(url, name, color) => {
                self.login_field = LoginField { url, name, color }
            }
            Message::MsgFieldChanged(mgs) => {
                self.msg_input = mgs;
            }
            Message::Keyboard(_) => {
                // Handled in the app_update adapter; ignore here to keep match exhaustive.
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match self.page {
            Page::Login => login_page(&self.login_field),
            Page::Chat => chat_page(self.messages.clone(), self.msg_input.clone()),
        };

        container(content).padding(20).into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

// Adapter functions for the iced 0.14 application builder
fn app_update(state: &mut Layout, message: Message) -> Task<Message> {
    match message {
        Message::Keyboard(event) => match event {
            keyboard::Event::KeyPressed { key, .. } => {
                use iced::keyboard::key::Key as KKey;
                use iced::keyboard::key::Named as KNamed;
                match key.as_ref() {
                    KKey::Named(KNamed::Tab) => {
                        // Focus next widget
                        return widget::operation::focus_next::<Message>();
                    }
                    _ => {}
                }
                Task::none()
            }
            _ => Task::none(),
        },
        other => {
            Layout::update(state, other);
            Task::none()
        }
    }
}

fn app_view(state: &Layout) -> Element<'_, Message> {
    Layout::view(state)
}

fn app_subscription(state: &Layout) -> Subscription<Message> {
    Layout::subscription(state)
}

fn app_theme(state: &Layout) -> Option<Theme> {
    Some(Layout::theme(state))
}
