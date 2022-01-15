use std::error;
use std::f64;

mod collection;
pub use collection::Collection;
pub use collection::Bin;
pub use collection::Point;
pub use collection::PointBin;
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
    buffer: Option<Vec<u8>>,
}

impl Branch{
    pub fn new(subtype: String, branch: Box<dyn Serializable>) -> Branch{
        let buffer: Option<Vec<u8>> = None;
        Branch {
            subtype,
            branch,
            buffer,
        }
    }
    /// Returns a Collection of the specified subtype from the Branch
    ///
    pub fn extract<T: Serializable + Deserializable>(&mut self) -> Result<Collection<T>, Box<dyn error::Error>> {
        if self.buffer.is_none() {
            self.buffer = Some(self.branch.to_msg()?);
        }
        if let Ok((out, _)) = Collection::<T>::from_msg(&mut self.buffer.as_ref().unwrap()){
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
        let pattern: Vec<char> = "{\"subtype\":}".chars().collect();
        for (i,dim) in s.trim_matches(|p| pattern.contains(&p)).split(",\"branch\":").enumerate() {
            match i {
                0 => subtype = dim.trim_matches(|p| p == '\"'),
                1 => branch_str = dim,
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
            "PointBin" => Box::new(Collection::<PointBin>::from_json(&branch_str)?),
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
                        let (branch,rest): (Box<dyn Serializable>,&[u8])  = match subtype {
                            "f64" => {
                                if let Ok((ot,rest)) = Collection::<f64>::from_msg(unparsed) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "ThreeVec" => {
                                if let Ok((ot,rest)) = Collection::<ThreeVec>::from_msg(unparsed) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "ThreeMat" => {
                                if let Ok((ot,rest)) = Collection::<ThreeMat>::from_msg(unparsed) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "FourVec" => {
                                if let Ok((ot,rest)) = Collection::<FourVec>::from_msg(unparsed) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "FourMat" => {
                                if let Ok((ot,rest)) = Collection::<FourMat>::from_msg(unparsed) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "Bin" => {
                                if let Ok((ot,rest)) = Collection::<Bin>::from_msg(&mut bytes) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "Point" => {
                                if let Ok((ot,rest)) = Collection::<Point>::from_msg(&mut bytes) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "PointBin" => {
                                if let Ok((ot,rest)) = Collection::<PointBin>::from_msg(&mut bytes) {
                                    (Box::new(ot),rest)
                                } else {
                                    return Err(Box::new(CalcifyError::ParseError));
                                }
                            },
                            "Object" => {
                                return Err(Box::new(CalcifyError::ObjectBranchDeserializeError));
                            },
                            _ => return Err(Box::new(CalcifyError::ParseError)),
                        };
                        return Ok((Branch::new(subtype.to_string(),branch),rest));
                    }
                }
            }
        }
        Err(Box::new(CalcifyError::ParseError))
    }
}
