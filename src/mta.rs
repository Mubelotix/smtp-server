#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::net::{*, ToSocketAddrs};
use native_tls::TlsConnector;
use trust_dns_resolver::Resolver;
use std::io::{prelude::*};
use trust_dns_resolver::config::*;
use crate::{address::EmailAddress, tcp_stream::Stream, commands::Command, replies::{ReplyType, Reply}};
use std::{net::TcpStream, thread::sleep, time::Duration};

pub enum MTAError {
    IOError(std::io::Error),
    DnsError(trust_dns_resolver::error::ResolveError),
    TlsError(native_tls::Error),
    TlsHandshakeError(native_tls::HandshakeError<std::net::TcpStream>),
    NoMxRecord,
    DeadMxRecord,
    ServiceNotReady,
}

impl From<std::io::Error> for MTAError {
    fn from(error: std::io::Error) -> MTAError {
        MTAError::IOError(error)
    }
}

impl From<trust_dns_resolver::error::ResolveError> for MTAError {
    fn from(error: trust_dns_resolver::error::ResolveError) -> MTAError {
        MTAError::DnsError(error)
    }
}

impl From<native_tls::Error> for MTAError {
    fn from(error: native_tls::Error) -> MTAError {
        MTAError::TlsError(error)
    }
}

impl From<native_tls::HandshakeError<std::net::TcpStream>> for MTAError {
    fn from(error: native_tls::HandshakeError<std::net::TcpStream>) -> MTAError {
        MTAError::TlsHandshakeError(error)
    }
}

pub fn transfert_mail(to: EmailAddress, from: EmailAddress, mail: String, domain: &str) -> Result<(), MTAError> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    let response = resolver.mx_lookup(&to.domain)?;

    // TODO select correct server in the iterator

    // Get the domain from a MX record
    let mut mda_domain = match response.iter().next() {
        Some(record) => record.exchange().to_string(),
        None => {
            warn!("No MX record found for {}", to.domain);
            return Err(MTAError::NoMxRecord);
        }
    };

    // Get the ip address from the domain
    let mda_address = match resolver.lookup_ip(&mda_domain)?.iter().next() {
        Some(addr) => addr,
        None => {
            warn!("Domain {} referenced by MX record has no associated ip address", mda_domain);
            return Err(MTAError::DeadMxRecord);
        }
    };

    debug!("Connecting to {}.", mda_domain);
    let mut stream = Stream::Unencryted(TcpStream::connect((mda_address, 25))?);
    
    // Get the init message and send Ehlo
    if let Ok(Ok(Reply{reply_type: ReplyType::ServiceReady, message})) = stream.read_reply() {
        mda_domain = string_tools::get_all_before(&message, " ").to_string();
        stream.send_command(Command::Ehlo(domain.to_string()))?;
    } else {
        warn!("Service is not ready");
        return Err(MTAError::ServiceNotReady);
    }
    
    // Get the Ehlo response
    if let Ok(Ok(Reply{reply_type: ReplyType::Ok, message})) = stream.read_reply() {
        if message.contains("\nSTARTTLS\n") {
            stream.send_command(Command::StartTls)?;

            if let Ok(Ok(Reply{reply_type: ReplyType::ServiceReady, message: _})) = stream.read_reply() {
                if let Stream::Unencryted(old_stream) = stream {
                    let connector = TlsConnector::new()?;
                    match connector.connect(&mda_domain, old_stream) {
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
                            return Err(MTAError::TlsHandshakeError(e));
                        },
                    };
                }
            }
        }
    } else {
        warn!("Service is not ready");
        return Err(MTAError::ServiceNotReady);
    }

    stream.send_command(Command::Mail(from))?;

    if let Ok(Ok(Reply{reply_type: ReplyType::Ok, message: _})) = stream.read_reply() {
        
    } else {
        warn!("Service refused MAIL FROM:");
        return Err(MTAError::ServiceNotReady);
    }

    stream.send_command(Command::Recipient(to))?;
    stream.read_reply()?;
    stream.send_command(Command::Data)?;
    stream.read_reply()?;
    
    let mail = mail.as_bytes();
    let mut requests = 0;
    let mut written = stream.write(mail)?;
    while written < mail.len() && requests < 60 {
        written += stream.write(&mail[written..])?;
        requests += 1;
    }

    stream.read_reply()?;
    stream.send_command(Command::Quit)?;
    
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