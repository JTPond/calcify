use std::error;
use std::fs;
use std::io::BufWriter;
use std::io::prelude::*;
use std::marker::Sized;
use std::path::Path;

use super::errors::CalcifyError;
use super::serializable::{Deserializable, Serializable};

/// Trait for saving a *Tree to disk.
pub trait ToFile {
    /// Write Self as json to file.
    fn write_json(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn error::Error>>
    where
        Self: Serializable,
    {
        let f = fs::File::create(path)?;
        let mut wr = BufWriter::new(f);
        wr.write_all(self.to_json().as_bytes())?;
        wr.flush()?;
        Ok(())
    }

    /// Write Self as msg to file.
    fn write_msg(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn error::Error>>
    where
        Self: Serializable,
    {
        let f = fs::File::create(path)?;
        let mut wr = BufWriter::new(f);
        wr.write_all(self.to_msg()?.as_slice())?;
        wr.flush()?;
        Ok(())
    }
}

/// Trait for loading a *Tree from disk.
pub trait FromFile {
    /// Read json file to Self.
    fn read_json(path: impl AsRef<Path>) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Deserializable + Sized,
    {
        Self::from_json(&fs::read_to_string(path)?)
    }

    /// Read msg file to Self.
    fn read_msg(path: impl AsRef<Path>) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Deserializable + Sized,
    {
        let bytes = fs::read(path)?;
        let (obj, rest) = Self::from_msg(&bytes)?;
        if !rest.is_empty() {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "trailing bytes after MessagePack object",
            ))));
        }
        Ok(obj)
    }
}
