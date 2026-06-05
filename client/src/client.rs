use iced::{
    Subscription,
    futures::{SinkExt, StreamExt, channel::mpsc},
    stream,
};
use std::hash::Hash;
use structs::{Request, Response};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Clone)]
pub enum Event {
    FailConnection,
    Connected(mpsc::Sender<Request>),
    Response(Response),
}

struct ConnectRecipe {
    addr: String,
}

impl iced_futures::subscription::Recipe for ConnectRecipe {
    type Output = Event;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        std::any::TypeId::of::<ConnectRecipe>().hash(state);
        self.addr.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let addr = self.addr;

        iced_futures::boxed_stream(stream::channel(
            100,
            move |mut output: mpsc::Sender<Event>| async move {
                let mut buf = vec![0; u16::MAX.into()];

                match TcpStream::connect(&addr).await {
                    Ok(stream) => {
                        let (tx, mut rx) = mpsc::channel(100);
                        let _ = output.send(Event::Connected(tx)).await;

                        let (mut reader, mut writer) = stream.into_split();

                        loop {
                            tokio::select! {
                                result = reader.read(&mut buf) => {
                                    match result {
                                        Ok(0) => {
                                            let _ = output.send(Event::FailConnection).await;
                                            break;
                                        }
                                        Ok(bytes) => {
                                            let data = String::from_utf8_lossy(&buf[..bytes]).to_string();
                                            let response: Response = serde_json::from_str(&data).unwrap();
                                            let _ = output.send(Event::Response(response)).await;
                                        }
                                        Err(_) => {
                                            let _ = output.send(Event::FailConnection).await;
                                            break;
                                        }
                                    }
                                }
                                msg = rx.select_next_some() => {
                                    let data = serde_json::to_string(&msg).unwrap();
                                    if writer.write_all(data.as_bytes()).await.is_err() {
                                        let _ = output.send(Event::FailConnection).await;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        let _ = output.send(Event::FailConnection).await;
                    }
                }
            },
        ))
    }
}

pub fn connect(addr: String) -> Subscription<Event> {
    iced_futures::subscription::from_recipe(ConnectRecipe { addr })
}
