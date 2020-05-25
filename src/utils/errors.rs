use std::error;
use std::fmt;

/// Enum of built in Error types
#[derive(Debug,Clone)]
pub enum CalcifyError {
    LightSpeedError,
    KeyError,
    ParseError,
    LengthError,
    ObjectBranchDeserializeError,
}

impl fmt::Display for CalcifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CalcifyError::LightSpeedError => write!(f,"Velocity greater than calcify::C_LIGHT."),
            CalcifyError::KeyError => write!(f,"Invalid Key"),
            CalcifyError::ParseError => write!(f,"Error on parse in Deserializable."),
            CalcifyError::LengthError => write!(f,"Invalid slice length"),
            CalcifyError::ObjectBranchDeserializeError => write!(f,"Attempted to deserialize Object Branch."),
        }
    }
}

impl error::Error for CalcifyError {
    fn description(&self) -> &str {
        match *self {
            CalcifyError::LightSpeedError => "Cannot have a velocity greater than calcify::C_LIGHT",
            CalcifyError::KeyError => "Convert HashMap Option behavior to Err on bad keys",
            CalcifyError::ParseError => "Probably a formatting error when the data was serialized, or there is a type mismatch.",
            CalcifyError::LengthError => "Length of slice must match Vector length",
            CalcifyError::ObjectBranchDeserializeError => "Cannot deserialize Object Branch.",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
