use std::str::FromStr;

mod collection;
pub use collection::Collection;
pub use collection::Bin;
pub use collection::Point;

use crate::utils::Serializable;
use crate::utils::errors::CalcifyError;

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
    pub fn extract<T: Serializable + FromStr>(&self) -> Result<Collection<T>, CalcifyError> {
        let mut out: Collection<T> = Collection::empty();
        match &self.subtype[..] {
            "f64" => {
                for ff in self.branch.to_json().trim_matches(|p| p == '[' || p == ']' ).split(','){
                    if let Ok(f) = ff.parse::<T>() {
                        out.push(f);
                    }
                    else {
                        return Err(CalcifyError::ExtractError);
                    }
                }
            },
            _ => {
                for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
                    if let Ok(f) = T::from_str(&ff) {
                        out.push(f);
                    }
                    else {
                        return Err(CalcifyError::ExtractError);
                    }
                }
            },
        }
        Ok(out)
    }
}

impl Serializable for Branch {
    fn to_json(&self) -> String {
        format!("{{\"branch\":{},\"subtype\":{}}}",self.branch.to_json(),self.subtype.to_json())
    }
    fn to_jsonc(&self) -> String {
        format!("{{\"branch\":{},\"subtype\":{}}}",self.branch.to_jsonc(),self.subtype.to_jsonc())
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
