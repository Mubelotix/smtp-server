use std::net::{*, ToSocketAddrs};
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;
use crate::{address::EmailAdress, tcp_stream::Stream, commands::Command};

pub fn transfert_mail(to: EmailAdress) -> std::io::Result<()> {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    let response = resolver.mx_lookup(&to.domain).unwrap();

    // TODO select correct server in the iterator
    let mda = response.iter().next().unwrap();
    let mda = resolver.lookup_ip(&mda.exchange().to_string()).unwrap().iter().next().unwrap();

    let mut stream = Stream::Unencryted(TcpStream::connect((mda, 25))?);

    println!("{:?}", stream.read_reply());
    stream.send_command(Command::Ehlo("mubelotix.dev".to_string()))?;
    println!("{:?}", stream.read_reply());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::EmailAdress;

    #[test]
    fn transfert_to_google() {
        transfert_mail(EmailAdress {
            username: String::from("mubelotix"),
            domain: String::from("gmail.com"),
        }).unwrap();
    }
}