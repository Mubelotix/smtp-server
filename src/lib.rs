#![allow(clippy::cognitive_complexity)]
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub mod commands;
pub mod mda;
pub mod mta;
pub mod replies;
pub mod config;
pub(crate) mod stream;

#[tokio::test]
async fn main_test() {
    use tokio::net::TcpListener;
    use mda::handle_client;
    use crate::config::ConfigBuilder;
    use std::fs::File;
    use std::io::prelude::*;
    use native_tls::{Identity, TlsAcceptor};

    env_logger::init();

    // read tls certificate
    let mut file = File::open("certificate.pfx").unwrap();
    let mut identity = Vec::new();
    file.read_to_end(&mut identity).unwrap();
    let identity =
        Identity::from_pkcs12(&identity, "password").unwrap();
    let acceptor = TlsAcceptor::new(identity).unwrap();

    // setup config
    let config = ConfigBuilder::new("mubelotix.dev").with_server_agent("Rust SMTP server (testing)").with_tls(acceptor).build();

    // open socket
    let mut listener = TcpListener::bind("0.0.0.0:25").await.unwrap();

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
