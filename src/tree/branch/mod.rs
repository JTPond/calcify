use std::error;
use std::f64;

mod collection;
pub use collection::Collection;
pub use collection::Bin;
pub use collection::Point;
use crate::four_mat::FourVec;
use crate::four_mat::FourMat;

use crate::three_mat::ThreeMat;
use crate::three_mat::ThreeVec;

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;

extern crate rmp;
use rmp::encode::*;
use rmp::decode::*;


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
        format!("{{\"subtype\":{},\"branch\":{}}}",self.subtype.to_json(),self.branch.to_json())
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_map_len(&mut buf, 2)?;
        write_str(&mut buf, "subtype")?;
        buf.append(&mut self.subtype.to_msg()?);
        write_str(&mut buf, "branch")?;
        buf.append(&mut self.branch.to_msg()?);
        Ok(buf)
    }
}

impl Deserializable for Branch {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut subtype: &str = "";
        let mut branch_str: &str = "";
        let pattern: Vec<char> = "{branch:}".chars().collect();
        for (i,dim) in s.trim_matches(|p| pattern.contains(&p)).split(",subtype:").enumerate() {
            match i {
                0 => branch_str = dim,
                1 => subtype = dim.trim_matches(|p| p == '\"'),
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }
        let branch: Box<dyn Serializable> = match subtype {
            "f64" => Box::new(Collection::<f64>::from_json(&branch_str)?),
            "ThreeVec" => Box::new(Collection::<ThreeVec>::from_json(&branch_str)?),
            "ThreeMat" => Box::new(Collection::<ThreeMat>::from_json(&branch_str)?),
            "FourVec" => Box::new(Collection::<FourVec>::from_json(&branch_str)?),
            "FourMat" => Box::new(Collection::<FourMat>::from_json(&branch_str)?),
            "Bin" => Box::new(Collection::<Bin>::from_json(&branch_str)?),
            "Point" => Box::new(Collection::<Point>::from_json(&branch_str)?),
            _ => return Err(Box::new(CalcifyError::ParseError)),
        };
        Ok(Branch::new(subtype.to_string(),branch))
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {

        if let Ok(_len) = read_map_len(&mut bytes) {
            let mut unparsed = &bytes[..];
            if let Ok((_,rest)) = read_str_from_slice(unparsed) {
                unparsed = rest;
                if let Ok((subtype,rest)) = read_str_from_slice(unparsed) {
                    unparsed = rest;
                    if let Ok((_,rest)) = read_str_from_slice(unparsed) {
                        unparsed = rest;
                        let branch: Box<dyn Serializable>  = match subtype {
                            "f64" => {
                                if let Ok((ot,_rest)) = Collection::<f64>::from_msg(unparsed) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "ThreeVec" => {
                                if let Ok((ot,_rest)) = Collection::<ThreeVec>::from_msg(unparsed) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "ThreeMat" => {
                                if let Ok((ot,_rest)) = Collection::<ThreeMat>::from_msg(unparsed) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "FourVec" => {
                                if let Ok((ot,_rest)) = Collection::<FourVec>::from_msg(unparsed) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "FourMat" => {
                                if let Ok((ot,_rest)) = Collection::<FourMat>::from_msg(unparsed) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "Bin" => {
                                if let Ok((ot,_rest)) = Collection::<Bin>::from_msg(&mut bytes) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "Point" => {
                                if let Ok((ot,_rest)) = Collection::<Point>::from_msg(&mut bytes) {
                                    Box::new(ot)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            _ => return Err(Box::new(CalcifyError::ParseError)),
                        };
                        return Ok((Branch::new(subtype.to_string(),branch),bytes));
                    }
                }
            }
        }
        Err(Box::new(CalcifyError::ParseError))
    }
}
