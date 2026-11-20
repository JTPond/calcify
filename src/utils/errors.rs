use std::error;
use std::fmt;

/// Enum of built-in Error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcifyError {
    LightSpeedError,
    KeyAlreadyExists(String),
    MissingKey(String),
    InvalidSubtype(String),
    SubtypeMismatch { expected: String, found: String },
    ParseError(String),
    LengthError,
    ObjectBranchDeserializeError,
}

impl fmt::Display for CalcifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CalcifyError::LightSpeedError => {
                write!(f, "Velocity greater than calcify::C_LIGHT.")
            }
            CalcifyError::KeyAlreadyExists(key) => {
                write!(f, "Key '{}' already exists.", key)
            }
            CalcifyError::MissingKey(key) => {
                write!(f, "Key '{}' not found.", key)
            }
            CalcifyError::InvalidSubtype(subtype) => {
                write!(f, "Invalid subtype '{}'.", subtype)
            }
            CalcifyError::SubtypeMismatch { expected, found } => {
                write!(
                    f,
                    "Subtype mismatch: expected '{}', found '{}'.",
                    expected, found
                )
            }
            CalcifyError::ParseError(detail) => {
                write!(f, "Error on parse in Deserializable: {}", detail)
            }
            CalcifyError::LengthError => write!(f, "Invalid slice length."),
            CalcifyError::ObjectBranchDeserializeError => {
                write!(f, "Attempted to deserialize Object Branch.")
            }
        }
    }
}

impl error::Error for CalcifyError {}
