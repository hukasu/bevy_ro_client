use std::fmt::Display;

#[derive(Debug)]
pub enum ActorError {
    WrongHeader,
    UnsupportedVersion,
    Io(std::io::Error),
}

impl Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ActorError::Io(io) => format!("An IO error occurred while reading 'act' file. '{io}'"),
            ActorError::UnsupportedVersion => {
                "The 'act' file had an unsupported version.".to_string()
            }
            ActorError::WrongHeader => "File was not an 'act' file.".to_string(),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for ActorError {}

impl From<std::io::Error> for ActorError {
    fn from(value: std::io::Error) -> Self {
        ActorError::Io(value)
    }
}
