#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::net::{*, ToSocketAddrs};
use native_tls::TlsConnector;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;
use crate::{address::EmailAdress, tcp_stream::Stream, commands::Command, replies::{ReplyType, Reply}};
use std::{net::TcpStream, thread::sleep, time::Duration};

pub fn transfert_mail(to: EmailAdress) -> std::io::Result<()> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let response = resolver.mx_lookup(&to.domain).unwrap();

    // TODO select correct server in the iterator
    let mda_mx = response.iter().next().unwrap();
    let mda_address = mda_mx.exchange().to_string();
    debug!("Connecting to {}.", mda_address);
    let mda = resolver.lookup_ip(&mda_address).unwrap().iter().next().unwrap();

    let mut stream = Stream::Unencryted(TcpStream::connect((mda, 25))?);
    
    if let Ok(Ok(Reply{reply_type: ReplyType::ServiceReady, message})) = stream.read_reply() {
        println!("{}", message);
        stream.send_command(Command::Ehlo("mubelotix.dev".to_string()))?;
    } else {
        warn!("Service is not ready");
        return Err(std::io::Error::from(std::io::ErrorKind::Other));
    }
    
    if let Ok(Ok(Reply{reply_type: ReplyType::Ok, message})) = stream.read_reply() {
        if message.contains("\nSTARTTLS\n") {
            stream.send_command(Command::StartTls)?;

            println!("{:?}", stream.read_reply());

            sleep(Duration::from_secs(1));

            if let Stream::Unencryted(old_stream) = stream {
                let connector = TlsConnector::new().unwrap();
                match connector.connect("mx.google.com", old_stream) {
                    Ok(new_stream) => {
                        println!("ENABLED");
                        stream = Stream::Encrypted(new_stream)
                    },
                    Err(e) => panic!("Failed to enable TLS {}", e),
                };
            }
        }
        stream.send_command(Command::Ehlo("mubelotix.dev".to_string()))?;
    } else {
        warn!("Service is not ready");
        return Err(std::io::Error::from(std::io::ErrorKind::Other));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::EmailAdress;

    #[test]
    fn transfert_to_google() {
        env_logger::try_init();

        transfert_mail(EmailAdress {
            username: String::from("mubelotix"),
            domain: String::from("gmail.com"),
        }).unwrap();
    }
}