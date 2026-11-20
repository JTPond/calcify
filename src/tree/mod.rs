use std::collections::HashMap;
use std::error;

mod branch;

pub use branch::Bin;
pub use branch::Branch;
pub use branch::CalcifyType;
pub use branch::Collection;
pub use branch::Point;
pub use branch::PointBin;
pub use branch::StampedSample;
use branch::STAMPED_SAMPLE_SUBTYPE;

mod feedtree;

pub use feedtree::FeedTree;

use crate::utils;
use utils::errors::CalcifyError;
use utils::io::{FromFile, ToFile};
use utils::serializable::json_object_entries;
use utils::{Deserializable, Serializable};

use rmp::decode::*;
use rmp::encode::*;

/// Tree of Collections for saving to a file.
pub struct Tree {
    metadata: HashMap<String, String>,
    branches: HashMap<String, Branch>,
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
    /// ttree.add_field("Desc", "This is a Tree for testing.").unwrap();
    /// ttree.add_branch("fcol", f_col).unwrap();
    /// ttree.add_branch("col_3v", v3_col).unwrap();
    /// ttree.add_branch("hist_3v", col_hist).unwrap();
    /// ```
    pub fn new(name: &str) -> Tree {
        let mut md = HashMap::new();
        md.insert(String::from("Name"), String::from(name));
        let br = HashMap::new();
        Tree {
            metadata: md,
            branches: br,
        }
    }

    /// Inserts a metadata field without overwriting an existing key.
    pub fn add_field(&mut self, key: &str, f: &str) -> Result<(), CalcifyError> {
        if self.metadata.contains_key(key) {
            return Err(CalcifyError::KeyAlreadyExists(String::from(key)));
        }
        self.metadata.insert(String::from(key), String::from(f));
        Ok(())
    }

    pub fn get_field(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Iterates over `(key, value)` pairs in metadata.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.metadata.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Iterates over metadata field keys.
    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.metadata.keys().map(String::as_str)
    }

    /// Returns the number of metadata fields.
    pub fn field_count(&self) -> usize {
        self.metadata.len()
    }

    /// Returns `true` if `key` is present in metadata.
    pub fn contains_field(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Inserts a new typed built-in branch into `Tree`.
    ///
    /// Returns `CalcifyError::KeyAlreadyExists` without modifying the tree if `key` already exists.
    pub fn add_branch<T: CalcifyType>(
        &mut self,
        key: &str,
        b: Collection<T>,
    ) -> Result<(), CalcifyError> {
        if self.branches.contains_key(key) {
            return Err(CalcifyError::KeyAlreadyExists(String::from(key)));
        }
        self.branches.insert(String::from(key), Branch::new(b));
        Ok(())
    }

    /// Inserts a timestamped stream branch (`Collection<StampedSample<T>>`) into `Tree`.
    ///
    /// Returns `CalcifyError::KeyAlreadyExists` without modifying the tree if `key` already exists.
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, StampedSample, ThreeVec, Tree};
    ///
    /// let stream = Collection::stream(&[1, 2], &[ThreeVec::new(1.0, 0.0, 0.0), ThreeVec::new(0.0, 1.0, 0.0)]);
    /// let mut ttree = Tree::new("Test_Tree");
    /// ttree.add_stamped_branch("positions", stream).unwrap();
    /// let read_back: Collection<StampedSample<ThreeVec>> = ttree.read_stamped_branch("positions").unwrap();
    /// assert_eq!(read_back.len(), 2);
    /// ```
    pub fn add_stamped_branch<T: CalcifyType>(
        &mut self,
        key: &str,
        b: Collection<StampedSample<T>>,
    ) -> Result<(), CalcifyError> {
        if self.branches.contains_key(key) {
            return Err(CalcifyError::KeyAlreadyExists(String::from(key)));
        }
        self.branches
            .insert(String::from(key), Branch::new_stamped(b));
        Ok(())
    }

    /// Inserts a custom `"Object"` branch into `Tree`.
    ///
    /// Returns `CalcifyError::KeyAlreadyExists` without modifying the tree if `key` already exists.
    pub fn add_object_branch<T: 'static + Serializable>(
        &mut self,
        key: &str,
        b: Collection<T>,
    ) -> Result<(), CalcifyError> {
        if self.branches.contains_key(key) {
            return Err(CalcifyError::KeyAlreadyExists(String::from(key)));
        }
        self.branches
            .insert(String::from(key), Branch::new_object(b));
        Ok(())
    }

    /// Inserts a custom `"Object"` timestamped stream branch into `Tree`.
    ///
    /// Returns `CalcifyError::KeyAlreadyExists` without modifying the tree if `key` already exists.
    pub fn add_stamped_object_branch<T: 'static + Serializable>(
        &mut self,
        key: &str,
        b: Collection<StampedSample<T>>,
    ) -> Result<(), CalcifyError> {
        if self.branches.contains_key(key) {
            return Err(CalcifyError::KeyAlreadyExists(String::from(key)));
        }
        self.branches
            .insert(String::from(key), Branch::new_stamped_object(b));
        Ok(())
    }

    /// Returns an immutable reference to a `Branch` in `Tree`.
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
    /// ttree.add_branch("fcol", f_col).unwrap();
    /// ttree.add_branch("bCol", b_col).unwrap();
    ///
    /// let ex_f_col: Collection<f64> = ttree.get_branch("fcol").unwrap().extract().unwrap();
    /// let ex_b_col: Collection<Bin> = ttree.get_branch("bCol").unwrap().extract().unwrap();
    ///
    /// assert_eq!(Collection::from(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]),ex_b_col);
    /// ```
    pub fn get_branch(&self, key: &str) -> Option<&Branch> {
        self.branches.get(key)
    }

    /// Iterates over `(key, &Branch)` pairs in `Tree`.
    pub fn branches(&self) -> impl Iterator<Item = (&str, &Branch)> {
        self.branches.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterates over branch names in `Tree`.
    pub fn branch_names(&self) -> impl Iterator<Item = &str> {
        self.branches.keys().map(String::as_str)
    }

    /// Returns the number of branches in `Tree`.
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Returns `true` if `key` exists as a branch in `Tree`.
    pub fn contains_branch(&self, key: &str) -> bool {
        self.branches.contains_key(key)
    }

    /// Clones and returns a `Collection<T>` branch from `Tree`.
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
    /// ttree.add_branch("fcol", f_col).unwrap();
    /// ttree.add_branch("bCol", b_col).unwrap();
    ///
    /// let ex_f_col: Collection<f64> = ttree.read_branch("fcol").unwrap();
    /// let ex_b_col: Collection<Bin> = ttree.read_branch("bCol").unwrap();
    ///
    /// assert_eq!(Collection::from(vec![0.0,0.0]),ex_f_col);
    /// assert_eq!(Collection::from(vec![Bin::new(0.0,1.0,10),Bin::new(1.0,2.0,10),Bin::new(2.0,3.0,10)]),ex_b_col);
    /// ```
    pub fn read_branch<T: 'static + Serializable + Clone>(
        &self,
        key: &str,
    ) -> Result<Collection<T>, CalcifyError> {
        let branch = self
            .branches
            .get(key)
            .ok_or_else(|| CalcifyError::MissingKey(String::from(key)))?;
        branch.extract::<T>()
    }

    /// Borrows a `Collection<T>` branch from `Tree` without copying.
    pub fn read_branch_ref<T: 'static + Serializable>(
        &self,
        key: &str,
    ) -> Result<&Collection<T>, CalcifyError> {
        let branch = self
            .branches
            .get(key)
            .ok_or_else(|| CalcifyError::MissingKey(String::from(key)))?;
        branch
            .as_collection::<T>()
            .ok_or_else(|| CalcifyError::SubtypeMismatch {
                expected: String::from(std::any::type_name::<T>()),
                found: branch.type_label(),
            })
    }

    /// Clones and returns a timestamped stream branch (`Collection<StampedSample<T>>`) from `Tree`.
    ///
    /// Returns `CalcifyError::MissingKey` if `key` does not exist, or `CalcifyError::SubtypeMismatch`
    /// if the branch is not a `StampedSample` branch or its inner value type does not match `T`.
    pub fn read_stamped_branch<T: 'static + Serializable + Clone>(
        &self,
        key: &str,
    ) -> Result<Collection<StampedSample<T>>, CalcifyError> {
        let branch = self
            .branches
            .get(key)
            .ok_or_else(|| CalcifyError::MissingKey(String::from(key)))?;
        if branch.subtype() != STAMPED_SAMPLE_SUBTYPE {
            return Err(CalcifyError::SubtypeMismatch {
                expected: format!("StampedSample<{}>", std::any::type_name::<T>()),
                found: branch.type_label(),
            });
        }
        branch.extract::<StampedSample<T>>()
    }

    /// Borrows a timestamped stream branch (`Collection<StampedSample<T>>`) from `Tree` without copying.
    pub fn read_stamped_branch_ref<T: 'static + Serializable>(
        &self,
        key: &str,
    ) -> Result<&Collection<StampedSample<T>>, CalcifyError> {
        let branch = self
            .branches
            .get(key)
            .ok_or_else(|| CalcifyError::MissingKey(String::from(key)))?;
        if branch.subtype() != STAMPED_SAMPLE_SUBTYPE {
            return Err(CalcifyError::SubtypeMismatch {
                expected: format!("StampedSample<{}>", std::any::type_name::<T>()),
                found: branch.type_label(),
            });
        }
        branch
            .as_collection::<StampedSample<T>>()
            .ok_or_else(|| CalcifyError::SubtypeMismatch {
                expected: format!("StampedSample<{}>", std::any::type_name::<T>()),
                found: branch.type_label(),
            })
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
        write_map_len(&mut buf, (self.metadata.len() + 1) as u32)?;
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
        for (key, val_str) in json_object_entries(s)? {
            match key.as_str() {
                "branches" => {
                    has_branches = true;
                    for (bkey, bval) in json_object_entries(val_str)? {
                        let branch = Branch::from_json(bval)?;
                        branches.insert(bkey, branch);
                    }
                }
                _ => {
                    let val = String::from_json(val_str)?;
                    metadata.insert(key, val);
                }
            }
        }
        if !has_branches {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "Tree JSON missing 'branches' field",
            ))));
        }
        Ok(Tree { metadata, branches })
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut branches: HashMap<String, Branch> = HashMap::new();
        let mut has_branches = false;
        let len = read_map_len(&mut bytes).map_err(|e| {
            Box::new(CalcifyError::ParseError(format!(
                "invalid MessagePack Tree map: {}",
                e
            ))) as Box<dyn error::Error>
        })?;
        for _ in 0..len {
            let (key, v_rest) = read_str_from_slice(bytes).map_err(|e| {
                Box::new(CalcifyError::ParseError(format!(
                    "invalid MessagePack Tree key: {}",
                    e
                ))) as Box<dyn error::Error>
            })?;
            match key {
                "branches" => {
                    if has_branches {
                        return Err(Box::new(CalcifyError::ParseError(String::from(
                            "duplicate 'branches' key in Tree",
                        ))));
                    }
                    has_branches = true;
                    let mut cur = v_rest;
                    let flen = read_map_len(&mut cur).map_err(|e| {
                        Box::new(CalcifyError::ParseError(format!(
                            "invalid MessagePack branches map: {}",
                            e
                        ))) as Box<dyn error::Error>
                    })?;
                    for _ in 0..flen {
                        let (bkey, bv_rest) = read_str_from_slice(cur).map_err(|e| {
                            Box::new(CalcifyError::ParseError(format!(
                                "invalid MessagePack branch key: {}",
                                e
                            ))) as Box<dyn error::Error>
                        })?;
                        let (value, rest) = Branch::from_msg(bv_rest)?;
                        cur = rest;
                        if branches.insert(String::from(bkey), value).is_some() {
                            return Err(Box::new(CalcifyError::ParseError(format!(
                                "duplicate branch key '{}' in MessagePack Tree",
                                bkey
                            ))));
                        }
                    }
                    bytes = cur;
                }
                _ => {
                    let (value, rest) = String::from_msg(v_rest)?;
                    bytes = rest;
                    if metadata.insert(String::from(key), value).is_some() {
                        return Err(Box::new(CalcifyError::ParseError(format!(
                            "duplicate metadata key '{}' in MessagePack Tree",
                            key
                        ))));
                    }
                }
            }
        }
        if !has_branches {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "MessagePack Tree missing 'branches' field",
            ))));
        }
        Ok((Tree { metadata, branches }, bytes))
    }
}

impl ToFile for Tree {}
impl FromFile for Tree {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bin, FourMat, FourVec, Point, PointBin, ThreeMat, ThreeVec};
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone, PartialEq)]
    struct CustomObj(f64);

    impl Serializable for CustomObj {
        fn to_json(&self) -> String {
            format!("{{\"v\":{}}}", self.0.to_json())
        }
        fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
            self.0.to_msg()
        }
    }

    fn sample_3vecs() -> Collection<ThreeVec> {
        let mut col = Collection::empty();
        for i in 0..9 {
            let v = i as f64;
            col.push(ThreeVec::new(v, v + 1.0, v + 2.0));
        }
        col
    }

    #[test]
    fn test_tree_write() -> Result<(), Box<dyn error::Error>> {
        fs::create_dir_all("./scratch")?;
        let fcol: Collection<f64> = Collection::from(vec![0.0, 0.0]);
        let col_3v = sample_3vecs();
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol)?;
        ttree.add_branch("col_3v", col_3v)?;
        ttree.write_msg("./scratch/test_tree.msg")?;
        Ok(())
    }

    #[test]
    fn test_tree_read() -> Result<(), Box<dyn error::Error>> {
        fs::create_dir_all("./scratch")?;
        let fcol: Collection<f64> = Collection::from(vec![0.0, 0.0]);
        let col_3v = sample_3vecs();
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol.clone())?;
        ttree.add_branch("col_3v", col_3v.clone())?;

        let msg_path = PathBuf::from("./scratch/test_tree_read.msg");
        let json_path = Path::new("./scratch/test_tree_read.json");
        ttree.write_msg(&msg_path)?;
        ttree.write_json(json_path)?;

        let ftree = Tree::read_msg(&msg_path)?;
        let ftree_branch1: Collection<f64> = ftree.read_branch("fcol")?;
        assert_eq!(ftree_branch1, fcol);
        let ftree_branch2: Collection<ThreeVec> = ftree.read_branch("col_3v")?;
        assert_eq!(ftree_branch2, col_3v);

        let jtree = Tree::read_json(json_path)?;
        assert_eq!(jtree.read_branch_ref::<f64>("fcol")?, &fcol);
        assert_eq!(jtree.read_branch_ref::<ThreeVec>("col_3v")?, &col_3v);
        Ok(())
    }

    #[test]
    fn test_tree_json() -> Result<(), Box<dyn error::Error>> {
        let fcol: Collection<f64> = Collection::from(vec![0.0, 0.0]);
        let col_3v = sample_3vecs();
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("col_3v", col_3v.clone())?;
        ttree.add_branch("fcol", fcol)?;
        ttree.add_branch("col_3v3", col_3v)?;
        let pp = ttree.to_json();
        let oo = Tree::from_json(&pp)?;
        assert_eq!(oo.read_branch::<f64>("fcol")?, Collection::from(vec![0.0, 0.0]));
        Ok(())
    }

    #[test]
    fn test_tree_msg() -> Result<(), Box<dyn error::Error>> {
        let fcol: Collection<f64> = Collection::from(vec![0.0, 0.0]);
        let col_3v = sample_3vecs();
        let mut ttree = Tree::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_branch("fcol", fcol)?;
        ttree.add_branch("col_3v", col_3v)?;
        let pp = ttree.to_msg()?;
        let (oo, _) = Tree::from_msg(&pp)?;
        assert_eq!(oo.read_branch::<f64>("fcol")?, Collection::from(vec![0.0, 0.0]));
        Ok(())
    }

    #[test]
    fn test_tree_immutability_errors_and_introspection() {
        let mut tree = Tree::new("Sim");
        tree.add_field("Author", "Alice").unwrap();
        assert_eq!(
            tree.add_field("Author", "Bob"),
            Err(CalcifyError::KeyAlreadyExists(String::from("Author")))
        );
        // Existing field must not be overwritten on error
        assert_eq!(tree.get_field("Author"), Some("Alice"));
        assert_eq!(
            tree.add_field("Name", "Other"),
            Err(CalcifyError::KeyAlreadyExists(String::from("Name")))
        );
        assert_eq!(tree.get_field("Name"), Some("Sim"));

        let c1 = Collection::from(vec![1.0, 2.0]);
        let c2 = Collection::from(vec![99.0]);
        tree.add_branch("energy", c1.clone()).unwrap();
        assert_eq!(
            tree.add_branch("energy", c2),
            Err(CalcifyError::KeyAlreadyExists(String::from("energy")))
        );
        assert_eq!(tree.read_branch::<f64>("energy").unwrap(), c1);

        let stream1 = Collection::stream(&[1, 2], &[10i64, 20]);
        let stream2 = Collection::stream(&[3], &[30i64]);
        tree.add_stamped_branch("ticks", stream1.clone()).unwrap();
        assert_eq!(
            tree.add_stamped_branch("ticks", stream2),
            Err(CalcifyError::KeyAlreadyExists(String::from("ticks")))
        );
        assert_eq!(
            tree.read_stamped_branch::<i64>("ticks").unwrap(),
            stream1
        );

        let obj_col = Collection::from(vec![CustomObj(7.5)]);
        tree.add_object_branch("objs", obj_col.clone()).unwrap();

        let stamped_obj = Collection::stream(&[5], &[CustomObj(9.5)]);
        tree.add_stamped_object_branch("stamped_objs", stamped_obj.clone())
            .unwrap();

        // Immutable &Tree reads including zero-copy ref accessors
        let ro: &Tree = &tree;
        assert_eq!(ro.read_branch_ref::<f64>("energy").unwrap(), &c1);
        assert_eq!(
            ro.read_stamped_branch_ref::<i64>("ticks").unwrap(),
            &stream1
        );
        assert_eq!(ro.read_branch_ref::<CustomObj>("objs").unwrap(), &obj_col);
        assert_eq!(
            ro.read_branch::<CustomObj>("objs").unwrap(),
            obj_col
        );
        assert_eq!(
            ro.read_stamped_branch_ref::<CustomObj>("stamped_objs")
                .unwrap(),
            &stamped_obj
        );

        // MissingKey & SubtypeMismatch
        assert_eq!(
            ro.read_branch::<f64>("missing"),
            Err(CalcifyError::MissingKey(String::from("missing")))
        );
        assert_eq!(
            ro.read_stamped_branch::<f64>("missing"),
            Err(CalcifyError::MissingKey(String::from("missing")))
        );
        assert!(matches!(
            ro.read_branch::<i64>("energy"),
            Err(CalcifyError::SubtypeMismatch { .. })
        ));
        assert!(matches!(
            ro.read_stamped_branch::<f64>("energy"),
            Err(CalcifyError::SubtypeMismatch { .. })
        ));
        assert!(matches!(
            ro.read_stamped_branch::<f64>("ticks"),
            Err(CalcifyError::SubtypeMismatch { .. })
        ));

        // Introspection
        assert_eq!(ro.field_count(), 2);
        assert!(ro.contains_field("Name"));
        assert!(ro.contains_field("Author"));
        assert!(!ro.contains_field("Missing"));
        let field_names: HashSet<&str> = ro.field_names().collect();
        assert_eq!(field_names, HashSet::from(["Name", "Author"]));
        assert_eq!(ro.fields().count(), 2);

        assert_eq!(ro.branch_count(), 4);
        assert!(ro.contains_branch("energy"));
        assert!(!ro.contains_branch("missing"));
        let bnames: HashSet<&str> = ro.branch_names().collect();
        assert_eq!(
            bnames,
            HashSet::from(["energy", "ticks", "objs", "stamped_objs"])
        );
        assert_eq!(ro.branches().count(), 4);
        let energy_br = ro.get_branch("energy").unwrap();
        assert_eq!(energy_br.subtype(), "f64");
        assert_eq!(energy_br.value_subtype(), None);
        assert_eq!(energy_br.len(), 2);
        assert!(!energy_br.is_empty());
    }

    #[test]
    fn test_tree_stamped_all_builtin_roundtrip() -> Result<(), Box<dyn error::Error>> {
        fs::create_dir_all("./scratch")?;
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

        ttree.add_branch("plain_f64", plain_f64.clone())?;
        ttree.add_stamped_branch("s_f64", s_f64.clone())?;
        ttree.add_stamped_branch("s_i64", s_i64.clone())?;
        ttree.add_stamped_branch("s_u64", s_u64.clone())?;
        ttree.add_stamped_branch("s_str", s_str.clone())?;
        ttree.add_stamped_branch("s_3v", s_3v.clone())?;
        ttree.add_stamped_branch("s_3m", s_3m.clone())?;
        ttree.add_stamped_branch("s_4v", s_4v.clone())?;
        ttree.add_stamped_branch("s_4m", s_4m.clone())?;
        ttree.add_stamped_branch("s_bin", s_bin.clone())?;
        ttree.add_stamped_branch("s_pt", s_pt.clone())?;
        ttree.add_stamped_branch("s_pbin", s_pbin.clone())?;
        ttree.add_stamped_branch("s_empty", s_empty.clone())?;

        ttree.write_msg("./scratch/test_stamped_tree.msg")?;
        ttree.write_json("./scratch/test_stamped_tree.json")?;

        for parsed in [
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
        fs::create_dir_all("./scratch")?;
        let mut obj_stream: Collection<StampedSample<CustomObj>> = Collection::empty();
        obj_stream.push_sample(1, CustomObj(4.5));

        let mut ttree = Tree::new("ObjTree");
        ttree.add_stamped_object_branch("objs", obj_stream)?;
        let json = ttree.to_json();
        let msg = ttree.to_msg()?;

        let err_json = Tree::from_json(&json).err().unwrap();
        assert_eq!(
            err_json.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );
        let err_msg = Tree::from_msg(&msg).err().unwrap();
        assert_eq!(
            err_msg.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );

        ttree.write_msg("./scratch/test_stamped_obj_tree.msg")?;
        let err_file = Tree::read_msg("./scratch/test_stamped_obj_tree.msg")
            .err()
            .unwrap();
        assert_eq!(
            err_file.downcast_ref::<CalcifyError>(),
            Some(&CalcifyError::ObjectBranchDeserializeError)
        );

        Ok(())
    }
}
