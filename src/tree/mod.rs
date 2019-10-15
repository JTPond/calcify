use std::collections::HashMap;
use std::str::FromStr;
use std::num::ParseFloatError;

mod feedtree;

pub use feedtree::Feed;
pub use feedtree::FeedTree;

mod collection;

pub use collection::Collection;
pub use collection::Bin;
pub use collection::Point;

pub use collection::Sinv;
pub use collection::beta;
pub use collection::gamma;
pub use collection::boost;
pub use collection::FourVec;
pub use collection::FourMat;
pub use collection::LightSpeedError;

pub use collection::ThreeMat;
pub use collection::ThreeVec;
pub use collection::{radians_between, degrees_between};

pub use collection::consts;
pub use collection::Serializable;

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

/// Tree of Collections for saving to a file.
pub struct Tree {
    metadata: HashMap<&'static str,&'static str>,
    branches: HashMap<&'static str,Branch>,
}

impl Branch {
    pub fn new(subtype: String, branch: Box<dyn Serializable>) -> Branch{
        Branch {
            subtype,
            branch,
        }
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_str(&self) -> Collection<String> {
        if self.subtype != "String" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<String> = Collection::empty();
        for ff in self.branch.to_json().trim_matches(|p| p == '[' || p == ']' ).split(','){
            out.push(ff.trim_matches(|p| p == '"').to_string());
        }
        out
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_f64(&self) -> Result<Collection<f64>, ParseFloatError> {
        if self.subtype != "f64" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<f64> = Collection::empty();
        for ff in self.branch.to_json().trim_matches(|p| p == '[' || p == ']' ).split(','){
            let f: f64 = ff.parse::<f64>()?;
            out.push(f);
        }
        Ok(out)
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_3v(&self) -> Result<Collection<ThreeVec>, ParseFloatError> {
        if self.subtype != "ThreeVec" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<ThreeVec> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: ThreeVec = ThreeVec::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_3m(&self) -> Result<Collection<ThreeMat>, ParseFloatError> {
        if self.subtype != "ThreeMat" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<ThreeMat> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: ThreeMat = ThreeMat::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_4v(&self) -> Result<Collection<FourVec>, ParseFloatError> {
        if self.subtype != "FourVec" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<FourVec> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: FourVec = FourVec::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_4m(&self) -> Result<Collection<FourMat>, ParseFloatError> {
        if self.subtype != "FourMat" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<FourMat> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: FourMat = FourMat::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_bin(&self) -> Result<Collection<Bin>, ParseFloatError> {
        if self.subtype != "Bin" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<Bin> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: Bin = Bin::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    /// Returns a Collection of the specified subtype from the Branch
    ///
    /// # Panics
    /// Branch.subtype does not match the used extract function.
    pub fn extract_point(&self) -> Result<Collection<Point>, ParseFloatError> {
        if self.subtype != "Point" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<Point> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: Point = Point::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }
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
    /// let f_col: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
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

    pub fn add_field(&mut self, key: &'static str, f: &'static str) {
        self.metadata.insert(key,f);
    }

    /// Inserts new branch into Tree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, &'static str
    /// * `b` - Branch,  Collection<T: Serializable>
    /// * `t` - Collection subtype,  &'static str, one of "f64", "String", "ThreeVec", "ThreeMat", "FourVec", "FourMat", "Bin", "Point"
    ///
    /// # Panics
    ///
    /// * `t` is invalid
    pub fn add_branch<T: 'static + Serializable>(&mut self, key: &'static str, b: Collection<T>, t: &'static str) {
        match t {
            "f64" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "String" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "ThreeVec" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "ThreeMat" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "FourVec" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "FourMat" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "Bin" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            "Point" => {
                let br = Branch::new(String::from(t),Box::new(b));
                self.branches.insert(key,br);
            },
            x => panic!("Subtype must be one of \"f64\", \"String\", \"ThreeVec\", \"ThreeMat\", \"FourVec\", \"FourMat\", \"Bin\", \"Point\", not {}",x),
        }
    }

    /// Returns Branch from a Trees
    ///
    /// # Note
    /// * Branch has no internal Collection functionality, this is intended to only be used with the appropriate extract function to get the Collection.
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
    /// let f_col: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
    /// let s_col: Collection<String> = Collection::from_vec(vec![String::from("test0"),String::from("test1")]);
    /// let b_col: Collection<Bin> = Collection::from_vec(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_branch("fcol", f_col, "f64");
    /// ttree.add_branch("sCol", s_col, "String");
    /// ttree.add_branch("bCol", b_col, "Bin");
    ///
    /// let ex_f_col: Collection<f64> = ttree.get_branch("fcol").unwrap().extract_f64().unwrap();
    /// let ex_s_col: Collection<String> = ttree.get_branch("sCol").unwrap().extract_str();
    /// let ex_b_col: Collection<Bin> = ttree.get_branch("bCol").unwrap().extract_bin().unwrap();
    ///
    /// assert_eq!(Collection::from_vec(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from_vec(vec![String::from("test0"),String::from("test1")]),ex_s_col);
    /// assert_eq!(Collection::from_vec(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]),ex_b_col);
    /// ```
    pub fn get_branch(&mut self, key: &'static str) -> Option<&Branch> {
        self.branches.get(key)
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
    fn to_jsonc(&self) -> String {
        let mut out = String::from("{");
        for (key, val) in &self.metadata {
            out.push_str(format!("\"{}\":\"{}\",",key,val).as_str());
        }
        out.push_str("\"branches\":{");
        for (key, val) in &self.branches {
            out.push_str(format!("\"{}\":{},",key,val.to_jsonc()).as_str());
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

    #[test]
    fn test_tree_json() {
        let f = File::create("test_tree.json").unwrap();
        let mut wr = BufWriter::new(f);
        let fcol: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::random(1.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.");
        ttree.add_branch("fcol", fcol, "f64");
        ttree.add_branch("col_3v", col_3v, "ThreeVec");
        wr.write(ttree.to_json().as_bytes()).unwrap();
    }
}
