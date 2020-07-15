use crate::address::EmailAddress;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use string_tools::*;

#[derive(Debug)]
pub enum Address<'a> {
    Domain(&'a str),
    Ipv4(&'a str),
}

#[derive(Debug)]
pub enum Command<'a> {
    Helo(String),
    Helo2(&'a str),
    Ehlo(String),
    Ehlo2(Address<'a>),
    Mail(EmailAddress),
    Reset,
    Recipient(EmailAddress),
    Verify(EmailAddress),
    Expand(String),
    Help,
    Noop,
    Quit,
    Data,
    StartTls,
    Auth(String),
}

impl<'a> ToString for Command<'a> {
    fn to_string(&self) -> String {
        match self {
            Command::Helo(domain) => format!("HELO {}\r\n", domain),
            Command::Ehlo(domain) => format!("EHLO {}\r\n", domain),
            Command::Mail(adress) => format!("MAIL FROM:<{}>\r\n", adress),
            Command::Recipient(adress) => format!("RCPT TO:<{}>\r\n", adress),
            Command::Data => "DATA\r\n".to_string(),
            Command::Reset => "RSET\r\n".to_string(),
            Command::Verify(user) => format!("VRFY {}\r\n", user),
            Command::Expand(mailing_list) => format!("EXPN {}\r\n", mailing_list),
            Command::Help => "HELP\r\n".to_string(),
            Command::Noop => "NOOP\r\n".to_string(),
            Command::Quit => "QUIT\r\n".to_string(),
            Command::StartTls => "STARTTLS\r\n".to_string(),
            Command::Auth(mechanism) => format!("AUTH {}\r\n", mechanism),
            _ => todo!()
        }
    }
}

#[derive(Debug)]
pub enum ParsingCommandError {
    UnknownCommand,
    SyntaxErrorInParameter(&'static str),
}

impl<'a> std::str::FromStr for Command<'a> {
    type Err = ParsingCommandError;

    fn from_str(mut command: &str) -> Result<Command<'a>, Self::Err> {
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
            }
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
            }
            c if c.starts_with("VRFY ") => {
                command = &command[5..];
                command = command.trim();

                if command.starts_with('<') && command.ends_with('>') {
                    command = &command[1..command.len() - 1];
                }

                let address = match command.parse::<EmailAddress>() {
                    Ok(address) => address,
                    Err(e) => return Err(ParsingCommandError::SyntaxErrorInParameter(e)),
                };
                Ok(Command::Verify(address))
            }
            c if c.starts_with("EXPN ") => Ok(Command::Expand(String::new())),
            c if c.starts_with("HELP ") => Ok(Command::Help),
            c if c.starts_with("NOOP ") => Ok(Command::Noop),
            c if c.starts_with("QUIT") => Ok(Command::Quit),
            c if c.starts_with("MAIL FROM:") => {
                command = &command[10..];
                command = command.trim();

                command = string_tools::get_all_between(command, "<", ">");

                let address = match command.parse::<EmailAddress>() {
                    Ok(address) => address,
                    Err(e) => return Err(ParsingCommandError::SyntaxErrorInParameter(e)),
                };

                Ok(Command::Mail(address))
            }
            c if c.starts_with("RCPT TO:") => {
                command = &command[8..];
                command = command.trim();

                if command.starts_with('<') && command.ends_with('>') {
                    command = &command[1..command.len() - 1];
                }

                let address = match command.parse::<EmailAddress>() {
                    Ok(address) => address,
                    Err(e) => return Err(ParsingCommandError::SyntaxErrorInParameter(e)),
                };

                Ok(Command::Recipient(address))
            }
            c if c.starts_with("AUTH ") => {
                let data = &command[5..];
                Ok(Command::Auth(data.to_string()))
            }
            c if c.starts_with("DATA") => Ok(Command::Data),
            c if c.starts_with("RSET") => Ok(Command::Reset),
            c if c.starts_with("STARTTLS") => Ok(Command::StartTls),
            _c => Err(ParsingCommandError::UnknownCommand),
        }
    }
}

mod parsing {
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::tag_no_case,
        bytes::complete::{tag, take_while},
        sequence::tuple,
        error::{ErrorKind, ParseError},
    };
    use super::*;
    use std::cell::Cell;

    #[derive(Debug)]
    pub enum Error {
        InvalidDomain,
        InvalidIpv4Address,
        Nom(ErrorKind)
    }

    impl ParseError<&str> for Error {
        fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
            Error::Nom(kind)
        }

        fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
            other
        }
    }

    fn domain(input: &str) -> IResult<&str, &str, Error> {
        let point_allowed = Cell::new(false);
        let hyphen_allowed = Cell::new(false);
        let end_allowed = Cell::new(false);
        let r = take_while(|c: char| {
            if c.is_alphanumeric() {
                point_allowed.set(true);
                hyphen_allowed.set(true);
                end_allowed.set(true);
                true
            } else if c == '.' && point_allowed.get() {
                point_allowed.set(false);
                hyphen_allowed.set(false);
                end_allowed.set(false);
                true
            } else if c == '-' && hyphen_allowed.get() {
                point_allowed.set(false);
                end_allowed.set(false);
                true
            } else {
                false
            }
        })(input)?;
        if !end_allowed.get() {
            return Err(nom::Err::Error(Error::InvalidDomain))
        }
        Ok(r)
    }

    fn ipv4_adress(input: &str) -> IResult<&str, &str, Error> {
        let (input, _useless) = tag("[")(input)?;

        let digit_idx = Cell::new(0);
        let allow_three_digits = Cell::new(true);
        let allow_high_second_digit = Cell::new(true);
        let allow_high_third_digit = Cell::new(true);
        let number_idx = Cell::new(0);
        let error = Cell::new(false);
        let (input, addr) = take_while(|c: char| {
            let number_idx2 = digit_idx.get();
            if c.is_ascii_digit() {
                if number_idx2 < 2 || (number_idx2 == 2 && allow_three_digits.get()) {
                    if number_idx2 == 0 {
                        allow_high_third_digit.set(true);
                        if c == '2' {
                            allow_three_digits.set(true);
                            allow_high_second_digit.set(false);
                        } else if c != '0' && c != '1' {
                            allow_three_digits.set(false);
                        } else  {
                            allow_three_digits.set(true);
                        }
                    } else if number_idx2 == 1 && !allow_high_second_digit.get() {
                        if c == '6' || c == '7' || c == '8' || c == '9' {
                            allow_three_digits.set(false);
                        } else if c == '5' {
                            allow_high_third_digit.set(false);
                        }
                    } else if number_idx2 == 2 && !allow_high_third_digit.get() {
                        if c == '6' || c == '7' || c == '8' || c == '9' {
                            error.set(true);
                            return false;
                        }
                    }
                    digit_idx.set(number_idx2 + 1);
                    true
                } else {
                    error.set(true);
                    false
                }
            } else if c == '.' && number_idx2 > 0 {
                let number_idx2 = number_idx.get();
                if number_idx2 < 3 {
                    digit_idx.set(0);
                    number_idx.set(number_idx2 + 1);
                    true
                } else {
                    error.set(true);
                    false
                }
            } else {
                if number_idx.get() < 3 {
                    error.set(true)
                }
                false
            }
        })(input)?;

        if error.get() {
            return Err(nom::Err::Error(Error::InvalidIpv4Address));
        }

        let (input, _useless) = tag("]")(input)?;

        Ok((input, addr))
    }

    fn address(input: &str) -> IResult<&str, Address, Error> {
        todo!();
    }

    fn helo(input: &str) -> IResult<&str, Command, Error> {
        let (remaining, (command, domain, end)) = tuple((tag_no_case("HELO "), domain, tag("\r\n")))(input)?;
        Ok((remaining, Command::Helo2(domain)))
    }

    fn ehlo(input: &str) -> IResult<&str, Command, Error> {
        let (remaining, (command, address, end)) = tuple((tag_no_case("HELO "), address, tag("\r\n")))(input)?;
        Ok((remaining, Command::Ehlo2(address)))
    }

    fn command(input: &str) -> IResult<&str, Command, Error> {
        alt((helo, helo))(input)
    }

    #[test]
    fn name654() {
        println!("{:?}", command("HELO here.we.go\r\n"));
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_domain_name() {
            assert!(domain("mubelotix.dev").is_ok());
            assert!(domain("example.com").is_ok());
            assert!(domain("www.example.com").is_ok());
            assert!(domain("www..example.com").is_err());
            assert!(domain("www.example-.com").is_err());
            assert!(domain("www.example.").is_err());
            assert!(domain(".example.com").is_err());
            assert!(domain("www.-example.com").is_err());
        }

        #[test]
        fn test_ipv4_address() {
            assert!(ipv4_adress("[192.168.1.1]").is_ok());
            assert!(ipv4_adress("[192.168.1.1.255]").is_err());
            assert!(ipv4_adress("[192.168..1]").is_err());
            assert!(ipv4_adress("[192.1681.1.1]").is_err());
            assert!(ipv4_adress("[192.168.1]").is_err());
            assert!(ipv4_adress("[192.168.1.1.1]").is_err());
            assert!(ipv4_adress("[192.368.1.1]").is_err());
            assert!(ipv4_adress("[192.268.1.1]").is_err());
            assert!(ipv4_adress("[192.258.1.1]").is_err());
        }
    }
}