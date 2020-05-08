#![allow(clippy::cognitive_complexity)]

use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, ErrorKind};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

/// TODO support 8-bit https://tools.ietf.org/html/rfc1652

pub mod commands;
pub mod replies;
pub mod address;
use commands::{Command, ParsingCommandError};
use replies::Reply;

pub const DOMAIN: &str = "mubelotix.dev";

pub enum Stream {
    Encrypted(TlsStream<TcpStream>),
    Unencryted(TcpStream),
}

impl Stream {
    fn send_command(&mut self, command: Command) -> std::io::Result<()> {
        let command = command.to_string();
        debug!("\x1B[35m{:?}\x1B[0m", command);

        let mut command = command.as_bytes();
        let mut timeout = 0;
        
        while !command.is_empty() && timeout < 20 {
            let written = self.write(command)?;
            command = &command[written..];
            timeout += 1;
        }

        if timeout == 20 {
            warn!("Infinite loop cancelled");
        }

        Ok(())
    }

    fn read_command(&mut self) -> std::io::Result<Result<Command, ParsingCommandError>> {
        let mut command = Vec::new();

        let mut requests = 0;
        while !command.ends_with(&[0x0D, 0x0A]) && requests < 30 {
            let mut t = [0;128];
            let i = self.read(&mut t)?;
            
            if i == 0 {
                sleep(Duration::from_millis(10));
                continue;
            }

            requests += 1;
            command.append(&mut t[..i].to_vec());
        }

        if requests == 30 {
            warn!("Infinite loop cancelled");
        }

        let command = match String::from_utf8(command) {
            Ok(command) => {
                debug!("\x1B[35m{:?}\x1B[0m", command);
                command
            },
            Err(e) => {
                warn!("Server returned invalid utf8. {}", e);
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            },
        };

        Ok(command.parse())
    }

    fn send_reply(&mut self, reply: Reply) -> std::io::Result<()> {
        let reply = reply.to_string();
        debug!("\x1B[32m{:?}\x1B[0m", reply);

        let mut reply = reply.as_bytes();
        let mut timeout = 0;
        
        while !reply.is_empty() && timeout < 20 {
            let written = self.write(reply)?;
            reply = &reply[written..];
            timeout += 1;
        }

        if timeout == 20 {
            warn!("Infinite loop cancelled");
        }

        Ok(())
    }
}

impl Read for Stream {
    fn read(&mut self, mut buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Encrypted(stream) => stream.read(&mut buffer),
            Stream::Unencryted(stream) => stream.read(&mut buffer)
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Encrypted(stream) => stream.write(&buffer),
            Stream::Unencryted(stream) => stream.write(&buffer)
        }
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
    let mut stream = Stream::Unencryted(stream);
    stream.send_reply(Reply::ServiceReady(String::from("mubelotix.dev Rust SMTP Server v1.0")))?;

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
                stream.send_reply(Reply::Ok(format!("{} greets {}\nAUTH PLAIN\nSTARTTLS", DOMAIN, domain)))?;
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
                stream.send_reply(Reply::ServiceReady(String::from("Go ahead")))?;
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

        debug!("Connection closed. Result: {:?}", handle_client(stream));
    }
    Ok(())
}