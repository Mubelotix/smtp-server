#[derive(Debug)]
pub enum Reply {
    SystemStatus(String),
    HelpMessage(String),
    ServiceReady(String),
    ServiceClosingTransmissionChannel(String),
    Ok(String),
    UserNotLocalHandled(String),
    UserNotLocal(String),
    CannotVerifyUser(String),
    StartMailInput(String),
    ServiceUnavailable(String),
    MailActionNotTaken(String),
    ActionNotTaken(String),
    MailActionAborted(String),
    ActionAborted(String),
    InsufficientStorage(String),
    UnableToAccomodateParameters(String),
    SyntaxError(String),
    SyntaxErrorInParametersOrArguments(String),
    CommandNotImplemented(String),
    BadSequenceOfCommands(String),
    CommandParameterNotImplemented(String),
    MailboxNameError(String),
    TransactionFailed(String),
}

impl ToString for Reply {
    fn to_string(&self) -> String {
        let (code, message) = match self {
            Reply::SystemStatus(message) => (211, message),
            Reply::HelpMessage(message) => (214, message),
            Reply::ServiceReady(message) => (220, message),
            Reply::ServiceClosingTransmissionChannel(message) => (221, message),
            Reply::Ok(message) => (250, message),
            Reply::UserNotLocalHandled(message) => (251, message),
            Reply::CannotVerifyUser(message) => (252, message),
            Reply::StartMailInput(message) => (354, message),
            Reply::ServiceUnavailable(message) => (421, message),
            Reply::MailActionNotTaken(message) => (450, message),
            Reply::ActionAborted(message) => (451, message),
            Reply::InsufficientStorage(message) => (452, message),
            Reply::UnableToAccomodateParameters(message) => (455, message),
            Reply::SyntaxError(message) => (500, message),
            Reply::SyntaxErrorInParametersOrArguments(message) => (501, message),
            Reply::CommandNotImplemented(message) => (502, message),
            Reply::BadSequenceOfCommands(message) => (503, message),
            Reply::CommandParameterNotImplemented(message) => (504, message),
            Reply::ActionNotTaken(message) => (550, message),
            Reply::UserNotLocal(message) => (551, message),
            Reply::MailActionAborted(message) => (552, message),
            Reply::MailboxNameError(message) => (553, message),
            Reply::TransactionFailed(message) => (554, message),
        };
        process_message(code, &message)
    }
}

impl std::str::FromStr for Reply {
    type Err = &'static str;

    fn from_str(message: &str) -> Result<Reply, Self::Err> {
        let code = match message[..3].parse::<usize>() {
            Ok(code) => code,
            Err(_e) => return Err("Missing reply code")
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

        match code {
            211 => Ok(Reply::SystemStatus(parsed_message)),
            214 => Ok(Reply::HelpMessage(parsed_message)),
            220 => Ok(Reply::ServiceReady(parsed_message)),
            221 => Ok(Reply::ServiceClosingTransmissionChannel(parsed_message)),
            250 => Ok(Reply::Ok(parsed_message)),
            251 => Ok(Reply::UserNotLocalHandled(parsed_message)),
            252 => Ok(Reply::CannotVerifyUser(parsed_message)),
            354 => Ok(Reply::StartMailInput(parsed_message)),
            421 => Ok(Reply::ServiceUnavailable(parsed_message)),
            450 => Ok(Reply::MailActionNotTaken(parsed_message)),
            451 => Ok(Reply::ActionAborted(parsed_message)),
            452 => Ok(Reply::InsufficientStorage(parsed_message)),
            455 => Ok(Reply::UnableToAccomodateParameters(parsed_message)),
            500 => Ok(Reply::SyntaxError(parsed_message)),
            501 => Ok(Reply::SyntaxErrorInParametersOrArguments(parsed_message)),
            502 => Ok(Reply::CommandNotImplemented(parsed_message)),
            503 => Ok(Reply::BadSequenceOfCommands(parsed_message)),
            504 => Ok(Reply::CommandParameterNotImplemented(parsed_message)),
            550 => Ok(Reply::ActionNotTaken(parsed_message)),
            551 => Ok(Reply::UserNotLocal(parsed_message)),
            552 => Ok(Reply::MailActionAborted(parsed_message)),
            553 => Ok(Reply::MailboxNameError(parsed_message)),
            554 => Ok(Reply::TransactionFailed(parsed_message)),
            _ => Err("Unknown code"),
        }
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
