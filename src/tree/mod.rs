use std::collections::HashMap;

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
struct Branch {
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

    pub fn add_branch<T: 'static + Serializable>(&mut self, key: &'static str, b: Collection<T>, t: &'static str) {
        let br = Branch::new(String::from(t),Box::new(b));
        self.branches.insert(key,br);
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
