use std::collections::HashMap;
use std::error;
use super::Collection;

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;
use utils::io::{ToFile,FromFile};

extern crate rmp;
use rmp::encode::*;
use rmp::decode::*;

/// Tree of Collections of only a single type, which impl the Feed trait for added functionality
#[derive(Debug, PartialEq, Clone)]
pub struct FeedTree<T: Serializable> {
    metadata: HashMap<String,String>,
    datafeeds: HashMap<String,Collection<T>>,
}

impl<T: Serializable> FeedTree<T> {
    /// Returns new FeedTree
    ///
    /// Name and subtype is the only required metadata.
    /// # Arguments
    ///
    /// * `name` - string
    /// * `subtype` - string, must match given Serializable type
    ///
    /// # Example
    /// ```
    /// use calcify::FeedTree;
    /// use calcify::Collection;
    ///
    /// let f_col: Collection<f64> = Collection::from(vec![0.0,0.0]);
    /// let mut ftree: FeedTree<f64> = FeedTree::new("Test_Tree","f64");
    /// ftree.add_field("Desc", "This is a FeedTree for testing.");
    /// ftree.add_feed("fcol", f_col);
    /// ftree.write("fcol", 1.0);
    /// ftree.write("fcol", 2.0);
    /// assert_eq!(Collection::from(vec![0.0,0.0,1.0,2.0]),*ftree.get_feed("fcol").unwrap())
    /// ```
    pub fn new(name: &str, subtype: &str) -> FeedTree<T> {
        let mut md = HashMap::new();
        md.insert(String::from("Name"),String::from(name));
        md.insert(String::from("SubType"),String::from(subtype));
        let df = HashMap::new();
        FeedTree {
            metadata: md,
            datafeeds: df,
        }
    }

    pub fn add_field(&mut self, key: &str, f: &str) -> Result<(),CalcifyError> {
        if let Some(_) = self.metadata.insert(String::from(key),String::from(f)) {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    /// Inserts new Collection<T> into FeedTree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, String
    /// * `f` - Collection<T: Serializable>
    pub fn add_feed(&mut self, key: &str, f: Collection<T>) -> Result<(),CalcifyError> {
        if let Some(_) = self.datafeeds.insert(String::from(key),f) {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    pub fn get_feed(&mut self, key: &str) -> Option<&Collection<T>> {
        self.datafeeds.get(key)
    }

    pub fn write(&mut self, key: &str, data: T) -> Result<(),CalcifyError> {
        if let Some(feed) = self.datafeeds.get_mut(key) {
            feed.push(data);
            Ok(())
        } else {
            Err(CalcifyError::KeyError)
        }
    }
}

impl<T: Serializable> Serializable for FeedTree<T> {
    fn to_json(&self) -> String {
        let mut out = String::from("{");
        for (key, val) in &self.metadata {
            out.push_str(format!("\"{}\":\"{}\",",key,val).as_str());
        }
        out.push_str("\"datafeeds\":{");
        for (key, val) in &self.datafeeds {
            out.push_str(format!("\"{}\":{},",key,val.to_json()).as_str());
        }
        out.pop();
        out.push_str("}}");
        out
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_map_len(&mut buf, (self.metadata.len()+1) as u32)?;
        for (key, val) in &self.metadata {
            write_str(&mut buf, key)?;
            write_str(&mut buf, val)?;
        }
        write_str(&mut buf, "datafeeds")?;
        write_map_len(&mut buf, (self.datafeeds.len()) as u32)?;
        for (key, val) in &self.datafeeds {
            write_str(&mut buf, key)?;
            buf.append(&mut val.to_msg()?);
        }
        Ok(buf)
    }
}

impl<T: Serializable + Deserializable> Deserializable for FeedTree<T> {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut metadata: HashMap<String,String> = HashMap::new();
        let mut datafeeds: HashMap<String,Collection<T>> = HashMap::new();
        for (i,dim) in s.split(",\"datafeeds\":").enumerate() {
            match i {
                0 => {
                    for pair in dim.trim_matches(|p| p == '{' || p == '"' ).split("\",\"") {
                        let ar: Vec<&str> = pair.split("\":\"").collect();
                        metadata.insert(String::from(ar[0]),String::from(ar[1]));
                    }
                },
                1 => {
                    for pair in dim.trim_matches(|p| p == '{' || p == '}' || p == '"' ).split("],\"") {
                        let ar: Vec<&str> = pair.split("\":").collect();
                        if let Ok(feed) = Collection::<T>::from_json(&ar[1..].join("\":")){
                            datafeeds.insert(String::from(ar[0]),feed);
                        } else {
                            return Err(Box::new(CalcifyError::ParseError));
                        }
                    }
                },
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }
        Ok(FeedTree{metadata, datafeeds})
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        let mut metadata: HashMap<String,String> = HashMap::new();
        let mut datafeeds: HashMap<String,Collection<T>> = HashMap::new();
        if let Ok(len) = read_map_len(&mut bytes) {
            for _ in 0..len {
                let mut unparsed: &[u8] = &bytes[..];
                if let Ok((key,v_rest)) = read_str_from_slice(unparsed) {
                    match key {
                        "datafeeds" => {
                            bytes = v_rest;
                            break;
                        },
                        _ => {
                            if let Ok((value,rest)) = read_str_from_slice(v_rest) {
                                unparsed = rest;
                                metadata.insert(String::from(key),String::from(value));
                            }
                        },
                    }

                }
                bytes = unparsed;
            }
            if let Ok(flen) = read_map_len(&mut bytes) {
                for _ in 0..flen {
                    let unparsed: &[u8] = &bytes[..];
                    if let Ok((key,v_rest)) = read_str_from_slice(unparsed) {
                        if let Ok((value,rest)) = Collection::<T>::from_msg(v_rest) {
                            bytes = rest;
                            datafeeds.insert(String::from(key),value);
                        }
                    }
                }
            }
        }
        Ok((FeedTree{metadata, datafeeds},bytes))
    }
}

impl<T: Serializable> ToFile for FeedTree<T>{}
impl<T: Serializable + Deserializable> FromFile for FeedTree<T>{}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreeVec;

    #[test]
    fn test_ftree_write() -> Result<(),Box<dyn error::Error>>{
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}
        let mut ttree = FeedTree::new("Test_Tree","ThreeVec");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v)?;
        ttree.write_msg("test_ftree.msg")?;
        Ok(())
    }

    #[test]
    fn test_ftree_read() -> Result<(),Box<dyn error::Error>>{
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}
        let mut ttree = FeedTree::new("Test_Tree","ThreeVec");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v)?;

        let ftree: FeedTree<ThreeVec> = FeedTree::read_msg("test_ftree.msg")?;
        assert_eq!(ftree,ttree);
        Ok(())
    }

    #[test]
    fn test_ftree_json() -> Result<(),Box<dyn error::Error>>{
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::random(1.0));}
        let mut ttree = FeedTree::new("Test_Tree","ThreeVec");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v.clone())?;
        ttree.add_feed("fcol1", col_3v.clone())?;
        ttree.add_feed("fcol2", col_3v)?;
        let pp = ttree.to_json();
        let oo = FeedTree::<ThreeVec>::from_json(&pp)?;
        println!("{:?}",oo);
        Ok(())
    }

    #[test]
    fn test_ftree_msg() -> Result<(),Box<dyn error::Error>>{
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::random(1.0));}
        let mut ttree = FeedTree::new("Test_Tree","ThreeVec");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v.clone())?;
        ttree.add_feed("fcol2", col_3v)?;
        let pp = ttree.to_msg().unwrap();
        let (oo,_) = FeedTree::<ThreeVec>::from_msg(&pp).unwrap();
        assert_eq!(oo,ttree);
        Ok(())
    }
}
