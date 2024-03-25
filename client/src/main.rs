use iced::{futures::channel::mpsc, widget::container, Element, Subscription, Theme};
use pages::{chat::chat_page, login::login_page};
use structs::{Color, ReqLogin, Request, RequestData, ResMsg};

mod client;
mod pages;

fn main() -> iced::Result {
    iced::program(Layout::title, Layout::update, Layout::view)
        .subscription(Layout::subscription)
        .theme(Layout::theme)
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
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
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
        if self.disconected {
            Subscription::none()
        } else {
            client::connect(self.login_field.url.clone()).map(Message::Subscription)
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
        }
    }

    fn view(&self) -> Element<Message> {
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
