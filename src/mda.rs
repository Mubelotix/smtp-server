use crate::{
    commands::*, /*mta::transfert_mail, */replies::Reply,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use tokio::net::TcpStream as TokioTcpStream;
use bytes::BytesMut;
use crate::config::Config;
use std::future::Future;
use crate::stream::TcpStream;

#[derive(Debug, PartialEq)]
pub enum OwnedServerIdentity {
    Domain(String),
    Ipv4(String),
}

#[derive(Debug, PartialEq)]
pub enum OwnedRecipient {
    Postmaster(String),
    LocalPostmaster,
    Path(String, OwnedServerIdentity)
}

impl<'a> From<Recipient<'a>> for OwnedRecipient {
    fn from(r: Recipient) -> Self {
        match r {
            Recipient::Postmaster(d) => OwnedRecipient::Postmaster(d.to_string()),
            Recipient::LocalPostmaster => OwnedRecipient::LocalPostmaster,
            Recipient::Path(Path(_sr,(lp, si))) => {
                let lp = match lp {
                    LocalPart::DotString(ds) => ds.to_string(),
                    LocalPart::QuotedString(qs) => qs,
                };
                let si = match si {
                    ServerIdentity::Domain(d) => OwnedServerIdentity::Domain(d.to_string()),
                    ServerIdentity::Ipv4(ip) => OwnedServerIdentity::Ipv4(ip.to_string()),
                };
                OwnedRecipient::Path(lp, si)
            },
        }
    }
}

pub async fn handle_client<F, F2, F3, R, R2, R3>(socket: TokioTcpStream, config: Config, mut verify_user: F, mut get_mailing_list: F2, mut deliver_mail: F3) where
    F: FnMut(&str) -> R,
    R: Future<Output = bool>,
    F2: FnMut(&str) -> R2,
    R2: Future<Output = Option<Vec<String>>>,
    F3: FnMut((String, OwnedServerIdentity), Vec<OwnedRecipient>, &str) -> R3,
    R3: Future<Output = Result<(), &'static str>> {
    debug!("New client: {:?}", socket);
    let mut socket = TcpStream::Unencrypted(socket);

    socket.send_reply(Reply::ServiceReady().with_message(format!(
        "{} {}: Service ready", config.domain(), config.server_agent()
    ))).await.unwrap();

    let mut reverse_path: Option<(String, OwnedServerIdentity)> = None;
    let mut forward_path: Vec<OwnedRecipient> = Vec::new();

    loop {
        let mut b = BytesMut::new();

        // The `read` method is defined by this trait.
        let n = socket.read_buf(&mut b).await.unwrap();
        let s = std::str::from_utf8(&b[..n]).unwrap();

        if s.is_empty() {
            warn!("Empty packet received.");
            socket.shutdown().await.unwrap();
            break;
        }
        
        let command = match Command::from_str(s) {
            Ok(command) => {
                debug!("Received command: {:?}", command);
                command
            },
            Err(e) => {
                error!("Failed to parse command: {:?} -> {:?}", s, e);
                socket.send_reply(Reply::SyntaxError().with_message("Unrecognized command".to_string())).await.unwrap();
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
                    config.domain(),
                    peer_domain,
                    if config.tls_enabled() {
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
                    config.domain(),
                    peer_domain
                ))).await.unwrap();
            },
            Command::From(path, _parameters) => {
                if let Some(Path(_sr,(lp, si))) = path {
                    // TODO verify identity
                    let lp = match lp {
                        LocalPart::DotString(ds) => ds.to_string(),
                        LocalPart::QuotedString(qs) => qs,
                    };
                    let si = match si {
                        ServerIdentity::Domain(d) => OwnedServerIdentity::Domain(d.to_string()),
                        ServerIdentity::Ipv4(ip) => OwnedServerIdentity::Ipv4(ip.to_string()),
                    };
                    reverse_path = Some((lp, si));
                    forward_path.clear();

                    socket.send_reply(Reply::Ok().with_message(format!(
                        "user recognized"
                    ))).await.unwrap();
                } else {
                    socket.send_reply(Reply::UserNotLocal().with_message(format!(
                        "please specify an existing user"
                    ))).await.unwrap();
                }
            }
            Command::To(recipient, _parameters) => {
                let recipient = recipient.into();
                if !forward_path.contains(&recipient) {
                    forward_path.push(recipient);

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

                socket.send_reply(Reply::Ok().with_message(format!(
                    "OK"
                ))).await.unwrap();
            }
            Command::Verify(user) => {
                if verify_user(user.as_str()).await {
                    socket.send_reply(Reply::Ok().with_message(format!(
                        "User recognized"
                    ))).await.unwrap();
                } else {
                    socket.send_reply(Reply::MailboxNotCorrect().with_message(format!(
                        "User Ambiguous"
                    ))).await.unwrap();
                }
            }
            Command::Expand(list_name) => {
                if let Some(mailing_list) = get_mailing_list(list_name.as_str()).await {
                    socket.send_reply(Reply::Ok().with_message(format!(
                        "{}", mailing_list.join("\n")
                    ))).await.unwrap();
                } else {
                    socket.send_reply(Reply::ActionNotTaken().with_message(format!(
                        "There is no mailing list with this name"
                    ))).await.unwrap();
                }
            }
            Command::Help(e) => {
                match e {
                    Some(e) => socket.send_reply(Reply::Ok().with_message(format!(
                        "Thanks for using this SMTP server! You asked help about {:?}", e.as_str()
                    ))).await.unwrap(),
                    None => socket.send_reply(Reply::Ok().with_message(format!(
                        "Thanks for using this SMTP server!"
                    ))).await.unwrap()
                }
            }
            Command::Noop(e) => {
                match e {
                    Some(e) => socket.send_reply(Reply::Ok().with_message(format!(
                        "It is a very sad thing that nowadays there is so little useless information.\nThank you for your {} useless bytes.", e.as_str().len(),
                    ))).await.unwrap(),
                    None => socket.send_reply(Reply::Ok().with_message(format!(
                        "It is better of course to do useless things than to do nothing."
                    ))).await.unwrap()
                }
            }
            Command::Quit => {
                socket.send_reply(Reply::ServiceClosingTransmissionChannel().with_message(format!(
                    "Goodbye!",
                ))).await.unwrap();
                socket.shutdown().await.unwrap();
                break;
            }
            Command::StartTLS => {
                if let Some(tls_acceptor) = config.tls_acceptor() {
                    socket.send_reply(Reply::ServiceReady().with_message("Let's encrypt!".to_string())).await.unwrap();
                    socket = match socket.accept(tls_acceptor).await {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed handshake with client: {}", e);
                            break;
                        }
                    }
                } else {
                    socket.send_reply(Reply::SyntaxError().with_message("Unrecognized command".to_string())).await.unwrap();
                }
            },
            Command::Data => {
                socket.send_reply(Reply::StartMailInput().with_message(format!(
                    "Go ahead!",
                ))).await.unwrap();
                let mut b = BytesMut::new();
                loop {
                    socket.read_buf(&mut b).await.unwrap();
                    if b.ends_with(b"\r\n.\r\n") {
                        break;
                    }
                }
                b.truncate(b.len() - 3);
                let mail = std::str::from_utf8(&b).unwrap();

                match deliver_mail(reverse_path.take().unwrap(), forward_path, mail).await {
                    Ok(()) => socket.send_reply(Reply::Ok().with_message(format!(
                        "Status confirmed, all bytes are down and the mail is secure.",
                    ))).await.unwrap(),
                    Err(e) => socket.send_reply(Reply::ActionAborted().with_message(format!(
                        "Mail not delivered: {}", e
                    ))).await.unwrap()
                }
                forward_path = Vec::new();

                
            }
        }
    }
    
}
