#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};
use string_tools::*;

#[derive(Debug)]
pub enum Command {
    Helo(String),
    Ehlo(String),
    Mail(String),
    Reset,
    Recipient(String),
    Verify,
    Expand,
    Help,
    Noop,
    Quit,
    Data,
    StartTls,
    Auth(String),
}

#[derive(Debug)]
pub enum ParsingCommandError {
    UnknownCommand
}

impl std::str::FromStr for Command {
    
    type Err = ParsingCommandError;

    fn from_str(command: &str) -> Result<Command, Self::Err> {
        match command.to_ascii_uppercase() {
            c if c.starts_with("EHLO ") => {
                let c = &command[5..];

                let mut domain = String::new();
                let mut last_was_point = false;

                for character in get_all_before(&c, "\r\n").chars() {
                    if character.is_ascii() {
                        domain.push(character);
                        last_was_point = false;
                    } else if character == '.' && !last_was_point {
                        domain.push(character);
                        last_was_point = true;
                    } else {
                        warn!("Unexpected character while parsing the domain name in the EHLO command: {:?}. Ignoring.", character);
                    }
                }

                Ok(Command::Ehlo(domain))
            },
            c if c.starts_with("HELO ") => {
                let c = &command[5..];

                let mut domain = String::new();
                let mut last_was_point = false;

                for character in get_all_before(&c, "\r\n").chars() {
                    if character.is_ascii_lowercase() || character == '-' {
                        domain.push(character);
                        last_was_point = false;
                    } else if character == '.' && !last_was_point {
                        domain.push(character);
                        last_was_point = true;
                    } else {
                        warn!("Unexpected character while parsing the domain name in the EHLO command: {:?}. Ignoring.", character);
                    }
                }

                Ok(Command::Helo(domain))
            },
            c if c.starts_with("VRFY ") => Ok(Command::Verify),
            c if c.starts_with("EXPN ") => Ok(Command::Expand),
            c if c.starts_with("HELP ") => Ok(Command::Help),
            c if c.starts_with("NOOP ") => Ok(Command::Noop),
            c if c.starts_with("QUIT") => Ok(Command::Quit),
            c if c.starts_with("MAIL FROM:") => {
                let from = get_all_between(command, "<", ">");
                Ok(Command::Mail(from.to_string()))
            },
            c if c.starts_with("RCPT TO:") => {
                let to = get_all_between(command, "<", ">");
                Ok(Command::Recipient(to.to_string()))
            },
            c if c.starts_with("AUTH ") => {
                let data = &command[5..];
                Ok(Command::Auth(data.to_string()))
            },
            c if c.starts_with("DATA") => Ok(Command::Data),
            c if c.starts_with("RSET") => Ok(Command::Reset),
            c if c.starts_with("STARTTLS") => Ok(Command::StartTls),
            _c => Err(ParsingCommandError::UnknownCommand),
        }

        
    }
}