use crate::{
    address::EmailAddress, commands::Command, mta::transfert_mail, replies::Reply,
    tcp_stream::Stream,
};
use email::MimeMessage;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use native_tls::TlsAcceptor;
use std::{io::prelude::*, net::TcpStream, sync::Arc};

pub fn handle_client(
    stream: TcpStream,
    tls_acceptor: Option<Arc<TlsAcceptor>>,
    domain: &str,
) -> std::io::Result<()> {
    let mut stream = Stream::Unencryted(stream);
    stream.send_reply(
        Reply::ServiceReady().with_message(format!("{} Rust SMTP Server v1.0", domain)),
    )?;

    assert!(tls_acceptor.is_some());

    let mut from: Option<EmailAddress> = None;
    let mut to = Vec::new();
    let mut body: Option<MimeMessage> = None;

    loop {
        let command = match stream.read_command()? {
            Ok(command) => command,
            Err(e) => {
                stream.send_reply(
                    Reply::SyntaxError().with_message(String::from("That command was strange!")),
                )?;
                warn!("Failed to parse command: {:?}", e);
                continue;
            }
        };

        match command {
            Command::Helo(_) => {
                stream.send_reply(Reply::Ok().with_message(domain.to_string()))?;
            }
            Command::Ehlo(peer_domain) => {
                stream.send_reply(Reply::Ok().with_message(format!(
                    "{} greets {}\nAUTH PLAIN{}",
                    domain,
                    peer_domain,
                    if tls_acceptor.is_some() {
                        "\nSTARTTLS"
                    } else {
                        ""
                    }
                )))?;
            }
            Command::Recipient(address) => {
                if address.domain == domain {
                    to.push(address);

                    stream.send_reply(Reply::Ok())?;
                } else if let Some(from) = &from {
                    if
                    /*from.domain == domain ||*/
                    true {
                        to.push(address);

                        stream.send_reply(Reply::Ok())?;
                    } else {
                        stream.send_reply(Reply::UnableToAccomodateParameters().with_message(
                            format!(
                                "The address {} is not hosted on this domain ({})",
                                address, domain
                            ),
                        ))?;
                    }
                }
            }
            Command::Mail(adress) => {
                from = Some(adress);
                to = Vec::new();
                body = None;

                stream.send_reply(Reply::Ok())?;
            }
            Command::Reset => {
                from = None;
                to = Vec::new();
                body = None;

                stream.send_reply(Reply::Ok())?;
            }
            Command::Data => {
                stream.send_reply(Reply::StartMailInput().with_message(String::from("")))?;

                let mut mail: Vec<u8> = Vec::new();
                let mut buffer = [0; 512];
                while !mail.ends_with(&[b'\r', b'\n', b'.', b'\r', b'\n']) {
                    let read = stream.read(&mut buffer)?;
                    mail.append(&mut buffer[..read].to_vec());
                }
                mail.remove(mail.len() - 1);
                mail.remove(mail.len() - 1);
                mail.remove(mail.len() - 1);
                if let Ok(mut file) = std::fs::File::create("mail.txt") {
                    file.write_all(&mail)?;
                }
                let mail = String::from_utf8(mail).unwrap();
                let mail = MimeMessage::parse(&mail).unwrap();
                info!("Received mail: {:#?}", mail);

                body = Some(mail);

                stream.send_reply(Reply::Ok())?;

                if let (Some(to), Some(from)) = (to.get(0), &from) {
                    if to.domain != domain && from.domain == domain {
                        transfert_mail(to, from, body.unwrap(), domain);
                    }
                }
            }
            #[allow(unused_must_use)]
            Command::Quit => {
                stream.send_reply(Reply::Ok());
                stream.shutdown();
                return Ok(());
            }
            Command::StartTls => {
                if let Some(tls_acceptor) = &tls_acceptor {
                    stream.send_reply(Reply::ServiceReady())?;
                    if let Stream::Unencryted(unencrypted_stream) = stream {
                        if let Ok(encrypted_stream) = tls_acceptor.accept(unencrypted_stream) {
                            stream = Stream::Encrypted(encrypted_stream);
                            info!("TLS enabled");
                        } else {
                            warn!("Failed to enable TLS");
                            return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
                        }
                    }
                } else {
                    stream.send_reply(
                        Reply::ActionNotTaken()
                            .with_message(String::from("TLS can't be activated")),
                    )?;
                }
            }
            Command::Auth(data) => {
                debug!("{:?}", data.as_bytes());
                let _written = stream.write(b"235 Authentication successful\r\n")?;
            }
            Command::Noop => {
                stream.send_reply(Reply::Ok())?;
            }
            _ => {
                stream.send_reply(Reply::CommandNotImplemented())?;
            }
        }
    }
}
