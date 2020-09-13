#![allow(clippy::cognitive_complexity)]
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::{prelude::*, ErrorKind};
use std::net::TcpListener;
use std::sync::Arc;

pub mod address;
/// TODO support 8-bit https://tools.ietf.org/html/rfc1652
pub mod commands;
pub mod mda;
pub mod mta;
pub mod replies;
pub mod tcp_stream;

use clap::clap_app;

fn main() -> std::io::Result<()> {
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
    let domain = matches.value_of("DOMAIN").unwrap();

    info!(
        "Launching SMTP server on port {}. TLS is {}.",
        port,
        if tls_acceptor.is_some() {
            "enabled"
        } else {
            "disabled"
        }
    );

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(listener) => listener,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            error!("The port {} requires sudo power.", port);
            return Err(e);
        }
        Err(e) => return Err(e),
    };

    // accept connections and process them serially
    for stream in listener.incoming().filter(|s| s.is_ok()) {
        let stream = stream.unwrap(); // it can only be ok thanks to the filter above

        if let Ok(ip) = stream.peer_addr() {
            debug!("New client (ip: {})", ip);
        } else {
            debug!("New client");
        }

        debug!(
            "Connection closed. Result: {:?}",
            mda::handle_client(stream, tls_acceptor.clone(), domain)
        );
    }
    Ok(())
}
