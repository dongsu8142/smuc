use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use structs::{Request, RequestData, ResError, ResMsg, Response, ResponseData, UserData};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{broadcast::Sender, Mutex},
};

pub async fn handler(
    mut socket: TcpStream,
    addr: SocketAddr,
    chat: Arc<Mutex<HashMap<SocketAddr, UserData>>>,
    tx: Sender<String>,
) {
    let mut rx = tx.subscribe();
    tracing::info!("{} joined", addr);
    loop {
        let mut buf = vec![0; u16::MAX.into()];
        tokio::select! {
            result = socket.read(&mut buf) => {
                let buf_len = match result {
                    Err(e) => {
                        tracing::error!("Reading data {}", e);
                        return;
                    }
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => n,
                };

                buf.resize(buf_len, 0);

                let data: Request = match serde_json::from_str(&String::from_utf8_lossy(&buf)) {
                    Err(err) => {
                        let error = Response {
                            status: "ERR".to_string(),
                            data: ResponseData::Err(ResError {
                                kind: "PARSE".to_string(),
                                msg: err.to_string(),
                            })
                        };
                        let _ = socket.write(&serde_json::to_vec(&error).unwrap()).await.unwrap();
                        break;
                    }
                    Ok(data) => data,
                };

                match (data.status.as_str(), &data.data) {
                    ("LOGIN", RequestData::Login(data)) => {
                        let mut add_user = chat.lock().await;

                        for i in add_user.iter() {
                            if i.1.name == data.name {
                                let error = Response {
                                status: "ERR".to_string(),
                                data: ResponseData::Err(ResError {
                                    kind: "PARSE".to_string(),
                                    msg: "Username already exists".to_string(),
                                }),
                            };

                            let _ = socket.write(&serde_json::to_vec(&error).unwrap())
                                .await
                                .unwrap();

                                tracing::info!("{} left", addr);
                                return;
                            }
                        }

                            add_user.insert(
                                addr,
                                UserData {
                                    name: data.name.clone(),
                                    color: data.color.clone(),
                                    addr,
                                },
                            );

                    }
                    ("MSG", RequestData::Msg(msg)) => {
                        let users = chat.lock().await;
                        let sender = users.get(&addr).unwrap();
                        let res = Response {
                            status: "MSG".to_string(),
                            data: ResponseData::Msg(ResMsg {
                                color: sender.color.clone(),
                                user: sender.name.clone(),
                                msg: msg.to_string(),
                            }),
                        };
                        tx.send(serde_json::to_string(&res).unwrap()).unwrap();
                    },
                    (&_, _) => {
                        let error = Response {
                            status: "ERR".to_string(),
                            data: ResponseData::Err({
                                ResError {
                                    kind: "DATA".to_string(),
                                    msg: "Data is not well balanced".into(),
                                }
                            }),
                        };
                        let _ = socket
                            .write(&serde_json::to_vec(&error).unwrap())
                            .await
                            .unwrap();

                    },
                };
            }
            msg = rx.recv() => {
                if let Ok(data) = msg {
                    let _ =socket.write(data.as_bytes()).await.unwrap();
                }
            }
        };
    }
    let mut add_user = chat.lock().await;
    add_user.remove(&addr);
    tracing::info!("{} left", addr);
}
