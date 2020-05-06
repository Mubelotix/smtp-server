use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
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
        
    loop {
        let mut t = [0;128];
        let i = stream.read(&mut t)?;
        
        if i == 0 {
            sleep(Duration::from_millis(10));
            continue;
        }

        trace!("{} bytes read", i);
        let rep = String::from_utf8(t[..i].to_vec()).unwrap();
        
        if let Ok(command) = rep.parse::<Command>() {
            match command {
                Command::Helo(_) => {
                    let _written = stream.write(format!("250 {}\r\n", DOMAIN).as_bytes())?;
                },
                Command::Ehlo(domain) => {
                    let _written = stream.write(format!("250-{} greets {}\r\n", DOMAIN, domain).as_bytes())?;
                    let _written = stream.write(b"250 STARTTLS\r\n")?;
                }
                Command::Mail | Command::Recipient | Command::Reset => {
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
                    debug!("{}", String::from_utf8_lossy(&mail));

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
                _ => (),
            }
        } else {
            debug!("500 Syntax error, command unrecognized\r\n");
            let _written = stream.write(b"500 Syntax error, command unrecognized\r\n")?;
        }
        
    }
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let listener = TcpListener::bind("0.0.0.0:25")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("New client");
        handle_client(stream?);
    }
    Ok(())
}

#[test]
fn test() {
    let mut test = TcpStream::connect("smtp.gmail.com:25").unwrap();
    let mut t = [0;128];
    let i = test.read(&mut t).unwrap();
    println!("{}", i);
}