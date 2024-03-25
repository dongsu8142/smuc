use iced::{
    widget::{container, pick_list, Button, Column, TextInput},
    Element, Length,
};

use crate::{Color, LoginField, Message};

pub fn login_page(login_field: &LoginField) -> Element<Message> {
    container(
        Column::new()
            .spacing(10)
            .push(
                TextInput::new("URL", &login_field.url)
                    .on_input(|url| {
                        Message::LoginFieldChanged(
                            url,
                            login_field.name.clone(),
                            login_field.color.clone(),
                        )
                    })
                    .padding(10)
                    .width(300),
            )
            .push(
                TextInput::new("Name", &login_field.name)
                    .on_input(|name| {
                        Message::LoginFieldChanged(
                            login_field.url.clone(),
                            name,
                            login_field.color.clone(),
                        )
                    })
                    .padding(10)
                    .width(300),
            )
            .push(
                pick_list(&Color::ALL[..], Some(&login_field.color), |color| {
                    Message::LoginFieldChanged(
                        login_field.url.clone(),
                        login_field.name.clone(),
                        color,
                    )
                })
                .padding(10)
                .width(300),
            )
            .push(
                Button::new("Login")
                    .on_press(Message::LoginSubmit)
                    .padding(10),
            )
            .padding(20),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
