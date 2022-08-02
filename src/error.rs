use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum GenError {
    IoError(IoError),
    JsonError(JsonError),
    YamlError(YamlError),
    WrongFileExtension(Option<String>),
}

impl From<IoError> for GenError {
    fn from(e: IoError) -> GenError {
        GenError::IoError(e)
    }
}

impl From<JsonError> for GenError {
    fn from(e: JsonError) -> GenError {
        GenError::JsonError(e)
    }
}

impl From<YamlError> for GenError {
    fn from(e: YamlError) -> GenError {
        GenError::YamlError(e)
    }
}

impl fmt::Display for GenError {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> Result<(), fmt::Error> {
        match self {
            GenError::IoError(e) => write!(f, "Io error occurred: {}", e),
            GenError::JsonError(e) => write!(f, "Json deserialization error occurred: {}", e),
            GenError::YamlError(e) => write!(f, "Yaml deserialization error occurred: {}", e),
            GenError::WrongFileExtension(o) => {
                if let Some(s) = o {
                    write!(f, "Bad file type: {}", s)
                } else {
                    write!(f, "Bad file type")
                }
            }
        }
    }
}

impl std::error::Error for GenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GenError::IoError(e) => Some(e),
            GenError::JsonError(e) => Some(e),
            GenError::YamlError(e) => Some(e),
            GenError::WrongFileExtension(_) => None,
        }
    }
}
