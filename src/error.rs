#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error),
    StdIo(std::io::Error),
    PgnParser,
    FileNotFound,
    IllegalMove { fen_str: String, mv: String },
    AmbiguousMove { fen_str: String, mv: String },
    Http,
    Reqwest(reqwest::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Error::Bincode(e) => {
                fmt.write_str(&format!("An error occured during serialization: {}", e))?;
            }
            Error::StdIo(e) => {
                fmt.write_str(&format!(
                    "An error occured while operating on a file: {}",
                    e
                ))?;
            }
            Error::Reqwest(e) => {
                fmt.write_str(&format!("An error occured during network request: {}", e))?;
            }
            Error::PgnParser => {
                fmt.write_str("Reading PGN file failed; Format might be incorrect")?;
            }
            Error::FileNotFound => {
                fmt.write_str("File does not exist")?;
            }
            Error::IllegalMove { fen_str, mv } => {
                fmt.write_str(&format!(
                    "Move '{}' is illegal in position '{}'",
                    mv, fen_str
                ))?;
            }
            Error::AmbiguousMove { fen_str, mv } => {
                fmt.write_str(&format!(
                    "Move '{}' is ambiguous in position '{}'",
                    mv, fen_str
                ))?;
            }
            Error::Http => {
                fmt.write_str("Received an unexpected HTTP return code")?;
            }
        }
        Ok(())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::StdIo(error)
    }
}

impl From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self {
        Error::Bincode(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Reqwest(error)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Bincode(e) => Some(e),
            Error::StdIo(e) => Some(e),
            Error::Reqwest(e) => Some(e),
            _ => None,
        }
    }
}
