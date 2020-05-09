use native_tls::TlsStream;
use std::{thread::sleep, time::Duration, net::TcpStream};
use std::io::{prelude::*, ErrorKind};
use crate::{commands::{Command, ParsingCommandError}, replies::Reply};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

pub enum Stream {
    Encrypted(TlsStream<TcpStream>),
    Unencryted(TcpStream),
}

impl Stream {
    pub fn send_command(&mut self, command: Command) -> std::io::Result<()> {
        let command = command.to_string();
        debug!("\x1B[35m{:?}\x1B[0m", command);

        let mut command = command.as_bytes();
        let mut timeout = 0;
        
        while !command.is_empty() && timeout < 20 {
            let written = self.write(command)?;
            command = &command[written..];
            timeout += 1;
        }

        if timeout == 20 {
            warn!("Infinite loop cancelled");
        }

        Ok(())
    }

    pub fn read_command(&mut self) -> std::io::Result<Result<Command, ParsingCommandError>> {
        let mut command = Vec::new();

        let mut requests = 0;
        while !command.ends_with(&[0x0D, 0x0A]) && requests < 30 {
            let mut t = [0;128];
            let i = self.read(&mut t)?;
            
            if i == 0 {
                sleep(Duration::from_millis(10));
                continue;
            }

            requests += 1;
            command.append(&mut t[..i].to_vec());
        }

        if requests == 30 {
            warn!("Infinite loop cancelled");
        }

        let command = match String::from_utf8(command) {
            Ok(command) => {
                debug!("\x1B[35m{:?}\x1B[0m", command);
                command
            },
            Err(e) => {
                warn!("Server returned invalid utf8. {}", e);
                return Err(std::io::Error::from(ErrorKind::InvalidData));
            },
        };

        Ok(command.parse())
    }

    pub fn send_reply(&mut self, reply: Reply) -> std::io::Result<()> {
        let reply = reply.to_string();
        debug!("\x1B[32m{:?}\x1B[0m", reply);

        let mut reply = reply.as_bytes();
        let mut timeout = 0;
        
        while !reply.is_empty() && timeout < 20 {
            let written = self.write(reply)?;
            reply = &reply[written..];
            timeout += 1;
        }

        if timeout == 20 {
            warn!("Infinite loop cancelled");
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Encrypted(stream) => stream.shutdown(),
            Stream::Unencryted(stream) => stream.shutdown(std::net::Shutdown::Both)
        }
    }

    pub fn is_encrypted(&self) -> bool {
        match self {
            Stream::Encrypted(_) => true,
            Stream::Unencryted(_) => false
        }
    }
}

impl Read for Stream {
    fn read(&mut self, mut buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Encrypted(stream) => stream.read(&mut buffer),
            Stream::Unencryted(stream) => stream.read(&mut buffer)
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Encrypted(stream) => stream.write(&buffer),
            Stream::Unencryted(stream) => stream.write(&buffer)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Encrypted(stream) => stream.flush(),
            Stream::Unencryted(stream) => stream.flush()
        }
    }
}