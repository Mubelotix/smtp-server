use crate::smtp::handle_client;
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::events::EventHandler;
use crate::config::Config;

pub struct SmtpServer {
    event_handler: Arc<dyn EventHandler>,
    config: Config,
    port: u16,
}

impl SmtpServer {
    pub fn new<T: EventHandler + 'static>(event_handler: T, domain: &str) -> SmtpServer {
        SmtpServer {
            event_handler: Arc::new(event_handler),
            port: 25,
            config: Config::new(domain.to_string()),
        }
    }

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn tls(&mut self, file: &str, password: &str) -> &mut Self {
        let mut file = File::open(file).unwrap();
        let mut identity = Vec::new();
        file.read_to_end(&mut identity).unwrap();
        let identity = Identity::from_pkcs12(&identity, password).unwrap();
        let acceptor = TlsAcceptor::new(identity).unwrap();
        self.config.tls_acceptor = Some(acceptor.into());

        self
    }

    pub fn run(&mut self) {
        let config = Arc::new(self.config.clone());

        futures::executor::block_on(async move {
            // open socket
            let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await.unwrap();

            loop {
                let event_handler = Arc::clone(&self.event_handler);
                let config = Arc::clone(&config);
                let (socket, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    handle_client(socket, config, event_handler).await;
                });
            }
        })
    }
}
