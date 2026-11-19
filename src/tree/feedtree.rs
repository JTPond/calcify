use std::collections::HashMap;
use std::error;
use super::{Collection, StampedSample};

use crate::utils;
use utils::{Serializable, Deserializable};
use utils::errors::CalcifyError;
use utils::io::{ToFile,FromFile};
use utils::serializable::split_json_object;

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
        if self.metadata.insert(String::from(key),String::from(f)).is_some() {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    pub fn get_field(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    pub fn subtype(&self) -> Option<&str> {
        self.get_field("SubType")
    }

    pub fn value_subtype(&self) -> Option<&str> {
        self.get_field("ValueSubType")
    }

    /// Inserts new Collection<T> into FeedTree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, String
    /// * `f` - Collection<T: Serializable>
    pub fn add_feed(&mut self, key: &str, f: Collection<T>) -> Result<(),CalcifyError> {
        if self.datafeeds.insert(String::from(key),f).is_some() {
            return Err(CalcifyError::KeyError);
        }
        Ok(())
    }

    pub fn get_feed(&mut self, key: &str) -> Option<&Collection<T>> {
        self.datafeeds.get(key)
    }

    pub fn get_feed_mut(&mut self, key: &str) -> Option<&mut Collection<T>> {
        self.datafeeds.get_mut(key)
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

impl<T: Serializable> FeedTree<StampedSample<T>> {
    /// Creates a `FeedTree` configured for timestamped streams (`"SubType": "StampedSample"`).
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, FeedTree, StampedSample};
    ///
    /// let mut ftree: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("Sensor_Log", "f64");
    /// ftree.add_stream("temp").unwrap();
    /// ftree.write_sample("temp", 1000, 22.5).unwrap();
    /// ftree.write_samples("temp", vec![StampedSample::new(1001, 22.7)]).unwrap();
    /// assert_eq!(ftree.subtype(), Some("StampedSample"));
    /// assert_eq!(ftree.value_subtype(), Some("f64"));
    /// assert_eq!(ftree.get_feed("temp").unwrap().len(), 2);
    /// ```
    pub fn new_stamped(name: &str, value_subtype: &str) -> FeedTree<StampedSample<T>> {
        let mut md = HashMap::new();
        md.insert(String::from("Name"), String::from(name));
        md.insert(String::from("SubType"), String::from("StampedSample"));
        md.insert(String::from("ValueSubType"), String::from(value_subtype));
        FeedTree {
            metadata: md,
            datafeeds: HashMap::new(),
        }
    }

    /// Inserts a new empty `Collection<StampedSample<T>>` feed under `key`.
    pub fn add_stream(&mut self, key: &str) -> Result<(), CalcifyError> {
        self.add_feed(key, Collection::empty())
    }

    /// Appends a single timestamped sample `(timestamp, value)` to the stream at `key`.
    pub fn write_sample(&mut self, key: &str, timestamp: i64, value: T) -> Result<(), CalcifyError> {
        self.write(key, StampedSample::new(timestamp, value))
    }

    /// Appends multiple `StampedSample<T>` items to the stream at `key`.
    pub fn write_samples<I: IntoIterator<Item = StampedSample<T>>>(
        &mut self,
        key: &str,
        samples: I,
    ) -> Result<(), CalcifyError> {
        if let Some(feed) = self.datafeeds.get_mut(key) {
            feed.extend(samples);
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
            out.push_str(&format!("{}:{},", key.to_json(), val.to_json()));
        }
        out.push_str("\"datafeeds\":{");
        let mut first = true;
        for (key, val) in &self.datafeeds {
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
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut datafeeds: HashMap<String, Collection<T>> = HashMap::new();
        let mut has_datafeeds = false;
        for (key, val_str) in split_json_object(s)? {
            match key.as_str() {
                "datafeeds" => {
                    if has_datafeeds {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    has_datafeeds = true;
                    for (fkey, fval) in split_json_object(val_str)? {
                        let feed = Collection::<T>::from_json(fval)?;
                        if datafeeds.insert(fkey, feed).is_some() {
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
        if !has_datafeeds {
            return Err(Box::new(CalcifyError::ParseError));
        }
        Ok(FeedTree {
            metadata,
            datafeeds,
        })
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        let mut metadata: HashMap<String,String> = HashMap::new();
        let mut datafeeds: HashMap<String,Collection<T>> = HashMap::new();
        let mut has_datafeeds = false;
        let len = read_map_len(&mut bytes)
            .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
        for _ in 0..len {
            let (key, v_rest) = read_str_from_slice(bytes)
                .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
            match key {
                "datafeeds" => {
                    if has_datafeeds {
                        return Err(Box::new(CalcifyError::ParseError));
                    }
                    has_datafeeds = true;
                    let mut cur = v_rest;
                    let flen = read_map_len(&mut cur)
                        .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
                    for _ in 0..flen {
                        let (fkey, fv_rest) = read_str_from_slice(cur)
                            .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
                        let (value, rest) = Collection::<T>::from_msg(fv_rest)?;
                        cur = rest;
                        if datafeeds.insert(String::from(fkey), value).is_some() {
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
        if !has_datafeeds {
            return Err(Box::new(CalcifyError::ParseError));
        }
        Ok((FeedTree { metadata, datafeeds }, bytes))
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
        ttree.write_msg("./scratch/test_ftree.msg")?;
        Ok(())
    }

    #[test]
    fn test_ftree_read() -> Result<(),Box<dyn error::Error>>{
        let mut col_3v: Collection<ThreeVec> = Collection::empty();
        for _i in 0..9 {col_3v.push(ThreeVec::new(1.0,2.0,3.0));}
        let mut ttree = FeedTree::new("Test_Tree","ThreeVec");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v)?;
        ttree.write_msg("./scratch/test_ftree_read.msg")?;

        let ftree: FeedTree<ThreeVec> = FeedTree::read_msg("./scratch/test_ftree_read.msg")?;
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

    #[derive(Debug, PartialEq, Clone)]
    struct CustomEvent {
        id: u64,
        reading: f64,
    }

    impl Serializable for CustomEvent {
        fn to_json(&self) -> String {
            format!("{{\"id\":{},\"reading\":{}}}", self.id, self.reading)
        }
        fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
            let mut buf = Vec::new();
            write_array_len(&mut buf, 2)?;
            write_uint(&mut buf, self.id)?;
            write_f64(&mut buf, self.reading)?;
            Ok(buf)
        }
    }

    impl Deserializable for CustomEvent {
        fn from_json(s: &str) -> Result<Self, Box<dyn error::Error>> {
            let mut id = None;
            let mut reading = None;
            for (k, v) in split_json_object(s)? {
                match k.as_str() {
                    "id" => id = Some(u64::from_json(v)?),
                    "reading" => reading = Some(f64::from_json(v)?),
                    _ => return Err(Box::new(CalcifyError::ParseError)),
                }
            }
            match (id, reading) {
                (Some(id), Some(reading)) => Ok(CustomEvent { id, reading }),
                _ => Err(Box::new(CalcifyError::ParseError)),
            }
        }

        fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
            if let Ok(2) = read_array_len(&mut bytes) {
                let id = read_int(&mut bytes)?;
                let reading = read_f64(&mut bytes)?;
                Ok((CustomEvent { id, reading }, bytes))
            } else {
                Err(Box::new(CalcifyError::ParseError))
            }
        }
    }

    #[test]
    fn test_ftree_stamped_builtins_and_helpers() -> Result<(), Box<dyn error::Error>> {
        // f64 stream with empty feed & KeyError checks
        let mut ftree_f64: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("Scalar_Feed", "f64");
        assert_eq!(ftree_f64.get_field("Name"), Some("Scalar_Feed"));
        assert_eq!(ftree_f64.subtype(), Some("StampedSample"));
        assert_eq!(ftree_f64.value_subtype(), Some("f64"));

        assert!(ftree_f64.write_sample("missing", 1, 0.0).is_err());
        assert!(
            ftree_f64
                .write_samples("missing", vec![StampedSample::new(1, 0.0)])
                .is_err()
        );

        ftree_f64.add_stream("empty_stream")?;
        ftree_f64.add_stream("temps")?;
        assert!(ftree_f64.add_stream("temps").is_err());
        ftree_f64.write_sample("temps", 100, 21.5)?;
        ftree_f64.write_samples(
            "temps",
            vec![
                StampedSample::new(101, 22.0),
                StampedSample::new(102, 22.25),
            ],
        )?;
        assert_eq!(ftree_f64.get_feed("temps").unwrap().len(), 3);
        ftree_f64.get_feed_mut("temps").unwrap().push_sample(103, 23.0);

        ftree_f64.write_msg("./scratch/test_stamped_ftree_f64.msg")?;
        ftree_f64.write_json("./scratch/test_stamped_ftree_f64.json")?;
        assert_eq!(
            FeedTree::<StampedSample<f64>>::read_msg("./scratch/test_stamped_ftree_f64.msg")?,
            ftree_f64
        );
        assert_eq!(
            FeedTree::<StampedSample<f64>>::read_json("./scratch/test_stamped_ftree_f64.json")?,
            ftree_f64
        );

        // ThreeVec stream
        let mut ftree_3v: FeedTree<StampedSample<ThreeVec>> =
            FeedTree::new_stamped("Vec_Feed", "ThreeVec");
        ftree_3v.add_stream("pos")?;
        ftree_3v.write_sample("pos", -5, ThreeVec::new(1.0, 2.0, 3.0))?;
        ftree_3v.write_sample("pos", 5, ThreeVec::new(4.0, 5.0, 6.0))?;
        assert_eq!(
            FeedTree::<StampedSample<ThreeVec>>::from_json(&ftree_3v.to_json())?,
            ftree_3v
        );
        let raw_3v = ftree_3v.to_msg()?;
        let (msg_3v, rem_3v) =
            FeedTree::<StampedSample<ThreeVec>>::from_msg(&raw_3v)?;
        assert!(rem_3v.is_empty());
        assert_eq!(msg_3v, ftree_3v);

        // String stream
        let mut ftree_str: FeedTree<StampedSample<String>> =
            FeedTree::new_stamped("Str_Feed", "String");
        ftree_str.add_stream("logs")?;
        ftree_str.write_sample("logs", 10, String::from("ok, {done}"))?;
        ftree_str.write_sample("logs", 20, String::from("quote \"here\""))?;
        assert_eq!(
            FeedTree::<StampedSample<String>>::from_json(&ftree_str.to_json())?,
            ftree_str
        );
        let raw_str = ftree_str.to_msg()?;
        let (msg_str, rem_str) =
            FeedTree::<StampedSample<String>>::from_msg(&raw_str)?;
        assert!(rem_str.is_empty());
        assert_eq!(msg_str, ftree_str);

        // Empty FeedTree (no feeds yet)
        let empty_ft: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("Empty", "f64");
        assert_eq!(
            FeedTree::<StampedSample<f64>>::from_json(&empty_ft.to_json())?,
            empty_ft
        );
        let raw_empty = empty_ft.to_msg()?;
        let (msg_empty, rem_empty) = FeedTree::<StampedSample<f64>>::from_msg(&raw_empty)?;
        assert!(rem_empty.is_empty());
        assert_eq!(msg_empty, empty_ft);

        Ok(())
    }

    #[test]
    fn test_ftree_stamped_custom_object() -> Result<(), Box<dyn error::Error>> {
        let mut ftree: FeedTree<StampedSample<CustomEvent>> =
            FeedTree::new_stamped("Events", "Object");
        ftree.add_stream("ev")?;
        ftree.write_sample("ev", 1_700_000_000, CustomEvent { id: 1, reading: 9.5 })?;
        ftree.write_sample("ev", 1_700_000_001, CustomEvent { id: 2, reading: 10.5 })?;

        assert_eq!(ftree.subtype(), Some("StampedSample"));
        assert_eq!(ftree.value_subtype(), Some("Object"));

        let json_rt = FeedTree::<StampedSample<CustomEvent>>::from_json(&ftree.to_json())?;
        assert_eq!(json_rt, ftree);

        ftree.write_msg("./scratch/test_stamped_ftree_obj.msg")?;
        let file_rt =
            FeedTree::<StampedSample<CustomEvent>>::read_msg("./scratch/test_stamped_ftree_obj.msg")?;
        assert_eq!(file_rt, ftree);

        Ok(())
    }
}
