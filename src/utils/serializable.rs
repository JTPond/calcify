use std::marker::Sized;
use std::error;

extern crate rmp;
use rmp::encode::*;
use rmp::decode::*;
/// Serialization trait which all types you intend to put in a Tree need to implement.
pub trait Serializable {
    /// Return object intensive json string
    /// # I.E.
    /// `FourVec -> {"x0":1.0,"x1":0.0,"x2":0.0,"x3":0.0}`
    fn to_json(&self) -> String;
    /// Return Result wrapped Vec<u8> in MsgPack
    /// Format is *not* like to_json it is array intensive not object
    ///
    /// #Errors
    /// * The rmp library returns `ValueWriteError` on write errors
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> ;
}
/// Deserialization trait which all types you intend to get out of a Tree need to implement.
/// Really only designed to work with data that was serialized with the Calcify::Serializable trait
/// and will not work on arbitrarily modified tree files
pub trait Deserializable {
    /// Return Self from string
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>>
        where Self: Sized;
    /// Return a tuple of Self and a byte array of remaining unparsed bytes from a byte array
    fn from_msg(bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>>
        where Self: Sized;
}

impl Serializable for u64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }

    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_uint(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for u64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        string.parse::<u64>().map_err(|e| e.into())
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        Ok((read_int(&mut bytes)?,bytes))
    }
}


impl Serializable for f64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_f64(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for f64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        string.parse::<f64>().map_err(|e| e.into())
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        Ok((read_f64(&mut bytes)?,bytes))
    }
}


/// Wraps the String in quotes("").
impl Serializable for String {
    fn to_json(&self) -> String {
        format!("\"{}\"",self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_str(&mut buf, self.as_str())?;
        Ok(buf)
    }
}
