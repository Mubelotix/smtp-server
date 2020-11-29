#![allow(clippy::cognitive_complexity)]
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub mod commands;
pub mod config;
pub mod mda;
pub mod mta;
pub mod replies;
pub(crate) mod stream;

use crate::config::ConfigBuilder;
use mda::handle_client;
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_mail<'b>(&self, email: std::pin::Pin<&email_parser::email::Email<'b>>) -> Result<(), String>;

    async fn expand_mailing_list(&self, _name: String) -> Option<Vec<String>> {
        None
    }

    async fn verify_user(&self, _name: String) -> bool {
        false
    }
}

pub struct SmtpServer<'a> {
    event_handler: Arc<dyn EventHandler>,
    port: u16,
    tls: Option<(&'a str, &'a str)>,
}

impl<'a> SmtpServer<'a> {
    pub fn new<T: EventHandler + 'static>(event_handler: T) -> SmtpServer<'a> {
        SmtpServer {
            event_handler: Arc::new(event_handler),
            port: 25,
            tls: None,
        }
    }

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn tls(&mut self, file: &'a str, password: &'a str) -> &mut Self {
        self.tls = Some((file, password));
        self
    }

    pub fn run(&mut self) {
        // read tls certificate
        // todo from builder values
        let mut file = File::open("certificate.pfx").unwrap();
        let mut identity = Vec::new();
        file.read_to_end(&mut identity).unwrap();
        let identity = Identity::from_pkcs12(&identity, "password").unwrap();
        let acceptor = TlsAcceptor::new(identity).unwrap();

        // setup config
        let config = ConfigBuilder::new("mubelotix.dev")
            .with_server_agent("Rust SMTP server (testing)")
            .with_tls(acceptor)
            .build();

        futures::executor::block_on(async move {
            // open socket
            let mut listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await.unwrap();

            loop {
                let event_handler = Arc::clone(&self.event_handler);
                let (socket, _) = listener.accept().await.unwrap();
                let config = config.clone();
                tokio::spawn(async move {
                    handle_client(socket, config, event_handler).await;
                });
            }
        })
    }
}
