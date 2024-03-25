use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};
use structs::UserData;
use tokio::{
    net::TcpListener,
    sync::{broadcast, Mutex},
};

mod handler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let chat = Arc::new(Mutex::new(HashMap::<SocketAddr, UserData>::new()));
    let (tx, _) = broadcast::channel::<String>(10);
    let port = env::var("PORT").unwrap_or(String::from("3000"));
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let thr_chat = chat.clone();

        let tx = tx.clone();

        tokio::spawn(async move {
            handler::handler(socket, addr, thr_chat, tx).await;
        });
    }
}
