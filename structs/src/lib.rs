use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
pub struct UserData {
    pub name: String,
    pub color: Color,
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub status: String,
    pub data: RequestData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequestData {
    Login(ReqLogin),
    Msg(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReqLogin {
    pub name: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub status: String,
    pub data: ResponseData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseData {
    Err(ResError),
    Msg(ResMsg),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResError {
    pub kind: String,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResMsg {
    pub user: String,
    pub color: Color,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub enum Color {
    #[default]
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
}

impl Color {
    pub const ALL: [Color; 5] = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Purple,
    ];

    pub fn to_rgb(&self) -> [f32; 3] {
        match self {
            Color::Red => [1.0, 0.0, 0.0],
            Color::Green => [0.0, 1.0, 0.0],
            Color::Yellow => [1.0, 1.0, 0.0],
            Color::Blue => [0.0, 0.0, 1.0],
            Color::Purple => [1.0, 0.0, 1.0],
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "Red"),
            Color::Green => write!(f, "Green"),
            Color::Yellow => write!(f, "Yellow"),
            Color::Blue => write!(f, "Blue"),
            Color::Purple => write!(f, "Purple"),
        }
    }
}
