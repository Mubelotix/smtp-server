use crate::{
    address::EmailAddress, commands::*, /*mta::transfert_mail, */replies::Reply,
};
use email::MimeMessage;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use tokio::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use bytes::BytesMut;
use std::sync::Arc;

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

pub async fn handle_client<F, F2>(mut socket: TcpStream, domain: Arc<String>, mut verify_user: F, mut get_mailing_list: F2) where
    F: FnMut(&str) -> bool,
    F2: FnMut(&str) -> Option<Vec<String>> {
    println!("GOT: {:?}", socket);

    let mut reverse_path: Option<(LocalPart, ServerIdentity)> = None;
    let mut forward_path: Vec<OwnedRecipient> = Vec::new();

    loop {
        let mut b = [0; 1024];

        // The `read` method is defined by this trait.
        let n = socket.read(&mut b).await.unwrap();
        let s = std::str::from_utf8(&b[..n]).unwrap();
        
        println!("{:?}", s);
        let command = Command::from_str(s).unwrap();
        println!("{:?}", command);
        match command {
            Command::Ehlo(peer_domain) => {
                // reset data
                reverse_path = None;
                forward_path.clear();

                // send reply
                socket.write_all(Reply::Ok().with_message(format!(
                    "{} greets {}",
                    domain,
                    peer_domain
                )).to_string().as_bytes()).await.unwrap();
            },
            Command::Helo(peer_domain) => {
                // reset data
                reverse_path = None;
                forward_path.clear();

                // send reply
                socket.write_all(Reply::Ok().with_message(format!(
                    "{} greets {}",
                    domain,
                    peer_domain
                )).to_string().as_bytes()).await.unwrap();
            },
            Command::From(path, _parameters) => {
                if let Some(path) = path {
                    // TODO verify identity
                    reverse_path = Some(path.1);
                    //forward_path.clear();

                    socket.write_all(Reply::Ok().with_message(format!(
                        "user recognized"
                    )).to_string().as_bytes()).await.unwrap();
                } else {
                    socket.write_all(Reply::UserNotLocal().with_message(format!(
                        "please specify an existing user"
                    )).to_string().as_bytes()).await.unwrap();
                }
            }
            Command::To(recipient, _parameters) => {
                let recipient = recipient.into();
                if !forward_path.contains(&recipient) {
                    forward_path.push(recipient);

                    socket.write_all(Reply::Ok().with_message(format!(
                        "1 recipient added, {} recipients in total", forward_path.len()
                    )).to_string().as_bytes()).await.unwrap();
                } else {
                    socket.write_all(Reply::Ok().with_message(format!(
                        "recipient already added, {} recipients in total", forward_path.len()
                    )).to_string().as_bytes()).await.unwrap();
                }
            },
            Command::Reset => {
                forward_path.clear();
                reverse_path = None;

                socket.write_all(Reply::Ok().with_message(format!(
                    "OK"
                )).to_string().as_bytes()).await.unwrap();
            }
            Command::Verify(user) => {
                if verify_user(user.as_str()) {
                    socket.write_all(Reply::Ok().with_message(format!(
                        "User recognized"
                    )).to_string().as_bytes()).await.unwrap();
                } else {
                    socket.write_all(Reply::MailboxNotCorrect().with_message(format!(
                        "User Ambiguous"
                    )).to_string().as_bytes()).await.unwrap();
                }
            }
            Command::Expand(list_name) => {
                if let Some(mailing_list) = get_mailing_list(list_name.as_str()) {
                    socket.write_all(Reply::Ok().with_message(format!(
                        "{}", mailing_list.join("\n")
                    )).to_string().as_bytes()).await.unwrap();
                } else {
                    socket.write_all(Reply::ActionNotTaken().with_message(format!(
                        "There is no mailing list with this name"
                    )).to_string().as_bytes()).await.unwrap();
                }
            }
            Command::Help(e) => {
                match e {
                    Some(e) => socket.write_all(Reply::Ok().with_message(format!(
                        "Thanks for using this SMTP server! You asked help about {:?}", e.as_str()
                    )).to_string().as_bytes()).await.unwrap(),
                    None => socket.write_all(Reply::Ok().with_message(format!(
                        "Thanks for using this SMTP server!"
                    )).to_string().as_bytes()).await.unwrap()
                }
            }
            Command::Noop(e) => {
                match e {
                    Some(e) => socket.write_all(Reply::Ok().with_message(format!(
                        "It is a very sad thing that nowadays there is so little useless information.\nThank you for your {} useless bytes.", e.as_str().len(),
                    )).to_string().as_bytes()).await.unwrap(),
                    None => socket.write_all(Reply::Ok().with_message(format!(
                        "It is better of course to do useless things than to do nothing."
                    )).to_string().as_bytes()).await.unwrap()
                }
            }
            Command::Quit => {
                socket.write_all(Reply::ServiceClosingTransmissionChannel().with_message(format!(
                    "Goodbye!",
                )).to_string().as_bytes()).await.unwrap();
                socket.shutdown(std::net::Shutdown::Both).unwrap();
            }
            Command::StartTLS => socket.write_all(Reply::CommandNotImplemented().to_string().as_bytes()).await.unwrap(),
            Command::Data => {
                socket.write_all(Reply::StartMailInput().with_message(format!(
                    "Go ahead!",
                )).to_string().as_bytes()).await.unwrap();
                let mut b = BytesMut::new();
                loop {
                    let n = socket.read_buf(&mut b).await.unwrap();
                    if b.ends_with(b"\r\n.\r\n") {
                        break;
                    }
                }
                b.truncate(b.len() - 3);
                
                println!("{}", std::str::from_utf8(&b).unwrap());
            }
        }
    }
    
}
