use iced::{
    Element, Subscription, Task, Theme, futures::channel::mpsc, keyboard, widget, widget::container,
};
use pages::{chat::chat_page, login::login_page};
use structs::{Color, ReqLogin, Request, RequestData, ResMsg};

use iced_toasts::{ToastContainer, ToastId, ToastLevel, toast, toast_container};

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
    messages_follow: bool,
    messages_scroll: iced::widget::Id,
    toasts: ToastContainer<'static, Message>,
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
    MessagesScrolled(iced::widget::scrollable::Viewport),
    DismissToast(ToastId),
    Keyboard(keyboard::Event),
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            page: Page::Login,
            disconected: true,
            login_field: LoginField {
                url: "".to_string(),
                name: "".to_string(),
                color: Color::default(),
            },
            messages: Vec::default(),
            msg_input: String::default(),
            messages_follow: true,
            messages_scroll: iced::widget::Id::new("chat_messages"),
            toasts: toast_container(Message::DismissToast),
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Subscription(event) => match event {
                client::Event::FailConnection => {
                    self.disconected = true;
                    self.page = Page::Login;
                    self.toasts.push(
                        toast(&format!("Failed to connect to {}", self.login_field.url))
                            .title("Connection")
                            .level(ToastLevel::Error),
                    );
                    Task::none()
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
                    self.messages_follow = true;
                    self.sender = Some(sender);
                    self.toasts.push(
                        toast(&format!("Connected to {}", self.login_field.url))
                            .title("Connection")
                            .level(ToastLevel::Success),
                    );
                    iced::widget::operation::snap_to_end(self.messages_scroll.clone())
                }
                client::Event::Response(res) => match res.data {
                    structs::ResponseData::Err(err) => {
                        println!("{:?}", err);
                        Task::none()
                    }
                    structs::ResponseData::Msg(msg) => {
                        self.messages.push(msg);
                        if self.messages_follow {
                            iced::widget::operation::snap_to_end(self.messages_scroll.clone())
                        } else {
                            Task::none()
                        }
                    }
                    structs::ResponseData::Join(join) => {
                        self.toasts.push(
                            toast(&format!("{} joined", join))
                                .title("Join")
                                .level(ToastLevel::Info),
                        );
                        Task::none()
                    }
                    structs::ResponseData::Leave(leave) => {
                        self.toasts.push(
                            toast(&format!("{} left", leave))
                                .title("Leave")
                                .level(ToastLevel::Info),
                        );
                        Task::none()
                    }
                },
            },
            Message::LoginSubmit => {
                if self.login_field.url.trim().is_empty() {
                    self.toasts.push(
                        toast("Please enter a server URL")
                            .title("Connection")
                            .level(ToastLevel::Warning),
                    );
                } else {
                    self.disconected = false;
                }
                Task::none()
            }
            Message::MsgSend => {
                if self.msg_input.trim().is_empty() {
                    return Task::none();
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
                Task::none()
            }
            Message::LoginFieldChanged(url, name, color) => {
                self.login_field = LoginField { url, name, color };
                Task::none()
            }
            Message::MsgFieldChanged(mgs) => {
                self.msg_input = mgs;
                Task::none()
            }
            Message::MessagesScrolled(viewport) => {
                self.messages_follow = is_scrollable_at_bottom(&viewport);
                Task::none()
            }
            Message::Keyboard(_) => Task::none(),
            Message::DismissToast(id) => {
                self.toasts.dismiss(id);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match self.page {
            Page::Login => login_page(&self.login_field),
            Page::Chat => chat_page(
                self.messages.clone(),
                self.msg_input.clone(),
                self.messages_scroll.clone(),
            ),
        };

        self.toasts.view(container(content).padding(20))
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn app_update(state: &mut Layout, message: Message) -> Task<Message> {
    match message {
        Message::Keyboard(event) => match event {
            keyboard::Event::KeyPressed { key, .. } => {
                use iced::keyboard::key::Key as KKey;
                use iced::keyboard::key::Named as KNamed;
                match key.as_ref() {
                    KKey::Named(KNamed::Tab) => {
                        return widget::operation::focus_next::<Message>();
                    }
                    _ => {}
                }
                Task::none()
            }
            _ => Task::none(),
        },
        other => Layout::update(state, other),
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

const SCROLL_BOTTOM_THRESHOLD: f32 = 16.0;

fn is_scrollable_at_bottom(viewport: &iced::widget::scrollable::Viewport) -> bool {
    let bounds = viewport.bounds();
    let content_bounds = viewport.content_bounds();
    let max_scroll_y = (content_bounds.height - bounds.height).max(0.0);

    if max_scroll_y <= SCROLL_BOTTOM_THRESHOLD {
        return true;
    }

    let current_scroll_y = viewport.absolute_offset().y;
    (max_scroll_y - current_scroll_y) <= SCROLL_BOTTOM_THRESHOLD
}
