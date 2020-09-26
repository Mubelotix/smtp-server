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

pub async fn handle_client(mut socket: TcpStream, domain: Arc<String>) {
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
                //forward_path.clear();

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
            }
            _ => {
                socket.write_all(Reply::CommandNotImplemented().to_string().as_bytes()).await.unwrap();
            }
        }
    }
    
}
