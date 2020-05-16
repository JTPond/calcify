use std::collections::HashMap;

mod branch;

pub use branch::Collection;
pub use branch::Bin;
pub use branch::Point;
pub use branch::Branch;

mod feedtree;

pub use feedtree::Feed;
pub use feedtree::FeedTree;

use crate::utils;

use utils::Serializable;
use utils::errors::CalcifyError;


extern crate rmp;
use rmp::encode::*;

/// Tree of Collections for saving to a file.
pub struct Tree<'a> {
    metadata: HashMap<&'a str,&'a str>,
    branches: HashMap<&'a str,Branch>,
}

impl<'a> Tree<'a> {
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
    pub fn new(name: &'static str) -> Tree {
        let mut md = HashMap::new();
        md.insert("Name",name);
        let br = HashMap::new();
        Tree {
            metadata: md,
            branches: br,
        }
    }

    pub fn add_field(&mut self, key: &'a str, f: &'static str) -> Result<(),CalcifyError> {
        if let Some(_) = self.metadata.insert(key,f) {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    /// Inserts new branch into Tree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, &'static str
    /// * `b` - Branch,  Collection<T: Serializable>
    /// * `t` - Collection subtype,  &'static str, one of "f64", "String", "ThreeVec", "ThreeMat", "FourVec", "FourMat", "Bin", "Point", "Object"
    ///
    /// # Panics
    ///
    /// * `t` is invalid
    pub fn add_branch<T: 'static + Serializable>(&mut self, key: &'a str, b: Collection<T>, t: &'static str) -> Result<(),CalcifyError> {
        let types = ["f64","String","ThreeVec","ThreeMat","FourVec","FourMat","Bin","Point","Object"];
        if types.contains(&t) {
            let br = Branch::new(String::from(t),Box::new(b));
            if let Some(_) = self.branches.insert(key,br) {
                return Err(CalcifyError::KeyError);
            }
            Ok(())
        } else {
            panic!("Subtype must be one of \"f64\", \"String\", \"ThreeVec\", \"ThreeMat\", \"FourVec\", \"FourMat\", \"Bin\", \"Point\", \"Object\" not {}",t);
        }
    }

    /// Returns Branch from a Trees
    ///
    /// # Note
    /// * Branch has no internal Collection functionality, this is intended to only be used with the extract function to get the Collection.
    ///
    /// # Arguments
    ///
    /// `key` - &'static str
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
    /// ttree.add_branch("fcol", f_col, "f64");
    /// ttree.add_branch("bCol", b_col, "Bin");
    ///
    /// let ex_f_col: Collection<f64> = ttree.get_branch("fcol").unwrap().extract().unwrap();
    /// let ex_b_col: Collection<Bin> = ttree.get_branch("bCol").unwrap().extract().unwrap();
    ///
    /// assert_eq!(Collection::from(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]),ex_b_col);
    /// ```
    pub fn get_branch(&mut self, key: &'a str) -> Option<&Branch> {
        self.branches.get(key)
    }
}

impl Serializable for Tree<'_> {
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

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::fs::File;
    use super::*;
    use crate::ThreeVec;

    #[test]
    fn test_tree_json() -> Result<(),CalcifyError>{
        let f = File::create("test_tree.json").unwrap();
        let mut wr = BufWriter::new(f);
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::random(1.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol, "f64")?;
        ttree.add_branch("col_3v", col_3v, "ThreeVec")?;
        wr.write(ttree.to_json().as_bytes()).unwrap();
        Ok(())
    }
}
