use crate::address::EmailAddress;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use string_tools::*;

#[derive(Debug, PartialEq)]
pub enum ServerIdentity<'a> {
    Domain(&'a str),
    Ipv4(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum LocalPart<'a> {
    DotString(&'a str),
    QuotedString(String),
}

#[derive(Debug, PartialEq)]
pub enum SmtpString<'a> {
    Atom(&'a str),
    QuotedString(String),
}

type PATH<'a> = (Vec<&'a str>, (LocalPart<'a>, ServerIdentity<'a>));
type PARAM<'a> = (&'a str, Option<&'a str>);

#[derive(Debug, PartialEq)]
pub enum Recipient<'a> {
    LocalPostmaster,
    Postmaster(&'a str),
    Path(PATH<'a>),
}

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Helo(String),
    Helo2(&'a str),
    Ehlo(String),
    Ehlo2(ServerIdentity<'a>),
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

#[derive(Debug, PartialEq)]
pub enum Command2<'a> {
    Helo(&'a str),
    Ehlo(ServerIdentity<'a>),
    From(Option<PATH<'a>>, Vec<PARAM<'a>>),
    To(Recipient<'a>, Vec<PARAM<'a>>),
    Data,
    Reset,
    Verify(SmtpString<'a>),
    Expand(SmtpString<'a>),
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
            _ => todo!(),
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

#[allow(dead_code)]
mod parsing {
    use super::Command2 as Command;
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::tag_no_case,
        bytes::complete::{tag, take_while, take_while1},
        error::{ErrorKind, ParseError},
        sequence::tuple,
        Err::Error as NomError,
        IResult,
    };
    use std::cell::Cell;

    #[derive(Debug)]
    pub enum Error<'a> {
        InvalidCommand,
        InvalidDomain,
        InvalidIpv4Address,
        ExpectedEndOfInput,
        InvalidIdentity,
        ExpectedCrlf,
        CommandName,
        Unknown,
        Known(&'a str),
    }

    fn is_atext(character: char) -> bool {
        (character as u8 >= 0x41 && character as u8 <= 0x5A)
            || (character as u8 >= 0x61 && character as u8 <= 0x7A)
            || (character as u8 >= 0x30 && character as u8 <= 0x39)
            || character == '!'
            || character == '#'
            || character == '$'
            || character == '%'
            || character == '&'
            || character == '\''
            || character == '*'
            || character == '+'
            || character == '-'
            || character == '/'
            || character == '='
            || character == '?'
            || character == '^'
            || character == '_'
            || character == '`'
            || character == '{'
            || character == '|'
            || character == '}'
            || character == '~'
    }

    fn is_qtext_smtp(character: char) -> bool {
        (character as u8 >= 32 && character as u8 <= 33)
            || (character as u8 >= 35 && character as u8 <= 91)
            || (character as u8 >= 93 && character as u8 <= 126)
    }

    fn dot_string(input: &str) -> Result<(&str, &str), Error> {
        let mut chars = input.chars();
        let mut idx = 0;
        let mut expects_text = true;

        while let Some(character) = chars.next() {
            if !is_atext(character) {
                if expects_text {
                    return Err(Error::Known("Invalid character in the local part of a mailbox at the first position or after a dot."));
                } else if character == '.' {
                    expects_text = true;
                } else {
                    break;
                }
            } else if expects_text {
                expects_text = false;
            }
            idx += 1;
        }

        Ok((&input[idx..], &input[..idx]))
    }

    fn quoted_string(mut input: &str) -> Result<(&str, String), Error> {
        input = tag::<_, _, ()>("\"")(input)
            .map_err(|_| {
                Error::Known("Expected double quote at the beginning of a quoted string.")
            })?
            .0;
        let mut chars = input.chars();
        let mut string = String::new();

        while let Some(character) = chars.next() {
            if is_qtext_smtp(character) {
                string.push(character);
            } else if character == '\\' {
                match chars.next() {
                    Some(character) if character as u8 >= 32 && character as u8 <= 126 => {
                        string.push(character);
                    }
                    Some(_character) => {
                        return Err(Error::Known(
                            "Invalid backslashed character in a quoted string.",
                        ));
                    }
                    None => {
                        return Err(Error::Known(
                            "Incomplete quoted string. Expected a character after backslash.",
                        ))
                    }
                }
            } else if character == '"' {
                return Ok((chars.as_str(), string));
            } else {
                return Err(Error::Known("Invalid character in a quoted string."));
            }
        }

        return Err(Error::Known(
            "Incomplete quoted string. Expected closing double quote.",
        ));
    }

    fn local_part(input: &str) -> Result<(&str, LocalPart), Error> {
        if let Ok((i, s)) = dot_string(input) {
            return Ok((i, LocalPart::DotString(s)));
        } else if let Ok((i, s)) = quoted_string(input) {
            return Ok((i, LocalPart::QuotedString(s)));
        } else {
            return Err(Error::Known(
                "Invalid local part (invalid dot_string AND invalid quoted_string)",
            ));
        }
    }

    fn mailbox(input: &str) -> Result<(&str, (LocalPart, ServerIdentity)), Error> {
        let (mut input, local_part) = local_part(input)?;
        input = tag::<_, _, ()>("@")(input)
            .map_err(|_| Error::Known("Expecting a '@' in an email address."))?
            .0;
        let (input, identity) = identity(input)?;
        Ok((input, (local_part, identity)))
    }

    fn domain(input: &str) -> Result<(&str, &str), Error> {
        let point_allowed = Cell::new(false);
        let hyphen_allowed = Cell::new(false);
        let end_allowed = Cell::new(false);
        let r = take_while::<_, _, ()>(|c: char| {
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
        })(input)
        .map_err(|_| Error::Unknown)?;
        if !end_allowed.get() {
            return Err(Error::Unknown);
        }
        Ok(r)
    }

    fn ipv4_address(input: &str) -> Result<(&str, &str), Error> {
        let (input, _useless) = tag::<_, _, ()>("[")(input).map_err(|_| Error::Unknown)?;

        let digit_idx = Cell::new(0);
        let allow_three_digits = Cell::new(true);
        let allow_high_second_digit = Cell::new(true);
        let allow_high_third_digit = Cell::new(true);
        let number_idx = Cell::new(0);
        let error = Cell::new(false);
        let (input, addr) = take_while::<_, _, ()>(|c: char| {
            let number_idx2 = digit_idx.get();
            if c.is_ascii_digit() {
                if number_idx2 < 2 || (number_idx2 == 2 && allow_three_digits.get()) {
                    match number_idx2 {
                        0 => {
                            allow_high_third_digit.set(true);
                            allow_three_digits.set(true);
                            allow_high_second_digit.set(true);

                            match c {
                                '2' => allow_high_second_digit.set(false),
                                '0' | '1' => (),
                                _ => allow_three_digits.set(false),
                            }
                        }
                        1 if !allow_high_second_digit.get() => match c {
                            '6' | '7' | '8' | '9' => allow_three_digits.set(false),
                            '5' => allow_high_third_digit.set(false),
                            _ => (),
                        },
                        2 if !allow_high_third_digit.get() => {
                            if c == '6' || c == '7' || c == '8' || c == '9' {
                                error.set(true);
                                return false;
                            }
                        }
                        _ => (),
                    };
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
        })(input)
        .map_err(|_| Error::Unknown)?;

        if error.get() {
            return Err(Error::InvalidIpv4Address);
        }

        let (input, _useless) = tag::<_, _, ()>("]")(input).map_err(|_| Error::Unknown)?;

        Ok((input, addr))
    }

    fn identity(input: &str) -> Result<(&str, ServerIdentity), Error> {
        if let Ok((input, addr)) = ipv4_address(input) {
            return Ok((input, ServerIdentity::Ipv4(addr)));
        } else if let Ok((input, domain)) = domain(input) {
            return Ok((input, ServerIdentity::Domain(domain)));
        } else {
            return Err(Error::InvalidIdentity);
        }
    }

    fn reverse_path(input: &str) -> Result<(&str, Option<PATH>), Error> {
        if let Ok((i, _p)) = tag::<_, _, ()>("<>")(input) {
            return Ok((i, None));
        }

        let (input, path) = path(input)?;
        Ok((input, Some(path)))
    }

    fn source_route(mut input: &str) -> Result<(&str, Vec<&str>), Error> {
        input = tag::<_, _, ()>("@")(input)
            .map_err(|_| Error::Known("Expected '@' at the beginning of a source route."))?
            .0;

        let (mut input, first_domain) = domain(input)?;
        let mut domains = Vec::new();
        domains.push(first_domain);

        while !input.is_empty() {
            match tag::<_, _, ()>(",")(input) {
                Ok((r, _comma)) => input = r,
                _ => break,
            }

            input = tag::<_, _, ()>("@")(input)
                .map_err(|_| Error::Known("Expected '@' after ',' in a source route."))?
                .0;

            let new_domain = domain(input)?;
            input = new_domain.0;
            domains.push(new_domain.1);
        }

        input = tag::<_, _, ()>(":")(input)
            .map_err(|_| Error::Known("Expected ':' at the end of a source route."))?
            .0;

        Ok((input, domains))
    }

    fn path(input: &str) -> Result<(&str, PATH), Error> {
        let (mut input, _begin) = tag::<_, _, ()>("<")(input)
            .map_err(|_| Error::Known("Expected '<' at the beginning of a path."))?;
        let source_route = match source_route(input) {
            Ok((i, sr)) => {
                input = i;
                sr
            }
            _ => Vec::new(),
        };
        let (input, mailbox) = mailbox(input)?;
        let (input, _end) = tag::<_, _, ()>(">")(input)
            .map_err(|_| Error::Known("Expected '>' at the end of a path."))?;

        Ok((input, (source_route, mailbox)))
    }

    fn helo(input: &str) -> Result<Command, Error> {
        let (input, _command_name) =
            tag_no_case::<_, _, ()>("HELO ")(input).map_err(|_| Error::CommandName)?;
        let (input, domain) = domain(input)?;
        let (input, _end) = tag::<_, _, ()>("\r\n")(input).map_err(|_| Error::ExpectedCrlf)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }
        Ok(Command::Helo(domain))
    }

    fn ehlo(input: &str) -> Result<Command, Error> {
        let (input, _command_name) =
            tag_no_case::<_, _, ()>("EHLO ")(input).map_err(|_| Error::CommandName)?;
        let (input, identity) = identity(input)?;
        let (input, _end) = tag::<_, _, ()>("\r\n")(input).map_err(|_| Error::ExpectedCrlf)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }
        Ok(Command::Ehlo(identity))
    }

    fn parameters(input: &str) -> Result<(&str, Vec<PARAM>), Error> {
        let mut parameters = Vec::new();
        let (mut input, first_param) = esmtp_param(input)?;
        parameters.push(first_param);

        while !input.is_empty() {
            let current_input;
            if let Ok((i, _)) = tag::<_, _, ()>(" ")(input) {
                current_input = i;
            } else {
                break;
            }

            if let Ok((i, param)) = esmtp_param(current_input) {
                parameters.push(param);
                input = i;
            } else {
                break;
            }
        }

        Ok((input, parameters))
    }

    fn esmtp_param(input: &str) -> Result<(&str, PARAM), Error> {
        let (mut input, keyword) = esmtp_keyword(input)?;
        match tag::<_, _, ()>("=")(input) {
            Ok((i, _)) => input = i,
            _ => return Ok((input, (keyword, None))),
        }
        let (input, value) = esmtp_value(input)?;
        Ok((input, (keyword, Some(value))))
    }

    fn esmtp_keyword(input: &str) -> Result<(&str, &str), Error> {
        let (input, keyword) = take_while1::<_, _, ()>(|character: char| {
            (character as u8 >= 0x41 && character as u8 <= 0x5A)
                || (character as u8 >= 0x61 && character as u8 <= 0x7A)
                || (character as u8 >= 0x30 && character as u8 <= 0x39)
                || character == '-'
        })(input)
        .map_err(|_| Error::Known("Empty esmtp_keyword"))?;

        if keyword.starts_with('-') {
            return Err(Error::Known("esmtp_keyword cannot start with a '\\\''"));
        }

        Ok((input, keyword))
    }

    fn esmtp_value(input: &str) -> Result<(&str, &str), Error> {
        Ok(take_while1::<_, _, ()>(|character: char| {
            character as u8 >= 33 && character as u8 <= 128 && character as u8 != 61
        })(input)
        .map_err(|_| Error::Known("Empty esmtp_value"))?)
    }

    fn string(input: &str) -> Result<(&str, SmtpString), Error> {
        if let Ok((input, s)) = take_while1::<_,_,()>(is_atext)(input) {
            return Ok((input, SmtpString::Atom(s)))
        }

        if let Ok((input, s)) = quoted_string(input) {
            return Ok((input, SmtpString::QuotedString(s)))
        }

        Err(Error::Known("Expected a string."))
    }

    fn recipient(input: &str) -> Result<(&str, Recipient), Error> {
        if let Ok((input, _)) = tag_no_case::<_, _, ()>("<postmaster@")(input) {
            if let Ok((input, domain)) = domain(input) {
                if let Ok((input, _)) = tag::<_, _, ()>(">")(input) {
                    return Ok((input, Recipient::Postmaster(domain)));
                }
            }
        }

        if let Ok((input, path)) = path(input) {
            return Ok((input, Recipient::Path(path)));
        }

        if let Ok((input, _)) = tag_no_case::<_, _, ()>("<postmaster>")(input) {
            return Ok((input, Recipient::LocalPostmaster));
        }

        Err(Error::Known("The recipient does not match anything."))
    }

    fn to(input: &str) -> Result<Command, Error> {
        let (input, _command_name) =
            tag_no_case::<_, _, ()>("RCPT TO:")(input).map_err(|_| Error::CommandName)?;
        let (mut input, recipient) = recipient(input)?;

        let mail_parameters;
        if let Ok((i, _)) = tag::<_, _, ()>(" ")(input) {
            let (i, p) = parameters(i)?;
            input = i;
            mail_parameters = p;
        } else {
            mail_parameters = Vec::new();
        }

        let (input, _end) = tag::<_, _, ()>("\r\n")(input).map_err(|_| Error::ExpectedCrlf)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }
        Ok(Command::To(recipient, mail_parameters))
    }

    fn from(input: &str) -> Result<Command, Error> {
        let (input, _command_name) =
            tag_no_case::<_, _, ()>("MAIL FROM:")(input).map_err(|_| Error::CommandName)?;
        let (mut input, path) = reverse_path(input)?;

        let mail_parameters;
        if let Ok((i, _)) = tag::<_, _, ()>(" ")(input) {
            let (i, p) = parameters(i)?;
            input = i;
            mail_parameters = p;
        } else {
            mail_parameters = Vec::new();
        }

        let (input, _end) = tag::<_, _, ()>("\r\n")(input).map_err(|_| Error::ExpectedCrlf)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }
        Ok(Command::From(path, mail_parameters))
    }

    fn data(input: &str) -> Result<Command, Error> {
        let (input, _) = tag_no_case::<_,_,()>("DATA\r\n")(input).map_err(|_| Error::CommandName)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }

        Ok(Command::Data)
    }

    fn reset(input: &str) -> Result<Command, Error> {
        let (input, _) = tag_no_case::<_,_,()>("RSET\r\n")(input).map_err(|_| Error::CommandName)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }

        Ok(Command::Reset)
    }

    fn verify(input: &str) -> Result<Command, Error> {
        let (input, _) = tag_no_case::<_,_,()>("VRFY ")(input).map_err(|_| Error::CommandName)?;
        let (input, string) = string(input)?;
        let (input, _end) = tag::<_, _, ()>("\r\n")(input).map_err(|_| Error::ExpectedCrlf)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }
        Ok(Command::Verify(string))
    }

    fn expand(input: &str) -> Result<Command, Error> {
        let (input, _) = tag_no_case::<_,_,()>("EXPN ")(input).map_err(|_| Error::CommandName)?;
        let (input, string) = string(input)?;
        let (input, _end) = tag::<_, _, ()>("\r\n")(input).map_err(|_| Error::ExpectedCrlf)?;
        if !input.is_empty() {
            return Err(Error::ExpectedEndOfInput);
        }
        Ok(Command::Expand(string))
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_helo() {
            assert_eq!(
                helo("HELO google.com\r\n").unwrap(),
                Command::Helo("google.com")
            );
            assert!(helo("HELO google.com\r\n invalid ").is_err());
        }

        #[test]
        fn test_data_and_reset() {
            assert_eq!(
                data("DATA\r\n").unwrap(),
                Command::Data
            );
            assert_eq!(
                reset("RSET\r\n").unwrap(),
                Command::Reset
            );
            assert!(data("DATA\r\n email data").is_err());
        }

        #[test]
        fn test_verify_and_expand() {
            assert_eq!(
                verify("VRFY mubelotix\r\n").unwrap(),
                Command::Verify(SmtpString::Atom("mubelotix"))
            );
            assert_eq!(
                verify("VRFY \"mubelotix\\@gmail\\.com\"\r\n").unwrap(),
                Command::Verify(SmtpString::QuotedString("mubelotix@gmail.com".to_string()))
            );
            assert!(verify("VRFY \"mubelotix\\@gmail\\.com\r\n").is_err());

            assert_eq!(
                expand("EXPN rustaceans\r\n").unwrap(),
                Command::Expand(SmtpString::Atom("rustaceans"))
            );
            assert_eq!(
                expand("EXPN \"Rust\\ lovers\"\r\n").unwrap(),
                Command::Expand(SmtpString::QuotedString("Rust lovers".to_string()))
            );
            assert!(expand("EXPN \"unterminated name\r\n").is_err());
        }

        #[test]
        fn test_from() {
            assert_eq!(
                from("MAIL FROM:<mubelotix@gmail.com>\r\n").unwrap(),
                Command::From(
                    Some((
                        vec![],
                        (
                            LocalPart::DotString("mubelotix"),
                            ServerIdentity::Domain("gmail.com")
                        )
                    )),
                    vec![]
                )
            );

            assert_eq!(
                from("MAIL FROM:<@example.com:mubelotix@gmail.com>\r\n").unwrap(),
                Command::From(
                    Some((
                        vec!["example.com"],
                        (
                            LocalPart::DotString("mubelotix"),
                            ServerIdentity::Domain("gmail.com")
                        )
                    )),
                    vec![]
                )
            );

            assert_eq!(
                from("MAIL FROM:<mubelotix@gmail.com> AUTH=<>\r\n").unwrap(),
                Command::From(
                    Some((
                        vec![],
                        (
                            LocalPart::DotString("mubelotix"),
                            ServerIdentity::Domain("gmail.com")
                        )
                    )),
                    vec![("AUTH", Some("<>"))]
                )
            );

            assert_eq!(
                from("MAIL FROM:<mubelotix@gmail.com> AUTH=<> anonymous\r\n").unwrap(),
                Command::From(
                    Some((
                        vec![],
                        (
                            LocalPart::DotString("mubelotix"),
                            ServerIdentity::Domain("gmail.com")
                        )
                    )),
                    vec![("AUTH", Some("<>")), ("anonymous", None)]
                )
            );
        }

        #[test]
        fn test_to() {
            assert_eq!(
                to("RCPT TO:<@jkl.org:userc@d.bar.org>\r\n").unwrap(),
                Command::To(
                    Recipient::Path((
                        vec!["jkl.org"],
                        (
                            LocalPart::DotString("userc"),
                            ServerIdentity::Domain("d.bar.org")
                        )
                    )),
                    vec![]
                )
            );

            assert_eq!(
                to("RCPT TO:<poStmasTer@gmail.com>\r\n").unwrap(),
                Command::To(
                    Recipient::Postmaster("gmail.com"),
                    vec![]
                )
            );

            assert_eq!(
                to("RCPT TO:<poStMasTer>\r\n").unwrap(),
                Command::To(
                    Recipient::LocalPostmaster,
                    vec![]
                )
            );

            assert_eq!(
                to("RCPT TO:<postmaster> name=value flag\r\n").unwrap(),
                Command::To(
                    Recipient::LocalPostmaster,
                    vec![("name", Some("value")), ("flag", None)]
                )
            );
        }

        #[test]
        fn test_parameters() {
            assert_eq!(
                parameters("AUTH=test").unwrap().1,
                vec![("AUTH", Some("test"))]
            );

            assert_eq!(
                parameters("AUTH=test PARAM-3=value > lorem ipsum dolor sit amet")
                    .unwrap()
                    .1,
                vec![("AUTH", Some("test")), ("PARAM-3", Some("value"))]
            );

            assert_eq!(
                parameters("AUTH=test PARAM-3=value lorem ipsum").unwrap().1,
                vec![
                    ("AUTH", Some("test")),
                    ("PARAM-3", Some("value")),
                    ("lorem", None),
                    ("ipsum", None)
                ]
            );

            assert!(parameters("-invalidname=data").is_err());
        }

        #[test]
        fn test_ehlo() {
            assert_eq!(
                ehlo("EHLO google.com\r\n").unwrap(),
                Command::Ehlo(ServerIdentity::Domain("google.com"))
            );
            assert_eq!(
                ehlo("EHLO [192.168.1.1]\r\n").unwrap(),
                Command::Ehlo(ServerIdentity::Ipv4("192.168.1.1"))
            );
            // todo ipv6
            assert!(ehlo("EHLO google.com\r\n invalid ").is_err());
        }

        #[test]
        fn test_reverse_path() {
            assert_eq!(reverse_path("<>").unwrap().1, None);
            assert_eq!(
                reverse_path("<mubelotix@mubelotix.dev>").unwrap().1,
                Some((
                    vec![],
                    (
                        LocalPart::DotString("mubelotix"),
                        ServerIdentity::Domain("mubelotix.dev")
                    )
                ))
            );
            assert_eq!(
                reverse_path("<@example.com,@gmail.com:mubelotix@mubelotix.dev>")
                    .unwrap()
                    .1,
                Some((
                    vec!["example.com", "gmail.com"],
                    (
                        LocalPart::DotString("mubelotix"),
                        ServerIdentity::Domain("mubelotix.dev")
                    )
                ))
            );
        }

        #[test]
        fn test_mailbox() {
            assert_eq!(
                mailbox("test@example.com").unwrap().1,
                (
                    LocalPart::DotString("test"),
                    ServerIdentity::Domain("example.com")
                )
            );
            assert_eq!(
                mailbox("john.snow@mubelotix.dev").unwrap().1,
                (
                    LocalPart::DotString("john.snow"),
                    ServerIdentity::Domain("mubelotix.dev")
                )
            );
            assert_eq!(
                mailbox("john.snow@[192.168.1.1]").unwrap().1,
                (
                    LocalPart::DotString("john.snow"),
                    ServerIdentity::Ipv4("192.168.1.1")
                )
            );
            assert_eq!(
                mailbox("\"John\\ Snow\"@gmail.com").unwrap().1,
                (
                    LocalPart::QuotedString("John Snow".to_string()),
                    ServerIdentity::Domain("gmail.com")
                )
            );
        }

        #[test]
        fn test_source_route() {
            assert_eq!(
                source_route("@example.com,@google.com,@mubelotix.dev:")
                    .unwrap()
                    .1,
                vec!["example.com", "google.com", "mubelotix.dev"]
            );
            assert_eq!(
                source_route("@example.com:mubelotix@mubelotix.dev")
                    .unwrap()
                    .1,
                vec!["example.com"]
            );
        }

        #[test]
        fn test_strings() {
            assert_eq!(dot_string("mubelotix").unwrap().1, "mubelotix");
            assert_eq!(
                dot_string("mubelotix@mubelotix.dev").unwrap().1,
                "mubelotix"
            );
            assert_eq!(dot_string("john.snow@example.com").unwrap().1, "john.snow");
            assert!(dot_string("john..snow@example.com").is_err());

            assert_eq!(quoted_string(r#""John\ Snow""#).unwrap().1, "John Snow");
            assert_eq!(
                quoted_string(r#""This\,\ is\ a\ \(valid\)\ email\ address\.""#)
                    .unwrap()
                    .1,
                "This, is a (valid) email address."
            );

            assert_eq!(string("mubelotix").unwrap().1, SmtpString::Atom("mubelotix"));
            assert_eq!(string(r#""John\ Snow""#).unwrap().1, SmtpString::QuotedString("John Snow".to_string()));
            assert!(string(r#"école"#).is_err());
        }

        #[test]
        fn test_domain_name() {
            assert_eq!(domain("mubelotix.dev").unwrap().1, "mubelotix.dev");
            assert_eq!(domain("mubelotix.dev:").unwrap().1, "mubelotix.dev");
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
            assert!(ipv4_address("[192.168.1.1]").is_ok());
            assert!(ipv4_address("[192.168.1.1.255]").is_err());
            assert!(ipv4_address("[192.168..1]").is_err());
            assert!(ipv4_address("[192.1681.1.1]").is_err());
            assert!(ipv4_address("[192.168.1]").is_err());
            assert!(ipv4_address("[192.168.1.1.1]").is_err());
            assert!(ipv4_address("[192.368.1.1]").is_err());
            assert!(ipv4_address("[192.268.1.1]").is_err());
            assert!(ipv4_address("[192.258.1.1]").is_err());
        }
    }
}
