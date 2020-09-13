#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[derive(Debug)]
pub struct Reply {
    pub reply_type: ReplyType,
    pub message: String,
}

impl Reply {
    pub fn Ok() -> Reply {
        Reply {
            reply_type: ReplyType::Ok,
            message: String::from("OK"),
        }
    }

    pub fn SystemStatus() -> Reply {
        Reply {
            reply_type: ReplyType::SystemStatus,
            message: String::from("undefined"),
        }
    }

    pub fn HelpMessage() -> Reply {
        Reply {
            reply_type: ReplyType::HelpMessage,
            message: String::from("undefined"),
        }
    }

    pub fn ServiceReady() -> Reply {
        Reply {
            reply_type: ReplyType::ServiceReady,
            message: String::from("undefined"),
        }
    }

    pub fn ServiceClosingTransmissionChannel() -> Reply {
        Reply {
            reply_type: ReplyType::ServiceClosingTransmissionChannel,
            message: String::from("undefined"),
        }
    }

    pub fn UserNotLocalHandled() -> Reply {
        Reply {
            reply_type: ReplyType::UserNotLocalHandled,
            message: String::from("undefined"),
        }
    }

    pub fn UserNotLocal() -> Reply {
        Reply {
            reply_type: ReplyType::UserNotLocal,
            message: String::from("undefined"),
        }
    }

    pub fn CannotVerifyUser() -> Reply {
        Reply {
            reply_type: ReplyType::CannotVerifyUser,
            message: String::from("undefined"),
        }
    }

    pub fn StartMailInput() -> Reply {
        Reply {
            reply_type: ReplyType::StartMailInput,
            message: String::from("undefined"),
        }
    }

    pub fn ServiceUnavailable() -> Reply {
        Reply {
            reply_type: ReplyType::ServiceUnavailable,
            message: String::from("undefined"),
        }
    }

    pub fn MailActionNotTaken() -> Reply {
        Reply {
            reply_type: ReplyType::MailActionNotTaken,
            message: String::from("undefined"),
        }
    }

    pub fn ActionNotTaken() -> Reply {
        Reply {
            reply_type: ReplyType::ActionNotTaken,
            message: String::from("undefined"),
        }
    }

    pub fn MailActionAborted() -> Reply {
        Reply {
            reply_type: ReplyType::MailActionAborted,
            message: String::from("undefined"),
        }
    }

    pub fn ActionAborted() -> Reply {
        Reply {
            reply_type: ReplyType::ActionAborted,
            message: String::from("undefined"),
        }
    }

    pub fn InsufficientStorage() -> Reply {
        Reply {
            reply_type: ReplyType::InsufficientStorage,
            message: String::from("undefined"),
        }
    }

    pub fn UnableToAccomodateParameters() -> Reply {
        Reply {
            reply_type: ReplyType::UnableToAccomodateParameters,
            message: String::from("undefined"),
        }
    }

    pub fn SyntaxError() -> Reply {
        Reply {
            reply_type: ReplyType::SyntaxError,
            message: String::from("undefined"),
        }
    }

    pub fn SyntaxErrorInParametersOrArguments() -> Reply {
        Reply {
            reply_type: ReplyType::SyntaxErrorInParametersOrArguments,
            message: String::from("undefined"),
        }
    }

    pub fn CommandNotImplemented() -> Reply {
        Reply {
            reply_type: ReplyType::CommandNotImplemented,
            message: String::from("undefined"),
        }
    }

    pub fn BadSequenceOfCommands() -> Reply {
        Reply {
            reply_type: ReplyType::BadSequenceOfCommands,
            message: String::from("undefined"),
        }
    }

    pub fn CommandParameterNotImplemented() -> Reply {
        Reply {
            reply_type: ReplyType::CommandParameterNotImplemented,
            message: String::from("undefined"),
        }
    }

    pub fn MailboxNameError() -> Reply {
        Reply {
            reply_type: ReplyType::MailboxNameError,
            message: String::from("undefined"),
        }
    }

    pub fn TransactionFailed() -> Reply {
        Reply {
            reply_type: ReplyType::TransactionFailed,
            message: String::from("undefined"),
        }
    }

    pub fn with_message(self, message: String) -> Reply {
        Reply { message, ..self }
    }
}

impl From<(usize, String)> for Reply {
    fn from((code, message): (usize, String)) -> Reply {
        Reply {
            reply_type: ReplyType::from(code),
            message,
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
    ActionNotTaken,
    MailActionAborted,
    ActionAborted,
    InsufficientStorage,
    UnableToAccomodateParameters,
    SyntaxError,
    SyntaxErrorInParametersOrArguments,
    CommandNotImplemented,
    BadSequenceOfCommands,
    CommandParameterNotImplemented,
    MailboxNameError,
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
            ReplyType::UnableToAccomodateParameters => 455,
            ReplyType::SyntaxError => 500,
            ReplyType::SyntaxErrorInParametersOrArguments => 501,
            ReplyType::CommandNotImplemented => 502,
            ReplyType::BadSequenceOfCommands => 503,
            ReplyType::CommandParameterNotImplemented => 504,
            ReplyType::ActionNotTaken => 550,
            ReplyType::UserNotLocal => 551,
            ReplyType::MailActionAborted => 552,
            ReplyType::MailboxNameError => 553,
            ReplyType::TransactionFailed => 554,
            ReplyType::Unknown => 600,
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
            455 => ReplyType::UnableToAccomodateParameters,
            500 => ReplyType::SyntaxError,
            501 => ReplyType::SyntaxErrorInParametersOrArguments,
            502 => ReplyType::CommandNotImplemented,
            503 => ReplyType::BadSequenceOfCommands,
            504 => ReplyType::CommandParameterNotImplemented,
            550 => ReplyType::ActionNotTaken,
            551 => ReplyType::UserNotLocal,
            552 => ReplyType::MailActionAborted,
            553 => ReplyType::MailboxNameError,
            554 => ReplyType::TransactionFailed,
            code => {
                warn!("Unknown code \"{}\".", code);
                ReplyType::Unknown
            }
        }
    }
}

impl ToString for Reply {
    fn to_string(&self) -> String {
        process_message(self.reply_type.clone().into(), &self.message)
    }
}

impl std::str::FromStr for Reply {
    type Err = &'static str;

    fn from_str(message: &str) -> Result<Reply, Self::Err> {
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
