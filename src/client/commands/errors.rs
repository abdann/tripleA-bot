#[derive(Debug)]
pub enum MemberNotFoundError {
    MemberNotFound(String),
}

impl std::error::Error for MemberNotFoundError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            MemberNotFoundError::MemberNotFound(ref _reply_string) => None,
        }
    }
}

impl std::fmt::Display for MemberNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MemberNotFoundError::MemberNotFound(ref reply_string) => write!(f, "{}", reply_string),
        }
    }
}
#[derive(Debug)]
pub enum ChannelNotFoundError {
    ChannelNotFound(String),
}

impl std::error::Error for ChannelNotFoundError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ChannelNotFoundError::ChannelNotFound(ref _reply_string) => None,
        }
    }
}

impl std::fmt::Display for ChannelNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChannelNotFoundError::ChannelNotFound(ref reply_string) => {
                write!(f, "{}", reply_string)
            }
        }
    }
}
