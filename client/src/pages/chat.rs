use iced::{
    Element, Length,
    widget::{Button, Column, Text, TextInput, row, scrollable},
};
use structs::ResMsg;

use crate::Message;

pub fn chat_page(
    messages: Vec<ResMsg>,
    msg_input: String,
    messages_scroll: iced::widget::Id,
) -> Element<'static, Message> {
    Column::new()
        .push(
            Column::new()
                .spacing(5)
                .align_x(iced::Alignment::Start)
                .push(Text::new("Messages:").size(24))
                .push(
                    scrollable(
                        Column::with_children(
                            messages
                                .into_iter()
                                .map(|msg| {
                                    row![
                                        Text::new(msg.user.clone())
                                            .size(16)
                                            .color(msg.color.to_rgb()),
                                        Text::new(format!(": {}", msg.msg)).size(16)
                                    ]
                                })
                                .map(Element::from)
                                .collect::<Vec<_>>(),
                        )
                        .spacing(6),
                    )
                    .id(messages_scroll)
                    .on_scroll(Message::MessagesScrolled)
                    .width(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::FillPortion(3)),
        )
        .push(
            Column::new()
                .push(
                    TextInput::new("Message", &msg_input)
                        .on_input(Message::MsgFieldChanged)
                        .on_submit(Message::MsgSend)
                        .padding(10)
                        .width(Length::Fill),
                )
                .push(Button::new("Send").on_press(Message::MsgSend).padding(10)),
        )
        .into()
}
