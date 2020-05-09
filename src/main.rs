#![allow(clippy::cognitive_complexity)]
use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, ErrorKind};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::sync::Arc;

/// TODO support 8-bit https://tools.ietf.org/html/rfc1652

pub mod commands;
pub mod replies;
pub mod address;
pub mod tcp_stream;
use commands::{Command, ParsingCommandError};
use replies::Reply;
use tcp_stream::Stream;

pub const DOMAIN: &str = "mubelotix.dev";

fn handle_client(stream: TcpStream, tls_acceptor: Option<Arc<TlsAcceptor>>) -> std::io::Result<()> {
    let mut stream = Stream::Unencryted(stream);
    stream.send_reply(Reply::ServiceReady(String::from("mubelotix.dev Rust SMTP Server v1.0")))?;

    assert!(tls_acceptor.is_some());

    let mut from = None;
    let mut to = Vec::new();
    let mut body = None;
        
    loop {
        let command = match stream.read_command()? {
            Ok(command) => command,
            Err(e) => {
                stream.send_reply(Reply::SyntaxError(String::from("That command was strange!")))?;
                warn!("Failed to parse command: {:?}", e);
                continue
            }
        };

        match command {
            Command::Helo(_) => {
                stream.send_reply(Reply::Ok(DOMAIN.to_string()))?;
            },
            Command::Ehlo(domain) => {
                stream.send_reply(Reply::Ok(format!("{} greets {}\nAUTH PLAIN{}", DOMAIN, domain, if tls_acceptor.is_some() {"\nSTARTTLS"} else {""})))?;
            }
            Command::Recipient(address) => {
                if address.domain == DOMAIN {
                    to.push(address);

                    stream.send_reply(Reply::Ok(String::from("OK")))?;
                } else {
                    stream.send_reply(Reply::UnableToAccomodateParameters(format!("The address {} is not hosted on this domain ({})", address, DOMAIN)))?;
                }
            }
            Command::Mail(adress) => {
                from = Some(adress);
                to = Vec::new();
                body = None;

                stream.send_reply(Reply::Ok(String::from("OK")))?;
            }
            Command::Reset => {
                from = None;
                to = Vec::new();
                body = None;

                stream.send_reply(Reply::Ok(String::from("OK")))?;
            }
            Command::Data => {
                let _written = stream.write(b"354\r\n")?;
                
                let mut mail: Vec<u8> = Vec::new();
                let mut buffer = [0;512];
                while !mail.ends_with(&[b'\r', b'\n', b'.', b'\r', b'\n']) {
                    let read = stream.read(&mut buffer)?;
                    mail.append(&mut buffer[..read].to_vec());
                }
                if let Ok(mut file) = std::fs::File::create("mail.txt") {
                    file.write_all(&mail)?;
                }
                let mail = String::from_utf8_lossy(&mail);
                info!("Received mail: {}", mail);
                
                body = Some(mail);

                stream.send_reply(Reply::Ok(String::from("OK")))?;
            }
            #[allow(unused_must_use)]
            Command::Quit => {
                stream.send_reply(Reply::Ok(String::from("OK")));
                stream.shutdown();
                return Ok(());
            }
            Command::StartTls => {
                if let Some(tls_acceptor) = &tls_acceptor {
                    stream.send_reply(Reply::ServiceReady(String::from("Go ahead")))?;
                    if let Stream::Unencryted(unencrypted_stream) = stream {
                        if let Ok(encrypted_stream) = tls_acceptor.accept(unencrypted_stream) {
                            stream = Stream::Encrypted(encrypted_stream);
                            info!("TLS enabled");
                        } else {
                            warn!("Failed to enable TLS");
                            return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
                        }
                    }
                } else {
                    stream.send_reply(Reply::ActionNotTaken(String::from("TLS is not activated")))?;
                }
            }
            Command::Auth(data) => {
                debug!("{:?}", data.as_bytes());
                let _written = stream.write(b"235 Authentication successful\r\n")?;
            }
            Command::Noop => {
                stream.send_reply(Reply::Ok(String::from("OK")))?;
            }
            _ => {stream.send_reply(Reply::CommandNotImplemented(String::from("This server does not implement this command for now.")))?;},
        }
    }
}

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
    ).get_matches();

    env_logger::init();

    let port: u16 = matches.value_of("PORT").unwrap_or("25").parse().unwrap_or(25);
    let tls_acceptor: Option<Arc<TlsAcceptor>> = if matches.occurrences_of("TLS") > 0 {
        let mut file = File::open(matches.value_of("CERT").unwrap()).unwrap();
        let mut identity = vec![];
        file.read_to_end(&mut identity).unwrap();
        let identity = Identity::from_pkcs12(&identity, matches.value_of("PASSWORD").unwrap()).unwrap();
        let acceptor = TlsAcceptor::new(identity).unwrap();
        let acceptor = Arc::new(acceptor);
        
        Some(acceptor)
    } else {
        None
    };

    info!("Launching SMTP server on port {}. TLS is {}.", port, if tls_acceptor.is_some() {"enabled"} else {"disabled"});

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(listener) => listener,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            error!("The port {} requires sudo power.", port);
            return Err(e);
        },
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

        debug!("Connection closed. Result: {:?}", handle_client(stream, tls_acceptor.clone()));
    }
    Ok(())
}