use std::collections::HashMap;
use std::error;

mod branch;

pub use branch::Collection;
pub use branch::Bin;
pub use branch::Point;
pub use branch::PointBin;
pub use branch::StampedSample;
pub use branch::Branch;
use branch::{STAMPED_SAMPLE_SUBTYPE, VALUE_SUBTYPES};

mod feedtree;

pub use feedtree::FeedTree;

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;
use utils::io::{ToFile,FromFile};
use utils::serializable::split_json_object;

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
        if self.metadata.insert(String::from(key),String::from(f)).is_some() {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    pub fn get_field(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Inserts new branch into Tree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, String
    /// * `b` - Branch,  Collection<T: Serializable>
    /// * `t` - Collection subtype,  String, one of "f64", "i64", "u64", "String", "ThreeVec", "ThreeMat", "FourVec", "FourMat", "Bin", "Point", "PointBin", "Object"
    ///
    /// # Panics
    ///
    /// * `t` is invalid or is `"StampedSample"` (use [`Tree::add_stamped_branch`] instead)
    pub fn add_branch<T: 'static + Serializable>(&mut self, key: &str, b: Collection<T>, t: &str) -> Result<(),CalcifyError> {
        if t == STAMPED_SAMPLE_SUBTYPE {
            panic!("Use Tree::add_stamped_branch(key, branch, value_subtype) for StampedSample branches.");
        }
        if VALUE_SUBTYPES.contains(&t) {
            let br = Branch::new(String::from(t),Box::new(b));
            if self.branches.insert(String::from(key),br).is_some() {
                return Err(CalcifyError::KeyError);
            }
            Ok(())
        } else {
            panic!("Subtype must be one of {:?} not {}", VALUE_SUBTYPES, t);
        }
    }

    /// Inserts a timestamped stream branch (`Collection<StampedSample<T>>`) into `Tree`.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key
    /// * `b` - `Collection<StampedSample<T>>`
    /// * `value_subtype` - Inner value subtype (`"f64"`, `"i64"`, `"u64"`, `"String"`, `"ThreeVec"`, `"ThreeMat"`, `"FourVec"`, `"FourMat"`, `"Bin"`, `"Point"`, `"PointBin"`, or `"Object"`)
    ///
    /// # Panics
    ///
    /// * `value_subtype` is invalid
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, StampedSample, ThreeVec, Tree};
    ///
    /// let stream = Collection::stream(&[1, 2], &[ThreeVec::new(1.0, 0.0, 0.0), ThreeVec::new(0.0, 1.0, 0.0)]);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_stamped_branch("positions", stream, "ThreeVec").unwrap();
    /// let read_back: Collection<StampedSample<ThreeVec>> = ttree.read_stamped_branch("positions").unwrap();
    /// assert_eq!(read_back.len(), 2);
    /// ```
    pub fn add_stamped_branch<T: 'static + Serializable>(
        &mut self,
        key: &str,
        b: Collection<StampedSample<T>>,
        value_subtype: &str,
    ) -> Result<(), CalcifyError> {
        if VALUE_SUBTYPES.contains(&value_subtype) {
            let br = Branch::new_stamped(String::from(value_subtype), Box::new(b));
            if self.branches.insert(String::from(key), br).is_some() {
                return Err(CalcifyError::KeyError);
            }
            Ok(())
        } else {
            panic!(
                "Value subtype must be one of {:?} not {}",
                VALUE_SUBTYPES, value_subtype
            );
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

    /// Reads a timestamped stream branch (`Collection<StampedSample<T>>`) from `Tree`.
    ///
    /// Returns `CalcifyError::KeyError` if `key` does not exist, or `CalcifyError::ParseError`
    /// if the branch is not a `StampedSample` branch or its inner value type does not match `T`.
    pub fn read_stamped_branch<T: Serializable + Deserializable>(
        &mut self,
        key: &str,
    ) -> Result<Collection<StampedSample<T>>, CalcifyError> {
        if let Some(branch) = self.branches.get_mut(key) {
            if branch.subtype() != STAMPED_SAMPLE_SUBTYPE {
                return Err(CalcifyError::ParseError);
            }
            if let Ok(collect) = branch.extract::<StampedSample<T>>() {
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
            out.push_str(&format!("{}:{},", key.to_json(), val.to_json()));
        }
        out.push_str("\"branches\":{");
        let mut first = true;
        for (key, val) in &self.branches {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&format!("{}:{}", key.to_json(), val.to_json()));
        }
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
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut branches: HashMap<String, Branch> = HashMap::new();
        let mut has_branches = false;
        for (key, val_str) in split_json_object(s)? {
            match key.as_str() {
                "branches" => {
                    if has_branches {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    has_branches = true;
                    for (bkey, bval) in split_json_object(val_str)? {
                        let branch = Branch::from_json(bval)?;
                        if branches.insert(bkey, branch).is_some() {
                            return Err(Box::new(CalcifyError::ParseError));
                        }
                    }
                }
                _ => {
                    let val = String::from_json(val_str)?;
                    if metadata.insert(key, val).is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                }
            }
        }
        if !has_branches {
            return Err(Box::new(CalcifyError::ParseError));
        }
        Ok(Tree { metadata, branches })
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut branches: HashMap<String, Branch> = HashMap::new();
        let mut has_branches = false;
        let len = read_map_len(&mut bytes)
            .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
        for _ in 0..len {
            let (key, v_rest) = read_str_from_slice(bytes)
                .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
            match key {
                "branches" => {
                    if has_branches {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    has_branches = true;
                    let mut cur = v_rest;
                    let flen = read_map_len(&mut cur)
                        .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
                    for _ in 0..flen {
                        let (bkey, bv_rest) = read_str_from_slice(cur)
                            .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
                        let (value, rest) = Branch::from_msg(bv_rest)?;
                        cur = rest;
                        if branches.insert(String::from(bkey), value).is_some() {
                            return Err(Box::new(CalcifyError::ParseError));
                        }
                    }
                    bytes = cur;
                }
                _ => {
                    let (value, rest) = String::from_msg(v_rest)?;
                    bytes = rest;
                    if metadata.insert(String::from(key), value).is_some() {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                }
            }
        }
        if !has_branches {
            return Err(Box::new(CalcifyError::ParseError));
        }
        Ok((Tree { metadata, branches }, bytes))
    }
}

impl ToFile for Tree{}
impl FromFile for Tree{}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bin, FourMat, FourVec, Point, PointBin, ThreeMat, ThreeVec};

    #[derive(Debug, Clone, PartialEq)]
    struct CustomObj(f64);

    impl Serializable for CustomObj {
        fn to_json(&self) -> String {
            format!("{{\"v\":{}}}", self.0)
        }
        fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
            self.0.to_msg()
        }
    }

    #[test]
    fn test_tree_write() -> Result<(),Box<dyn error::Error>>{
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol, "f64")?;
        ttree.add_branch("col_3v", col_3v, "ThreeVec")?;
        ttree.write_msg("./scratch/test_tree.msg")?;
        Ok(())
    }

    #[test]
    fn test_tree_read() -> Result<(),Box<dyn error::Error>>{
        let fcol: Collection<f64> = Collection::from(vec![0.0,0.0]);
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol.clone(), "f64")?;
        ttree.add_branch("col_3v", col_3v.clone(), "ThreeVec")?;
        ttree.write_msg("./scratch/test_tree_read.msg")?;

        let mut ftree = Tree::read_msg("./scratch/test_tree_read.msg")?;
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

    #[test]
    fn test_tree_stamped_all_builtin_roundtrip() -> Result<(), Box<dyn error::Error>> {
        let s_f64 = Collection::stream(&[-10, 0, 10], &[1.25, 2.5, 3.75]);
        let s_i64 = Collection::stream(&[i64::MIN, 0, i64::MAX], &[-99, 42, 123456789]);
        let s_u64 = Collection::stream(&[1, 2], &[10u64, 20u64]);
        let s_str = Collection::stream(
            &[100, 200],
            &[String::from("alpha,beta"), String::from("{\"k\":\"v\"}")],
        );
        let s_3v = Collection::stream(
            &[1, 2],
            &[ThreeVec::new(1.0, 2.0, 3.0), ThreeVec::new(4.0, 5.0, 6.0)],
        );
        let s_3m = Collection::stream(
            &[1],
            &[ThreeMat::new(
                ThreeVec::new(1.0, 0.0, 0.0),
                ThreeVec::new(0.0, 2.0, 0.0),
                ThreeVec::new(0.0, 0.0, 3.0),
            )],
        );
        let s_4v = Collection::stream(&[5], &[FourVec::new(10.0, 1.0, 2.0, 3.0)]);
        let s_4m = Collection::stream(
            &[6],
            &[FourMat::new(
                FourVec::new(1.0, 0.0, 0.0, 0.0),
                FourVec::new(0.0, 1.0, 0.0, 0.0),
                FourVec::new(0.0, 0.0, 1.0, 0.0),
                FourVec::new(0.0, 0.0, 0.0, 1.0),
            )],
        );
        let s_bin = Collection::stream(&[7], &[Bin::new(0.0, 1.0, 5)]);
        let s_pt = Collection::stream(&[8], &[Point::new(1.5, -2.5)]);
        let s_pbin = Collection::stream(&[9], &[PointBin::new(0.0, 1.0, 0.0, 1.0, 9)]);
        let s_empty: Collection<StampedSample<f64>> = Collection::empty();
        let plain_f64 = Collection::from(vec![9.0, 8.0]);

        let mut ttree = Tree::new("Stamped_Tree");
        ttree.add_field("Desc", "Stamped and plain branches")?;
        assert_eq!(ttree.get_field("Name"), Some("Stamped_Tree"));
        assert_eq!(ttree.get_field("Desc"), Some("Stamped and plain branches"));

        ttree.add_branch("plain_f64", plain_f64.clone(), "f64")?;
        ttree.add_stamped_branch("s_f64", s_f64.clone(), "f64")?;
        ttree.add_stamped_branch("s_i64", s_i64.clone(), "i64")?;
        ttree.add_stamped_branch("s_u64", s_u64.clone(), "u64")?;
        ttree.add_stamped_branch("s_str", s_str.clone(), "String")?;
        ttree.add_stamped_branch("s_3v", s_3v.clone(), "ThreeVec")?;
        ttree.add_stamped_branch("s_3m", s_3m.clone(), "ThreeMat")?;
        ttree.add_stamped_branch("s_4v", s_4v.clone(), "FourVec")?;
        ttree.add_stamped_branch("s_4m", s_4m.clone(), "FourMat")?;
        ttree.add_stamped_branch("s_bin", s_bin.clone(), "Bin")?;
        ttree.add_stamped_branch("s_pt", s_pt.clone(), "Point")?;
        ttree.add_stamped_branch("s_pbin", s_pbin.clone(), "PointBin")?;
        ttree.add_stamped_branch("s_empty", s_empty.clone(), "f64")?;

        ttree.write_msg("./scratch/test_stamped_tree.msg")?;
        ttree.write_json("./scratch/test_stamped_tree.json")?;

        for mut parsed in [
            Tree::read_msg("./scratch/test_stamped_tree.msg")?,
            Tree::read_json("./scratch/test_stamped_tree.json")?,
        ] {
            assert_eq!(parsed.get_field("Name"), Some("Stamped_Tree"));
            assert_eq!(parsed.read_branch::<f64>("plain_f64")?, plain_f64);
            assert!(parsed.read_stamped_branch::<f64>("plain_f64").is_err());
            assert_eq!(parsed.read_stamped_branch::<f64>("s_f64")?, s_f64);
            assert_eq!(parsed.read_branch::<StampedSample<f64>>("s_f64")?, s_f64);
            assert_eq!(parsed.read_stamped_branch::<i64>("s_i64")?, s_i64);
            assert_eq!(parsed.read_stamped_branch::<u64>("s_u64")?, s_u64);
            assert_eq!(parsed.read_stamped_branch::<String>("s_str")?, s_str);
            assert_eq!(parsed.read_stamped_branch::<ThreeVec>("s_3v")?, s_3v);
            assert_eq!(parsed.read_stamped_branch::<ThreeMat>("s_3m")?, s_3m);
            assert_eq!(parsed.read_stamped_branch::<FourVec>("s_4v")?, s_4v);
            assert_eq!(parsed.read_stamped_branch::<FourMat>("s_4m")?, s_4m);
            assert_eq!(parsed.read_stamped_branch::<Bin>("s_bin")?, s_bin);
            assert_eq!(parsed.read_stamped_branch::<Point>("s_pt")?, s_pt);
            assert_eq!(parsed.read_stamped_branch::<PointBin>("s_pbin")?, s_pbin);
            assert_eq!(parsed.read_stamped_branch::<f64>("s_empty")?, s_empty);
        }

        // Empty branches map round-trips cleanly in JSON & MsgPack
        let empty_tree = Tree::new("Empty");
        let empty_json = empty_tree.to_json();
        assert!(Tree::from_json(&empty_json).is_ok());
        let empty_msg = empty_tree.to_msg()?;
        assert!(Tree::from_msg(&empty_msg).is_ok());

        Ok(())
    }

    #[test]
    fn test_tree_stamped_object_branch() -> Result<(), Box<dyn error::Error>> {
        let mut obj_stream: Collection<StampedSample<CustomObj>> = Collection::empty();
        obj_stream.push_sample(1, CustomObj(4.5));

        let mut ttree = Tree::new("ObjTree");
        ttree.add_stamped_branch("objs", obj_stream, "Object")?;
        let json = ttree.to_json();
        let msg = ttree.to_msg()?;

        let err_json = Tree::from_json(&json).err().unwrap();
        assert_eq!(
            err_json.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
        );
        let err_msg = Tree::from_msg(&msg).err().unwrap();
        assert_eq!(
            err_msg.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
        );

        ttree.write_msg("./scratch/test_stamped_obj_tree.msg")?;
        let err_file = Tree::read_msg("./scratch/test_stamped_obj_tree.msg").err().unwrap();
        assert_eq!(
            err_file.downcast_ref::<CalcifyError>().map(|e| format!("{}", e)),
            Some(format!("{}", CalcifyError::ObjectBranchDeserializeError))
        );

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_add_branch_rejects_stamped_sample() {
        let mut ttree = Tree::new("Bad");
        let stream = Collection::stream(&[1], &[2.0]);
        let _ = ttree.add_branch("s", stream, "StampedSample");
    }
}
