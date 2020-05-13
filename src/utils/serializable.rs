extern crate rmp;
use rmp::encode::*;

/// Serialization trait which all types you intend to put in a Tree need to implement.
pub trait Serializable {
    /// Return object intensive json string
    /// # I.E.
    /// `FourVec -> {"x0":1.0,"x1":0.0,"x2":0.0,"x3":0.0}`
    fn to_json(&self) -> String;
    /// Return array intensive compact json string
    /// # I.E.
    /// `FourVec -> [1.0,0.0,0.0,0.0]`
    fn to_jsonc(&self) -> String;
    /// Return Result wrapped Vec<u8> in MsgPack
    /// Format mimics jsonc *not* json
    ///
    /// #Errors
    /// * The rmp library returns `ValueWriteError` on write errors
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> ;
}

impl Serializable for u64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }
    fn to_jsonc(&self) -> String {
        format!("{}",self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_uint(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Serializable for f64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }
    fn to_jsonc(&self) -> String {
        format!("{}",self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_f64(&mut buf, *self)?;
        Ok(buf)
    }
}

/// Wraps the String in quotes("").
impl Serializable for String {
    fn to_json(&self) -> String {
        format!("\"{}\"",self)
    }
    fn to_jsonc(&self) -> String {
        format!("\"{}\"",self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_str(&mut buf, self.as_str())?;
        Ok(buf)
    }
}
