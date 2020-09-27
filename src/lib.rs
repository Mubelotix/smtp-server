#![allow(clippy::cognitive_complexity)]
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub mod commands;
pub mod mda;
pub mod mta;
pub mod replies;
pub mod config;

#[tokio::test]
async fn main_test() {
    use tokio::net::TcpListener;
    use mda::handle_client;
    use crate::config::ConfigBuilder;

    env_logger::init();
    let port = 50587;
    info!(
        "Launching SMTP server on port {}.",
        port,
    );

    let mut listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await.unwrap();
    let config = ConfigBuilder::new("mubelotix.dev").with_server_agent("Rust SMTP server (testing)").build();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let config = config.clone();
        tokio::spawn(async move {
            handle_client(socket, config,  |_s| async {true}, |name| {
                fn asyncize(d: Option<Vec<String>>) -> impl std::future::Future<Output = Option<Vec<String>>> {
                    async {
                        d
                    }
                }

                match name {
                    "administration" => return asyncize(Some(vec!["Mubelotix <mubelotix@mubelotix.dev>".to_string()])),
                    _ => return asyncize(None),
                }
            }, |_from, _to, _mail| async {
                println!("Received a mail!!");
                Ok(())
            }).await;
        });
    }
}
