use std::collections::HashMap;
use std::error;

mod branch;

pub use branch::Collection;
pub use branch::Bin;
pub use branch::Point;
pub use branch::Branch;

mod feedtree;

pub use feedtree::FeedTree;

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;
use utils::io::{ToFile,FromFile};

extern crate rmp;
use rmp::encode::*;
use rmp::decode::*;

/// Tree of Collections for saving to a file.
pub struct Tree {
    metadata: HashMap<String,String>,
    branches: HashMap<String,Branch>,
}

impl Tree {
    /// Returns new Tree
    ///
    /// Name is the only required metadata.
    /// # Arguments
    ///
    /// * `name` - string
    ///
    /// # Example
    /// ```
    /// use calcify::Tree;
    /// use calcify::Collection;
    /// use calcify::Bin;
    /// use calcify::ThreeVec;
    ///
    /// let f_col: Collection<f64> = Collection::from(vec![0.0,0.0]);
    /// let mut v3_col: Collection<ThreeVec> = Collection::empty();
    /// for _i in 0..9999 {v3_col.push(ThreeVec::random(1.0));}
    /// let col_hist: Collection<Bin> = v3_col.map(ThreeVec::r).hist(50);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_field("Desc", "This is a Tree for testing.");
    /// ttree.add_branch("fcol", f_col, "f64");
    /// ttree.add_branch("col_3v", v3_col, "ThreeVec");
    /// ttree.add_branch("hist_3v", col_hist, "Bin");
    /// ```
    pub fn new(name: &str) -> Tree {
        let mut md = HashMap::new();
        md.insert(String::from("Name"),String::from(name));
        let br = HashMap::new();
        Tree {
            metadata: md,
            branches: br,
        }
    }

    pub fn add_field(&mut self, key: &str, f: &str) -> Result<(),CalcifyError> {
        if let Some(_) = self.metadata.insert(String::from(key),String::from(f)) {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    /// Inserts new branch into Tree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, String
    /// * `b` - Branch,  Collection<T: Serializable>
    /// * `t` - Collection subtype,  String, one of "f64", "String", "ThreeVec", "ThreeMat", "FourVec", "FourMat", "Bin", "Point", "Object"
    ///
    /// # Panics
    ///
    /// * `t` is invalid
    pub fn add_branch<T: 'static + Serializable>(&mut self, key: &str, b: Collection<T>, t: &str) -> Result<(),CalcifyError> {
        let types = ["f64","String","ThreeVec","ThreeMat","FourVec","FourMat","Bin","Point","Object"];
        if types.contains(&t) {
            let br = Branch::new(String::from(t),Box::new(b));
            if let Some(_) = self.branches.insert(String::from(key),br) {
                return Err(CalcifyError::KeyError);
            }
            Ok(())
        } else {
            panic!("Subtype must be one of \"f64\", \"String\", \"ThreeVec\", \"ThreeMat\", \"FourVec\", \"FourMat\", \"Bin\", \"Point\", \"Object\" not {}",t);
        }
    }

    /// Returns Branch from a Trees
    ///
    /// # Arguments
    ///
    /// `key` - String
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::Tree;
    /// use calcify::Collection;
    /// use calcify::Bin;
    ///
    /// let f_col: Collection<f64> = Collection::from(vec![0.0,0.0]);
    /// let b_col: Collection<Bin> = Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_branch("fcol", f_col, "f64").expect("KeyError");
    /// ttree.add_branch("bCol", b_col, "Bin").expect("KeyError");
    ///
    /// let ex_f_col: Collection<f64> = ttree.get_branch("fcol").unwrap().extract().unwrap();
    /// let mut ex_b_col: Collection<Bin> = ttree.get_branch("bCol").unwrap().extract().unwrap();
    ///
    /// assert_eq!(Collection::from(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]),ex_b_col);
    /// ```
    pub fn get_branch(&mut self, key: &str) -> Option<&mut Branch> {
        self.branches.get_mut(&String::from(key))
    }

    /// Returns Collection from a Trees
    ///
    /// # Arguments
    ///
    /// `key` - String
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::Tree;
    /// use calcify::Collection;
    /// use calcify::Bin;
    ///
    /// let f_col: Collection<f64> = Collection::from(vec![0.0,0.0]);
    /// let b_col: Collection<Bin> = Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_branch("fcol", f_col, "f64").expect("KeyError");
    /// ttree.add_branch("bCol", b_col, "Bin").expect("KeyError");
    ///
    /// let ex_f_col: Collection<f64> = ttree.read_branch("fcol").unwrap();
    /// let mut ex_b_col: Collection<Bin> = ttree.read_branch("bCol").unwrap();
    /// // read from buffer
    /// ex_b_col = ttree.read_branch("bCol").unwrap();
    ///
    /// assert_eq!(Collection::from(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]),ex_b_col);
    /// ```
    pub fn read_branch<T: Serializable + Deserializable>(&mut self, key: &str) -> Result<Collection<T>, CalcifyError> {
        if let Some(branch) = self.branches.get_mut(&String::from(key)) {
            if let Ok(collect) = branch.extract() {
                return Ok(collect);
            } else {
                return Err(CalcifyError::ParseError);
            }
        }
        Err(CalcifyError::KeyError)
    }
}

impl Serializable for Tree {
    fn to_json(&self) -> String {
        let mut out = String::from("{");
        for (key, val) in &self.metadata {
            out.push_str(format!("\"{}\":\"{}\",",key,val).as_str());
        }
        out.push_str("\"branches\":{");
        for (key, val) in &self.branches {
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
        write_str(&mut buf, "branches")?;
        write_map_len(&mut buf, (self.branches.len()) as u32)?;
        for (key, val) in &self.branches {
            write_str(&mut buf, key)?;
            buf.append(&mut val.to_msg()?);
        }
        Ok(buf)
    }
}

impl Deserializable for Tree {
    fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
        let mut metadata: HashMap<String,String> = HashMap::new();
        let mut branches: HashMap<String,Branch> = HashMap::new();
        for (i,mut dim) in s.split(",\"branches\":").enumerate() {
            match i {
                0 => {
                    for pair in dim.trim_matches(|p| p == '{' || p == '"' ).split("\",\"") {
                        let ar: Vec<&str> = pair.split("\":\"").collect();
                        metadata.insert(String::from(ar[0]),String::from(ar[1]));
                    }
                },
                1 => {
                    dim = dim.trim_matches(|p| p == '{' || p == '}' || p == '"' );
                    for pair in dim.split("},\"") {
                        let ar: Vec<&str> = pair.split("\":").collect();
                        if let Ok(branch) = Branch::from_json(&ar[1..].join("\":")){
                            branches.insert(String::from(ar[0]),branch);
                        } else {
                            return Err(Box::new(CalcifyError::ParseError));
                        }
                    }
                },
                _ => return Err(Box::new(CalcifyError::ParseError)),
            }
        }
        Ok(Tree{metadata, branches})
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        let mut metadata: HashMap<String,String> = HashMap::new();
        let mut branches: HashMap<String,Branch> = HashMap::new();
        if let Ok(len) = read_map_len(&mut bytes) {
            for _ in 0..len {
                let mut unparsed: &[u8] = &bytes[..];
                if let Ok((key,v_rest)) = read_str_from_slice(unparsed) {
                    match key {
                        "branches" => {
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
                        if let Ok((value,rest)) = Branch::from_msg(v_rest) {
                            bytes = rest;
                            branches.insert(String::from(key),value);
                        }
                    }
                }
            }
        }
        Ok((Tree{metadata, branches},bytes))
    }
}

impl ToFile for Tree{}
impl FromFile for Tree{}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreeVec;

    #[test]
    fn test_tree_write() -> Result<(),Box<dyn error::Error>>{
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol, "f64")?;
        ttree.add_branch("col_3v", col_3v, "ThreeVec")?;
        ttree.write_msg("test_tree.msg")?;
        Ok(())
    }

    #[test]
    fn test_tree_read() -> Result<(),Box<dyn error::Error>>{
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}

        let mut ftree = Tree::read_msg("test_tree.msg")?;
        let ftree_branch1: Collection<f64> = ftree.read_branch("fcol")?;
        assert_eq!(ftree_branch1,fcol);
        let ftree_branch2: Collection<ThreeVec> = ftree.read_branch("col_3v")?;
        assert_eq!(ftree_branch2,col_3v);
        Ok(())
        // Err(Box::new(CalcifyError::ParseError))
    }

    #[test]
    fn test_tree_json() -> Result<(),Box<dyn error::Error>>{
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::random(1.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("col_3v", col_3v.clone(), "ThreeVec")?;
        ttree.add_branch("fcol", fcol, "f64")?;
        ttree.add_branch("col_3v3", col_3v, "ThreeVec")?;
        let pp = ttree.to_json();
        let mut oo = Tree::from_json(&pp)?;
        assert_eq!(oo.read_branch("fcol").unwrap(),Collection::from(vec![0.0,0.0]));
        Ok(())
    }

    #[test]
    fn test_tree_msg() -> Result<(),Box<dyn error::Error>>{
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::random(1.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol, "f64")?;
        ttree.add_branch("col_3v", col_3v, "ThreeVec")?;
        let pp = ttree.to_msg().unwrap();
        let (mut oo,_) = Tree::from_msg(&pp).unwrap();
        assert_eq!(oo.read_branch("fcol").unwrap(),Collection::from(vec![0.0,0.0]));
        Ok(())
    }
}
