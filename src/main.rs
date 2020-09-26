#![allow(clippy::cognitive_complexity)]
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

pub mod commands;
pub mod mda;
pub mod mta;
pub mod replies;

use clap::clap_app;
use mda::handle_client;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.1")
        (author: "Mubelotix <mubelotix@gmail.com>")
        (about: "Rust SMTP Server")
        (@arg PORT:     -p --port +takes_value default_value("25") "Set the listening port.")
        (@arg TLS:      --tls requires("PASSWORD") requires("CERT") "Enable TLS.")
        (@arg CERT:     -c --cert +takes_value "The certificate pfx file.")
        (@arg PASSWORD: --password +takes_value "The password of the certificate.")
        (@arg DOMAIN:   -d --domain +takes_value +required "The hosting domain. Set to your ip in square brackets if you don't own a domain (ex: [127.0.0.1]).")
    )
    .get_matches();

    env_logger::init();

    let port: u16 = matches
        .value_of("PORT")
        .unwrap_or("25")
        .parse()
        .unwrap_or(25);
    let tls_acceptor: Option<Arc<TlsAcceptor>> = if matches.occurrences_of("TLS") > 0 {
        let mut file = File::open(matches.value_of("CERT").unwrap()).unwrap();
        let mut identity = vec![];
        file.read_to_end(&mut identity).unwrap();
        let identity =
            Identity::from_pkcs12(&identity, matches.value_of("PASSWORD").unwrap()).unwrap();
        let acceptor = TlsAcceptor::new(identity).unwrap();
        let acceptor = Arc::new(acceptor);

        Some(acceptor)
    } else {
        None
    };
    let domain = Arc::new(matches.value_of("DOMAIN").unwrap().to_string());

    info!(
        "Launching SMTP server on port {}. TLS is {}.",
        port,
        if tls_acceptor.is_some() {
            "enabled"
        } else {
            "disabled"
        }
    );

    let mut listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let domain = Arc::clone(&domain);
        tokio::spawn(async move {
            handle_client(socket, domain, |_s| true, |name| {
                if name == "administration" {
                    Some(vec!["Mubelotix <mubelotix@mubelotix.dev>".to_string(), "Other <other@mubelotix.dev>".to_string()])
                } else {
                    None
                }
            }, |_from, _to, _mail| {
                println!("Received a mail!!");
                Ok(())
            }).await;
        });
    }
}
