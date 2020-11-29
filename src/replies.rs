#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[derive(Debug)]
pub struct Reply<T = String>
where
    T: std::fmt::Display,
{
    pub reply_type: ReplyType,
    pub message: Option<T>,
}

#[allow(non_snake_case)]
impl<T> Reply<T>
where
    T: std::fmt::Display,
{
    pub fn Ok() -> Reply<T> {
        Reply {
            reply_type: ReplyType::Ok,
            message: None,
        }
    }

    pub fn SystemStatus() -> Reply<T> {
        Reply {
            reply_type: ReplyType::SystemStatus,
            message: None,
        }
    }

    pub fn HelpMessage() -> Reply<T> {
        Reply {
            reply_type: ReplyType::HelpMessage,
            message: None,
        }
    }

    pub fn ServiceReady() -> Reply<T> {
        Reply {
            reply_type: ReplyType::ServiceReady,
            message: None,
        }
    }

    pub fn ServiceClosingTransmissionChannel() -> Reply<T> {
        Reply {
            reply_type: ReplyType::ServiceClosingTransmissionChannel,
            message: None,
        }
    }

    pub fn UserNotLocalHandled() -> Reply<T> {
        Reply {
            reply_type: ReplyType::UserNotLocalHandled,
            message: None,
        }
    }

    pub fn UserNotLocal() -> Reply<T> {
        Reply {
            reply_type: ReplyType::UserNotLocal,
            message: None,
        }
    }

    pub fn CannotVerifyUser() -> Reply<T> {
        Reply {
            reply_type: ReplyType::CannotVerifyUser,
            message: None,
        }
    }

    pub fn StartMailInput() -> Reply<T> {
        Reply {
            reply_type: ReplyType::StartMailInput,
            message: None,
        }
    }

    pub fn ServiceUnavailable() -> Reply<T> {
        Reply {
            reply_type: ReplyType::ServiceUnavailable,
            message: None,
        }
    }

    pub fn MailActionNotTaken() -> Reply<T> {
        Reply {
            reply_type: ReplyType::MailActionNotTaken,
            message: None,
        }
    }

    pub fn ActionNotTaken() -> Reply<T> {
        Reply {
            reply_type: ReplyType::ActionNotTaken,
            message: None,
        }
    }

    pub fn MailActionAborted() -> Reply<T> {
        Reply {
            reply_type: ReplyType::MailActionAborted,
            message: None,
        }
    }

    pub fn ActionAborted() -> Reply<T> {
        Reply {
            reply_type: ReplyType::ActionAborted,
            message: None,
        }
    }

    pub fn InsufficientStorage() -> Reply<T> {
        Reply {
            reply_type: ReplyType::InsufficientStorage,
            message: None,
        }
    }

    pub fn UnableToAccomodateParameters() -> Reply<T> {
        Reply {
            reply_type: ReplyType::UnableToAccomodateParameters,
            message: None,
        }
    }

    pub fn SyntaxError() -> Reply<T> {
        Reply {
            reply_type: ReplyType::SyntaxError,
            message: None,
        }
    }

    pub fn SyntaxErrorInParametersOrArguments() -> Reply<T> {
        Reply {
            reply_type: ReplyType::SyntaxErrorInParametersOrArguments,
            message: None,
        }
    }

    pub fn CommandNotImplemented() -> Reply<T> {
        Reply {
            reply_type: ReplyType::CommandNotImplemented,
            message: None,
        }
    }

    pub fn BadSequenceOfCommands() -> Reply<T> {
        Reply {
            reply_type: ReplyType::BadSequenceOfCommands,
            message: None,
        }
    }

    pub fn CommandParameterNotImplemented() -> Reply<T> {
        Reply {
            reply_type: ReplyType::CommandParameterNotImplemented,
            message: None,
        }
    }

    pub fn MailboxNotCorrect() -> Reply<T> {
        Reply {
            reply_type: ReplyType::MailboxNotCorrect,
            message: None,
        }
    }

    pub fn TransactionFailed() -> Reply<T> {
        Reply {
            reply_type: ReplyType::TransactionFailed,
            message: None,
        }
    }

    pub fn TlsRequired() -> Reply<T> {
        Reply {
            reply_type: ReplyType::TlsRequired,
            message: None,
        }
    }

    pub fn TlsUnavailable() -> Reply<T> {
        Reply {
            reply_type: ReplyType::TlsUnavailable,
            message: None,
        }
    }

    pub fn with_message(self, message: T) -> Reply<T> {
        Reply {
            message: Some(message),
            ..self
        }
    }
}

impl<T> From<(usize, T)> for Reply<T>
where
    T: std::fmt::Display,
{
    fn from((code, message): (usize, T)) -> Reply<T> {
        Reply {
            reply_type: ReplyType::from(code),
            message: Some(message),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReplyType {
    SystemStatus,
    HelpMessage,
    ServiceReady,
    ServiceClosingTransmissionChannel,
    Ok,
    UserNotLocalHandled,
    UserNotLocal,
    CannotVerifyUser,
    StartMailInput,
    ServiceUnavailable,
    MailActionNotTaken,
    TlsRequired,
    ActionNotTaken,
    MailActionAborted,
    ActionAborted,
    InsufficientStorage,
    TlsUnavailable,
    UnableToAccomodateParameters,
    SyntaxError,
    SyntaxErrorInParametersOrArguments,
    CommandNotImplemented,
    BadSequenceOfCommands,
    CommandParameterNotImplemented,
    MailboxNotCorrect,
    TransactionFailed,
    Unknown,
}

impl Into<usize> for ReplyType {
    fn into(self) -> usize {
        match self {
            ReplyType::SystemStatus => 211,
            ReplyType::HelpMessage => 214,
            ReplyType::ServiceReady => 220,
            ReplyType::ServiceClosingTransmissionChannel => 221,
            ReplyType::Ok => 250,
            ReplyType::UserNotLocalHandled => 251,
            ReplyType::CannotVerifyUser => 252,
            ReplyType::StartMailInput => 354,
            ReplyType::ServiceUnavailable => 421,
            ReplyType::MailActionNotTaken => 450,
            ReplyType::ActionAborted => 451,
            ReplyType::InsufficientStorage => 452,
            ReplyType::TlsUnavailable => 454,
            ReplyType::UnableToAccomodateParameters => 455,
            ReplyType::SyntaxError => 500,
            ReplyType::SyntaxErrorInParametersOrArguments => 501,
            ReplyType::CommandNotImplemented => 502,
            ReplyType::BadSequenceOfCommands => 503,
            ReplyType::CommandParameterNotImplemented => 504,
            ReplyType::TlsRequired => 530,
            ReplyType::ActionNotTaken => 550,
            ReplyType::UserNotLocal => 551,
            ReplyType::MailActionAborted => 552,
            ReplyType::MailboxNotCorrect => 553,
            ReplyType::TransactionFailed => 554,
            ReplyType::Unknown => 500,
        }
    }
}

impl From<usize> for ReplyType {
    fn from(code: usize) -> ReplyType {
        match code {
            211 => ReplyType::SystemStatus,
            214 => ReplyType::HelpMessage,
            220 => ReplyType::ServiceReady,
            221 => ReplyType::ServiceClosingTransmissionChannel,
            250 => ReplyType::Ok,
            251 => ReplyType::UserNotLocalHandled,
            252 => ReplyType::CannotVerifyUser,
            354 => ReplyType::StartMailInput,
            421 => ReplyType::ServiceUnavailable,
            450 => ReplyType::MailActionNotTaken,
            451 => ReplyType::ActionAborted,
            452 => ReplyType::InsufficientStorage,
            454 => ReplyType::TlsUnavailable,
            455 => ReplyType::UnableToAccomodateParameters,
            500 => ReplyType::SyntaxError,
            501 => ReplyType::SyntaxErrorInParametersOrArguments,
            502 => ReplyType::CommandNotImplemented,
            503 => ReplyType::BadSequenceOfCommands,
            504 => ReplyType::CommandParameterNotImplemented,
            530 => ReplyType::TlsRequired,
            550 => ReplyType::ActionNotTaken,
            551 => ReplyType::UserNotLocal,
            552 => ReplyType::MailActionAborted,
            553 => ReplyType::MailboxNotCorrect,
            554 => ReplyType::TransactionFailed,
            code => {
                warn!("Unknown code \"{}\".", code);
                ReplyType::Unknown
            }
        }
    }
}

impl<T> ToString for Reply<T>
where
    T: std::fmt::Display,
{
    fn to_string(&self) -> String {
        match self.message {
            Some(ref m) => process_message(self.reply_type.clone().into(), &m.to_string()),
            None => process_message(self.reply_type.clone().into(), "undefined"),
        }
    }
}

impl std::str::FromStr for Reply<String> {
    type Err = &'static str;

    fn from_str(message: &str) -> Result<Reply<String>, Self::Err> {
        let code = match message[..3].parse::<usize>() {
            Ok(code) => code,
            Err(_e) => return Err("Missing reply code"),
        };

        let mut parsed_message = String::new();
        for mut line in message.split("\r\n") {
            if line.len() >= 3 {
                line = &line[3..];
            }
            if line.starts_with(' ') || line.starts_with('-') {
                line = &line[1..];
            }
            parsed_message.push_str(line);
            parsed_message.push('\n');
        }

        Ok(Reply::from((code, parsed_message)))
    }
}

fn process_message(code: usize, message: &str) -> String {
    let original_lines = message.split('\n');
    let count = original_lines.clone().count();

    let mut message = String::new();

    for (idx, line) in original_lines.enumerate() {
        if idx == count - 1 {
            message.push_str(&format!("{} {}\r\n", code, line));
        } else {
            message.push_str(&format!("{}-{}\r\n", code, line));
        }
    }

    message
}
