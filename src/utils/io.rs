use std::error;
use std::marker::Sized;

use std::io::prelude::*;
use std::io::BufWriter;
use std::fs;

use super::serializable::{Serializable, Deserializable};
use super::errors::CalcifyError;


pub trait ToFile {
    /// Write Self as json to file.
    fn write_json(&self, filename: &str) -> Result<(), Box<dyn error::Error>>
        where Self: Serializable {
            let f = fs::File::create(filename)?;
            let mut wr = BufWriter::new(f);
            wr.write(self.to_json().as_bytes())?;
            Ok(())
    }

    /// Write Self as msg to file.
    fn write_msg(&self, filename: &str) -> Result<(), Box<dyn error::Error>>
        where Self: Serializable {
            let f = fs::File::create(filename)?;
            let mut wr = BufWriter::new(f);
            wr.write(self.to_msg()?.as_slice())?;
            Ok(())
    }
}

pub trait FromFile {
    /// Read json file to Self.
    fn read_json(filename: &str) -> Result<Self, Box<dyn error::Error>>
        where Self: Deserializable + Sized {
            Self::from_json(&fs::read_to_string(filename)?)
    }

    /// Read msg file to Self.
    fn read_msg(filename: &str) -> Result<Self, Box<dyn error::Error>>
        where Self: Deserializable + Sized {
            if let Ok((obj,_)) = Self::from_msg(&fs::read(filename)?) {
                return Ok(obj);
            } else {
                return Err(Box::new(CalcifyError::ParseError));
            }
    }
}
