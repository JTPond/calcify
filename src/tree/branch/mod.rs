use std::error;

mod collection;
pub use collection::Collection;
pub use collection::Bin;
pub use collection::Point;

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;

extern crate rmp;
use rmp::encode::*;


/// Branch
///
/// # Note
///
/// * **Not intended for direct use. Use the memebers of Tree instead.**
pub struct Branch {
    subtype: String,
    branch: Box<dyn Serializable>,
}

impl Branch{
    pub fn new(subtype: String, branch: Box<dyn Serializable>) -> Branch{
        Branch {
            subtype,
            branch,
        }
    }
    /// Returns a Collection of the specified subtype from the Branch
    ///
    pub fn extract<T: Serializable + Deserializable>(&self) -> Result<Collection<T>, Box<dyn error::Error>> {
        let mut bytes = &(self.branch.to_msg()?)[..];
        if let Ok((out, _)) = Collection::<T>::from_msg(&mut bytes){
            return Ok(out);
        }
        Err(Box::new(CalcifyError::ParseError))
    }
}

impl Serializable for Branch {
    fn to_json(&self) -> String {
        format!("{{\"branch\":{},\"subtype\":{}}}",self.branch.to_json(),self.subtype.to_json())
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_map_len(&mut buf, 2)?;
        write_str(&mut buf, "branch")?;
        buf.append(&mut self.branch.to_msg()?);
        write_str(&mut buf, "subtype")?;
        buf.append(&mut self.subtype.to_msg()?);
        Ok(buf)
    }
}
