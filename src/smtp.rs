use crate::{commands::*, config::Config, replies::Reply, stream::TcpStream};
use bytes::BytesMut;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use tokio::net::TcpStream as TokioTcpStream;

pub(crate) async fn handle_client(
    socket: TokioTcpStream,
    config: std::sync::Arc<Config>,
    event_handler: std::sync::Arc<dyn crate::events::EventHandler>,
) {
    debug!("New client: {:?}", socket);
    let mut socket = TcpStream::Unencrypted(socket);

    socket
        .send_reply(Reply::ServiceReady().with_message(format!(
            "{} {}: Service ready",
            config.domain, config.server_agent
        )))
        .await
        .unwrap();

    let mut reverse_path: Option<(LocalPart, ServerIdentity)> = None;
    let mut forward_path: Vec<Recipient> = Vec::new();

    loop {
        let mut b = BytesMut::new();

        // The `read` method is defined by this trait.
        let n = socket.read_buf(&mut b).await.unwrap();
        let s = std::str::from_utf8(unsafe {
            // BIG WARNING:
            // This is disabling compiler lifetime checks on the received data.
            // However, your are still disallowed to move references to this data outside of this scope.
            // Please call to_owned() when you need to save data.
            std::mem::transmute::<_, &'static [u8]>(&b[..n])
        }).unwrap();

        if s.is_empty() {
            warn!("Empty packet received.");
            socket.shutdown().await.unwrap();
            break;
        }

        let command = match Command::from_str(s) {
            Ok(command) => {
                debug!("Received command: {:?}", command);
                command
            }
            Err(e) => {
                error!("Failed to parse command: {:?} -> {:?}", s, e);
                socket
                    .send_reply(
                        Reply::SyntaxError().with_message("Unrecognized command".to_string()),
                    )
                    .await
                    .unwrap();
                continue;
            }
        };

        match command {
            Command::Ehlo(peer_domain) => {
                // reset data
                reverse_path = None;
                forward_path.clear();

                // send reply
                socket.send_reply(Reply::Ok().with_message(format!(
                    "{} greets {}{}",
                    config.domain,
                    peer_domain,
                    if config.tls_acceptor.is_some() || config.tls_required {
                        "\nSTARTTLS"
                    } else {
                        ""
                    }
                ))).await.unwrap();
            },
            Command::Helo(peer_domain) => {
                // reset data
                reverse_path = None;
                forward_path.clear();

                // send reply
                socket.send_reply(Reply::Ok().with_message(format!(
                    "{} greets {}",
                    config.domain,
                    peer_domain
                ))).await.unwrap();
            },
            Command::Quit => {
                socket.send_reply(Reply::ServiceClosingTransmissionChannel().with_message("Goodbye!".to_string())).await.unwrap();
                socket.shutdown().await.unwrap();
                break;
            }
            Command::StartTLS => {
                if let Some(tls_acceptor) = &config.tls_acceptor {
                    socket.send_reply(Reply::ServiceReady().with_message("Let's encrypt!".to_string())).await.unwrap();
                    socket = match socket.accept(tls_acceptor).await {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed handshake with client: {}", e);
                            break;
                        }
                    };
                    forward_path.clear();
                    reverse_path = None;
                } else if config.tls_required {
                    socket.send_reply(Reply::TlsUnavailable().with_message("TLS required, but unavailable due to temporary reason".to_string())).await.unwrap();
                } else {
                    socket.send_reply(Reply::SyntaxError().with_message("Unrecognized command".to_string())).await.unwrap();
                }
            },
            Command::Noop(e) => {
                match e {
                    Some(e) => socket.send_reply(Reply::Ok().with_message(format!(
                        "It is a very sad thing that nowadays there is so little useless information.\nThank you for your {} useless bytes.", e.len(),
                    ))).await.unwrap(),
                    None => socket.send_reply(Reply::Ok().with_message("It is better of course to do useless things than to do nothing.".to_string())).await.unwrap()
                }
            }
            _ if config.tls_required && !socket.is_encrypted() => {
                socket.send_reply(Reply::TlsRequired().with_message("Must issue a STARTTLS command first".to_string())).await.unwrap();
            }
            Command::From(path, _parameters) => {
                if let Some(Path(_sr,(lp, si))) = path {
                    // TODO verify identity
                    reverse_path = Some((lp.to_owned(), si.to_owned()));
                    forward_path.clear();

                    socket.send_reply(Reply::Ok().with_message("user recognized".to_string())).await.unwrap();
                } else {
                    socket.send_reply(Reply::UserNotLocal().with_message("please specify an existing user".to_string())).await.unwrap();
                }
            }
            Command::To(recipient, _parameters) => {
                let recipient = recipient.into();
                if !forward_path.contains(&recipient) {
                    forward_path.push(recipient.to_owned());

                    socket.send_reply(Reply::Ok().with_message(format!(
                        "1 recipient added, {} recipients in total", forward_path.len()
                    ))).await.unwrap();
                } else {
                    socket.send_reply(Reply::Ok().with_message(format!(
                        "recipient already added, {} recipients in total", forward_path.len()
                    ))).await.unwrap();
                }
            },
            Command::Reset => {
                forward_path.clear();
                reverse_path = None;

                socket.send_reply(Reply::Ok().with_message("OK".to_string())).await.unwrap();
            }
            Command::Verify(user) => {
                if event_handler.verify_user(user.to_string()).await {
                    socket.send_reply(Reply::Ok().with_message("User recognized".to_string())).await.unwrap();
                } else {
                    socket.send_reply(Reply::MailboxNotCorrect().with_message("User Ambiguous".to_string())).await.unwrap();
                }
            }
            Command::Expand(list_name) => {
                if let Some(mailing_list) = event_handler.expand_mailing_list(list_name.to_string()).await {
                    socket.send_reply(Reply::Ok().with_message(mailing_list.join("\n").to_string())).await.unwrap();
                } else {
                    socket.send_reply(Reply::ActionNotTaken().with_message("There is no mailing list with this name".to_string())).await.unwrap();
                }
            }
            Command::Help(e) => {
                match e {
                    Some(e) => socket.send_reply(Reply::Ok().with_message(format!(
                        "Thanks for using this SMTP server! You asked help about {:?}", e.as_ref()
                    ))).await.unwrap(),
                    None => socket.send_reply(Reply::Ok().with_message("Thanks for using this SMTP server!".to_string())).await.unwrap()
                }
            }
            Command::Data => {
                socket.send_reply(Reply::StartMailInput().with_message("Go ahead!".to_string())).await.unwrap();
                let mut b = BytesMut::new();
                loop {
                    socket.read_buf(&mut b).await.unwrap();
                    if b.ends_with(b"\r\n.\r\n") {
                        break;
                    }
                }
                b.truncate(b.len() - 3);
                use email_parser::prelude::*;

                let email = Email::parse(&b).unwrap();

                match event_handler.on_mail(std::pin::Pin::new(&email)).await {
                    Ok(()) => socket.send_reply(Reply::Ok().with_message("Status confirmed, all bytes are down and the mail is secure.".to_string())).await.unwrap(),
                    Err(e) => socket.send_reply(Reply::ActionAborted().with_message(format!(
                        "Mail not delivered: {}", e
                    ))).await.unwrap()
                }
                forward_path = Vec::new();


            }
        }
    }
}
