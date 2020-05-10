#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::net::{*, ToSocketAddrs};
use native_tls::TlsConnector;
use trust_dns_resolver::Resolver;
use std::io::prelude::*;
use trust_dns_resolver::config::*;
use crate::{address::EmailAddress, tcp_stream::Stream, commands::Command, replies::{ReplyType, Reply}};
use std::{net::TcpStream, thread::sleep, time::Duration};

pub fn transfert_mail(to: EmailAddress, from: EmailAddress, mail: String, domain: &str) -> std::io::Result<()> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let response = resolver.mx_lookup(&to.domain).unwrap();

    // TODO select correct server in the iterator
    let mda_mx = response.iter().next().unwrap();
    let mut mda_address = mda_mx.exchange().to_string();
    debug!("Connecting to {}.", mda_address);
    let mda = resolver.lookup_ip(&mda_address).unwrap().iter().next().unwrap();

    let mut stream = Stream::Unencryted(TcpStream::connect((mda, 25))?);
    
    // Get the init message and send Ehlo
    if let Ok(Ok(Reply{reply_type: ReplyType::ServiceReady, message})) = stream.read_reply() {
        mda_address = string_tools::get_all_before(&message, " ").to_string();
        stream.send_command(Command::Ehlo(domain.to_string()))?;
    } else {
        warn!("Service is not ready");
        return Err(std::io::Error::from(std::io::ErrorKind::Other));
    }
    
    // Get the Ehlo response
    if let Ok(Ok(Reply{reply_type: ReplyType::Ok, message})) = stream.read_reply() {
        if message.contains("\nSTARTTLS\n") {
            stream.send_command(Command::StartTls)?;

            if let Ok(Ok(Reply{reply_type: ReplyType::ServiceReady, message: _})) = stream.read_reply() {
                if let Stream::Unencryted(old_stream) = stream {
                    let connector = TlsConnector::new().unwrap();
                    match connector.connect(&mda_address, old_stream) {
                        Ok(new_stream) => {
                            debug!("Tls enabled");
                            stream = Stream::Encrypted(new_stream);
                            stream.send_command(Command::Ehlo(domain.to_string()))?;

                            if let Ok(Ok(Reply{reply_type: ReplyType::Ok, message})) = stream.read_reply() {
                                println!("{:?}", message);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to enable TLS {}", e);
                            return Err(std::io::Error::from(std::io::ErrorKind::Other));
                        },
                    };
                }
            }
        }
    } else {
        warn!("Service is not ready");
        return Err(std::io::Error::from(std::io::ErrorKind::Other));
    }

    stream.send_command(Command::Mail(from));
    stream.read_reply();
    stream.send_command(Command::Recipient(to));
    stream.read_reply();
    stream.send_command(Command::Data);
    stream.read_reply();
    stream.write(mail.as_bytes());
    stream.read_reply();
    stream.send_command(Command::Quit);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::EmailAddress;

    #[test]
    fn transfert_to_google() {
        env_logger::try_init();

        /*transfert_mail(EmailAddress {
            username: String::from("mubelotix"),
            domain: String::from("gmail.com"),
        }).unwrap();*/
    }
}