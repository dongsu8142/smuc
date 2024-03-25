use iced::{
    futures::{channel::mpsc, SinkExt, StreamExt},
    subscription, Subscription,
};
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

pub enum State {
    Disconnect,
    Connected(mpsc::Receiver<Request>, TcpStream),
}

pub fn connect(addr: String) -> Subscription<Event> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Disconnect;

            loop {
                let mut buf = vec![0; u16::MAX.into()];
                match &mut state {
                    State::Disconnect => match TcpStream::connect(&addr).await {
                        Ok(stream) => {
                            let (tx, rx) = mpsc::channel(100);
                            output.send(Event::Connected(tx)).await.unwrap();
                            state = State::Connected(rx, stream);
                        }
                        Err(_) => {
                            output.send(Event::FailConnection).await.unwrap();
                        }
                    },
                    State::Connected(rx, stream) => {
                        let (mut reader, mut writer) = stream.split();

                        tokio::select! {
                            result = reader.read(&mut buf) => {
                                match result {
                                    Ok(0) => {
                                        continue;
                                    }
                                    Ok(bytes) => {
                                        let data = String::from_utf8_lossy(&buf[..bytes]).to_string();
                                        let response: Response = serde_json::from_str(&data).unwrap();
                                        output.send(Event::Response(response)).await.unwrap();
                                    }
                                    Err(_) => {
                                        let _ = output.send(Event::FailConnection).await;
                                        state = State::Disconnect;
                                    }
                                }
                            }
                            msg = rx.select_next_some() => {
                                let data = serde_json::to_string(&msg).unwrap();
                                if writer.write_all(data.as_bytes()).await.is_err() {
                                    let _ = output.send(Event::FailConnection).await;
                                    state = State::Disconnect;
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}
