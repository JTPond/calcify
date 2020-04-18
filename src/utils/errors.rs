use std::error;
use std::fmt;

/// Cannot have a velocity greater than C_LIGHT
#[derive(Debug,Clone)]
pub enum CalcifyError {
    LightSpeed,
    KeyError,
}

impl fmt::Display for CalcifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CalcifyError::LightSpeed => write!(f,"Velocity greater than calcify::C_LIGHT."),
            CalcifyError::KeyError => write!(f,"Invalid Key"),
        }
    }
}

impl error::Error for CalcifyError {
    fn description(&self) -> &str {
        match *self {
            CalcifyError::LightSpeed => "Cannot have a velocity greater than calcify::C_LIGHT",
            CalcifyError::KeyError => "Convert HashMap Option behavior to Err on bad keys",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}