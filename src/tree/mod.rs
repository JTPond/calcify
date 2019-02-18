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

/// Tree of Collections for saving to a file.
pub struct Tree {
    pub metadata: HashMap<&'static str,&'static str>,
    pub branches: HashMap<&'static str,Box<Serializable>>,
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
    /// let fCol: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
    /// let mut v3Col: Collection<ThreeVec> = Collection::empty();
    /// for _i in 0..9 {v3Col.push(ThreeVec::random(1.0));}
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_field("Desc", "This is a Tree for testing.");
    /// ttree.add_branch("fCol", fCol);
    /// ttree.add_branch("v3Col", v3Col);
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

    pub fn add_branch<T: 'static + Serializable>(&mut self, key: &'static str, b: Collection<T>) {
        self.branches.insert(key,Box::new(b));
    }
}

impl Serializable for Tree {
    /// Returns Json encoded String representation of the tree
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::prelude::*;
    /// use std::io::BufWriter;
    /// use std::fs::File;
    ///
    /// use calcify::Tree;
    /// use calcify::Collection;
    /// use calcify::ThreeVec;
    ///
    /// let f = File::create("test_tree.json").unwrap();
    /// let mut wr = BufWriter::new(f);
    ///
    /// let fCol: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
    /// let mut v3Col: Collection<ThreeVec> = Collection::empty();
    /// for _i in 0..9 {v3Col.push(ThreeVec::random(1.0));}
    ///
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_field("Desc", "This is a Tree for testing.");
    /// ttree.add_branch("fCol", fCol);
    /// ttree.add_branch("v3Col", v3Col);
    ///
    /// wr.write(ttree.to_json().as_bytes()).unwrap();
    /// ```
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
        ttree.add_branch("fCol", fCol);
        ttree.add_branch("v3Col", v3Col);
        wr.write(ttree.to_json().as_bytes()).unwrap();
    }
}
