#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use crate::commands::*;
use trust_dns_resolver::{config::{ResolverConfig, ResolverOpts}, Resolver};

pub enum MTAError {
    IOError(std::io::Error),
    DnsError(trust_dns_resolver::error::ResolveError),
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

pub fn transfert_mail(
    recipient: &[LocalPart],
    recipient_server: ServerIdentity,
    from: Option<Path>,
    mail: &str,
    domain: &str,
) -> Result<(), MTAError> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())?;
    let mda_address = match recipient_server {
        ServerIdentity::Domain(domain) => {
            let response = resolver.mx_lookup(domain)?;
            // TODO select correct server in the iterator

            // Get the domain from a MX record
            let mut mda_domain = match response.iter().next() {
                Some(record) => record.exchange().to_string(),
                None => {
                    warn!("No MX record found for {}", domain);
                    return Err(MTAError::NoMxRecord);
                }
            };

            // Get the ip address from the domain
            match resolver.lookup_ip(&mda_domain)?.iter().next() {
                Some(addr) => addr,
                None => {
                    warn!(
                        "Domain {} referenced by MX record has no associated ip address",
                        &mda_domain
                    );
                    return Err(MTAError::DeadMxRecord);
                }
            }
        },
        ServerIdentity::Ipv4(ip) => {
            ip.parse().unwrap()
        }
    };
    
    /*debug!("Connecting to {}.", "[mda_domain]");
    let mut stream = Stream::Unencryted(TcpStream::connect((mda_address, 25))?);

    // Get the init message and send Ehlo
    if let Ok(Ok(Reply {
        reply_type: ReplyType::ServiceReady,
        message,
    })) = stream.read_reply()
    {
        mda_domain = string_tools::get_all_before(&message, " ").to_string();
        stream.send_command(Command::Ehlo(domain.to_string()))?;
    } else {
        warn!("Service is not ready");
        return Err(MTAError::ServiceNotReady);
    }

    // Get the Ehlo response
    if let Ok(Ok(Reply {
        reply_type: ReplyType::Ok,
        message,
    })) = stream.read_reply()
    {
        if message.contains("\nSTARTTLS\n") {
            stream.send_command(Command::StartTls)?;

            if let Ok(Ok(Reply {
                reply_type: ReplyType::ServiceReady,
                message: _,
            })) = stream.read_reply()
            {
                if let Stream::Unencryted(old_stream) = stream {
                    let connector = TlsConnector::new()?;
                    match connector.connect(&mda_domain, old_stream) {
                        Ok(new_stream) => {
                            debug!("Tls enabled");
                            stream = Stream::Encrypted(new_stream);
                            stream.send_command(Command::Ehlo(domain.to_string()))?;

                            if let Ok(Ok(Reply {
                                reply_type: ReplyType::Ok,
                                message,
                            })) = stream.read_reply()
                            {
                                println!("{:?}", message);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to enable TLS {}", e);
                            return Err(MTAError::TlsHandshakeError(e));
                        }
                    };
                }
            }
        }
    } else {
        warn!("Service is not ready");
        return Err(MTAError::ServiceNotReady);
    }

    stream.send_command(Command::Mail(from.clone()))?;

    if let Ok(Ok(Reply {
        reply_type: ReplyType::Ok,
        message: _,
    })) = stream.read_reply()
    {
    } else {
        warn!("Service refused MAIL FROM:");
        return Err(MTAError::ServiceNotReady);
    }

    stream.send_command(Command::Recipient(to.clone()))?;
    stream.read_reply()?;
    stream.send_command(Command::Data)?;
    stream.read_reply()?;

    let mail = mail.as_string();
    let mail = mail.as_bytes();
    let mut requests = 0;
    let mut written = stream.write(mail)?;
    while written < mail.len() && requests < 60 {
        written += stream.write(&mail[written..])?;
        requests += 1;
    }

    stream.read_reply()?;
    stream.send_command(Command::Quit)?;*/

    unimplemented!();

    Ok(())
}