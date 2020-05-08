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