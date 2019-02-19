use std::collections::HashMap;
use std::str::FromStr;
use std::num::ParseFloatError;

mod collection;

pub use collection::Collection;

pub use collection::Sinv;
pub use collection::beta;
pub use collection::gamma;
pub use collection::boost;
pub use collection::FourVec;
pub use collection::FourMat;

pub use collection::ThreeMat;
pub use collection::ThreeVec;
pub use collection::{radians_between, degrees_between};

pub use collection::consts;
pub use collection::Serializable;

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

    pub fn extract_f64(&self) -> Result<Collection<f64>, ParseFloatError> {
        if self.subtype != "f64" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<f64> = Collection::empty();
        for ff in self.branch.to_json().trim_matches(|p| p == '[' || p == ']' ).split(','){
            let f: f64 = ff.parse::<f64>()?;
            out.push(f);
        }
        Ok(out)
    }

    pub fn extract_ThreeVec(&self) -> Result<Collection<ThreeVec>, ParseFloatError> {
        if self.subtype != "ThreeVec" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<ThreeVec> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: ThreeVec = ThreeVec::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    pub fn extract_ThreeMat(&self) -> Result<Collection<ThreeMat>, ParseFloatError> {
        if self.subtype != "ThreeMat" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<ThreeMat> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: ThreeMat = ThreeMat::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    pub fn extract_FourVec(&self) -> Result<Collection<FourVec>, ParseFloatError> {
        if self.subtype != "FourVec" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<FourVec> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: FourVec = FourVec::from_str(ff)?;
            out.push(f);
        }
        Ok(out)
    }

    pub fn extract_FourMat(&self) -> Result<Collection<FourMat>, ParseFloatError> {
        if self.subtype != "FourMat" {panic!("Used incorrect extract function. Check Branch.subtype.")}
        let mut out: Collection<FourMat> = Collection::empty();
        for ff in self.branch.to_json().replace("},{","}|{").trim_matches(|p| p == '[' || p == ']' ).split('|'){
            let f: FourMat = FourMat::from_str(ff)?;
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
    /// use calcify::ThreeVec;
    ///
    /// let f_col: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
    /// let mut v3_col: Collection<ThreeVec> = Collection::empty();
    /// for _i in 0..9 {v3_col.push(ThreeVec::random(1.0));}
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_field("Desc", "This is a Tree for testing.");
    /// ttree.add_branch("fCol", f_col, "f64");
    /// ttree.add_branch("v3Col", v3_col, "ThreeVec");
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
    /// * `t` - Collection subtype,  &'static str, one of "f64", "String", "ThreeVec", "ThreeMat", "FourVec", "FourMat"
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
            x => panic!("Subtype must be one of \"f64\", \"String\", \"ThreeVec\", \"ThreeMat\", \"FourVec\", \"FourMat\", not {}",x),
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
    ///
    /// let f_col: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
    /// let s_col: Collection<String> = Collection::from_vec(vec![String::from("test0"),String::from("test1")]);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_branch("fCol", f_col, "f64");
    /// ttree.add_branch("sCol", s_col, "String");
    ///
    /// let ex_f_col: Collection<f64> = ttree.get_branch("fCol").unwrap().extract_f64().unwrap();
    /// let ex_s_col: Collection<String> = ttree.get_branch("sCol").unwrap().extract_str();
    ///
    /// assert_eq!(Collection::from_vec(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from_vec(vec![String::from("test0"),String::from("test1")]),ex_s_col);
    /// ```
    pub fn get_branch(&mut self, key: &'static str) -> Option<&Branch> {
        self.branches.get(key)
    }
}

impl Serializable for Branch {
    fn to_json(&self) -> String {
        format!("{{\"branch\":{},\"subtype\":{}}}",self.branch.to_json(),self.subtype.to_json())
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
        let fCol: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
        let mut v3Col: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {v3Col.push(ThreeVec::random(1.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.");
        ttree.add_branch("fCol", fCol, "f64");
        ttree.add_branch("v3Col", v3Col, "ThreeVec");
        wr.write(ttree.to_json().as_bytes()).unwrap();
    }
}
