use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, ErrorKind};
use log::{trace, debug, info, warn, error};
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

/// TODO support 8-bit https://tools.ietf.org/html/rfc1652

pub mod commands;
use commands::Command;

pub const DOMAIN: &str = "mubelotix.dev";

pub enum Stream {
    Encrypted(TlsStream<TcpStream>),
    Unencryted(TcpStream),
}

impl Read for Stream {
    fn read(&mut self, mut buffer: &mut [u8]) -> std::io::Result<usize> {
        let bytes = match self {
            Stream::Encrypted(stream) => stream.read(&mut buffer),
            Stream::Unencryted(stream) => stream.read(&mut buffer)
        };
        if let Ok(bytes) = bytes {
            if bytes > 0 {
                if buffer[..bytes].ends_with(&[b'\r', b'\n']) {
                    debug!("\x1B[35m{}\x1B[0m", String::from_utf8_lossy(&buffer[..bytes - 2]));
                } else {
                    debug!("\x1B[35m{}\x1B[0m", String::from_utf8_lossy(&buffer[..bytes]));
                }
            }
        }
        bytes
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let bytes = match self {
            Stream::Encrypted(stream) => stream.write(&buffer),
            Stream::Unencryted(stream) => stream.write(&buffer)
        };
        if let Ok(bytes) = bytes {
            if buffer[..bytes].ends_with(&[b'\r', b'\n']) {
                debug!("\x1B[32m{}\x1B[0m", String::from_utf8_lossy(&buffer[..bytes - 2]));
            } else {
                debug!("\x1B[32m{}\x1B[0m", String::from_utf8_lossy(&buffer[..bytes]));
            }
        }
        bytes
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Encrypted(stream) => stream.flush(),
            Stream::Unencryted(stream) => stream.flush()
        }
    }
}

impl Stream {
    fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Encrypted(stream) => stream.shutdown(),
            Stream::Unencryted(stream) => stream.shutdown(std::net::Shutdown::Both)
        }
    }

    fn is_encrypted(&self) -> bool {
        match self {
            Stream::Encrypted(_) => true,
            Stream::Unencryted(_) => false
        }
    }
}

fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    // 220 or 554 but wait the Quit while responding 503

    let mut stream = Stream::Unencryted(stream);
    let _read = stream.write(b"220 mubelotix.dev Rust SMTP Server v1.0\r\n")?;

    let mut file = File::open("test.pfx").unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "testingcert").unwrap();
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    let mut from = None;
    let mut to = Vec::new();
    let mut body = None;
        
    loop {
        let mut t = [0;128];
        let i = stream.read(&mut t)?;
        
        if i == 0 {
            sleep(Duration::from_millis(10));
            continue;
        }

        let rep = String::from_utf8(t[..i].to_vec()).unwrap();
        
        if let Ok(command) = rep.parse::<Command>() {
            match command {
                Command::Helo(_) => {
                    let _written = stream.write(format!("250 {}\r\n", DOMAIN).as_bytes())?;
                },
                Command::Ehlo(domain) => {
                    let _written = stream.write(format!("250-{} greets {}\r\n", DOMAIN, domain).as_bytes())?;
                    let _written = stream.write(b"250-AUTH PLAIN\r\n")?;
                    let _written = stream.write(b"250 STARTTLS\r\n")?;
                }
                Command::Recipient(address) => {
                    if address.ends_with(DOMAIN) {
                        to.push(address);

                        let _written = stream.write(b"250 OK\r\n")?;
                    } else {
                        let _written = stream.write(format!("455 The address {} is not hosted on this domain ({})\r\n", address, DOMAIN).as_bytes())?;
                    }
                }
                Command::Mail(adress) => {
                    from = Some(adress);
                    to = Vec::new();
                    body = None;

                    let _written = stream.write(b"250 OK\r\n")?;
                }
                Command::Reset => {
                    from = None;
                    to = Vec::new();
                    body = None;

                    let _written = stream.write(b"250 OK\r\n")?;
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

                    let _written = stream.write(b"250 OK\r\n")?;
                }
                Command::Quit => {
                    let _written = stream.write(b"221 OK\r\n");
                    stream.shutdown();
                    return Ok(());
                }
                Command::StartTls => {
                    let _written = stream.write(b"220 Go ahead\r\n");
                    if let Stream::Unencryted(unencrypted_stream) = stream {
                        if let Ok(encrypted_stream) = acceptor.accept(unencrypted_stream) {
                            stream = Stream::Encrypted(encrypted_stream);
                            info!("TLS enabled");
                        } else {
                            warn!("Failed to enable TLS");
                            return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
                        }
                    }
                }
                Command::Auth(data) => {
                    debug!("{:?}", data.as_bytes());
                    let _written = stream.write(b"235 Authentication successful\r\n")?;
                }
                Command::Noop => {
                    let _written = stream.write(b"250 OK\r\n")?;
                }
                _ => {let written = stream.write(b"502 Command not implemented\r\n")?;},
            }
        } else {
            let _written = stream.write(b"500 Syntax error, command unrecognized\r\n")?;
        }
        
    }
}

use clap::clap_app;

fn main() -> std::io::Result<()> {
    let matches = clap_app!(myapp =>
        (version: "1.1")
        (author: "Mubelotix <mubelotix@gmail.com>")
        (about: "Rust SMTP Server")
        (@arg TLS: --tls +takes_value "Enable TLS by providing a pfx file.")
        (@arg PORT: -p --port +takes_value default_value("25") "Set the listening port.")
    ).get_matches();

    env_logger::init();

    let port: u16 = matches.value_of("PORT").unwrap_or("25").parse().unwrap_or(25);
    let tls_cert_file = matches.value_of("TLS");

    info!("Launching SMTP server on port {}. TLS is {}.", port, if tls_cert_file.is_some() {"enabled"} else {"disabled"});

    let listener = match TcpListener::bind("0.0.0.0:25") {
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
        
        handle_client(stream);

        debug!("Connection closed");
    }
    Ok(())
}