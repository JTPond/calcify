use std::collections::HashMap;
use std::error;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use super::branch::{OBJECT_SUBTYPE, STAMPED_SAMPLE_SUBTYPE};
use super::{CalcifyType, Collection, StampedSample};

use crate::utils;
use utils::errors::CalcifyError;
use utils::io::{FromFile, ToFile};
use utils::serializable::json_object_entries;
use utils::{Deserializable, Serializable};

use rmp::decode::*;
use rmp::encode::*;

/// Tree of Collections of only a single type, which impl the Feed trait for added functionality
#[derive(Debug, PartialEq, Clone)]
pub struct FeedTree<T: Serializable> {
    metadata: HashMap<String, String>,
    datafeeds: HashMap<String, Collection<T>>,
}

impl<T: CalcifyType> FeedTree<T> {
    /// Returns a new typed `FeedTree<T>` whose `"SubType"` metadata is inferred from `T`.
    ///
    /// # Example
    /// ```
    /// use calcify::FeedTree;
    /// use calcify::Collection;
    ///
    /// let f_col: Collection<f64> = Collection::from(vec![0.0,0.0]);
    /// let mut ftree: FeedTree<f64> = FeedTree::new("Test_Tree");
    /// ftree.add_field("Desc", "This is a FeedTree for testing.").unwrap();
    /// ftree.add_feed("fcol", f_col).unwrap();
    /// ftree.write("fcol", 1.0).unwrap();
    /// ftree.write("fcol", 2.0).unwrap();
    /// assert_eq!(Collection::from(vec![0.0,0.0,1.0,2.0]),*ftree.get_feed("fcol").unwrap())
    /// ```
    pub fn new(name: &str) -> FeedTree<T> {
        Self::new_with_subtype(name, T::SUBTYPE)
    }
}

impl<T: Serializable> FeedTree<T> {
    /// Returns a new `"Object"` `FeedTree<T>` for custom user structs.
    pub fn new_object(name: &str) -> FeedTree<T> {
        Self::new_with_subtype(name, OBJECT_SUBTYPE)
    }

    /// Returns a new `FeedTree<T>` with an explicit `"SubType"` metadata string.
    pub fn new_with_subtype(name: &str, subtype: &str) -> FeedTree<T> {
        let mut md = HashMap::new();
        md.insert(String::from("Name"), String::from(name));
        md.insert(String::from("SubType"), String::from(subtype));
        let df = HashMap::new();
        FeedTree {
            metadata: md,
            datafeeds: df,
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

    pub fn subtype(&self) -> Option<&str> {
        self.get_field("SubType")
    }

    pub fn value_subtype(&self) -> Option<&str> {
        self.get_field("ValueSubType")
    }

    /// Inserts a new `Collection<T>` into `FeedTree` without overwriting an existing key.
    pub fn add_feed(&mut self, key: &str, f: Collection<T>) -> Result<(), CalcifyError> {
        if self.datafeeds.contains_key(key) {
            return Err(CalcifyError::KeyAlreadyExists(String::from(key)));
        }
        self.datafeeds.insert(String::from(key), f);
        Ok(())
    }

    pub fn get_feed(&self, key: &str) -> Option<&Collection<T>> {
        self.datafeeds.get(key)
    }

    pub fn get_feed_mut(&mut self, key: &str) -> Option<&mut Collection<T>> {
        self.datafeeds.get_mut(key)
    }

    /// Iterates over `(key, &Collection<T>)` pairs in `FeedTree`.
    pub fn feeds(&self) -> impl Iterator<Item = (&str, &Collection<T>)> {
        self.datafeeds.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Iterates over datafeed names in `FeedTree`.
    pub fn feed_names(&self) -> impl Iterator<Item = &str> {
        self.datafeeds.keys().map(String::as_str)
    }

    /// Returns the number of datafeeds in `FeedTree`.
    pub fn feed_count(&self) -> usize {
        self.datafeeds.len()
    }

    /// Returns `true` if `key` exists as a datafeed in `FeedTree`.
    pub fn contains_feed(&self, key: &str) -> bool {
        self.datafeeds.contains_key(key)
    }

    pub fn write(&mut self, key: &str, data: T) -> Result<(), CalcifyError> {
        if let Some(feed) = self.datafeeds.get_mut(key) {
            feed.push(data);
            Ok(())
        } else {
            Err(CalcifyError::MissingKey(String::from(key)))
        }
    }
}

impl<T: CalcifyType> FeedTree<StampedSample<T>> {
    /// Creates a typed `FeedTree` configured for timestamped streams (`"SubType": "StampedSample"`).
    ///
    /// # Example
    /// ```
    /// use calcify::{Collection, FeedTree, StampedSample};
    ///
    /// let mut ftree: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("Sensor_Log");
    /// ftree.add_stream("temp").unwrap();
    /// ftree.write_sample("temp", 1000, 22.5).unwrap();
    /// ftree.write_samples("temp", vec![StampedSample::new(1001, 22.7)]).unwrap();
    /// assert_eq!(ftree.subtype(), Some("StampedSample"));
    /// assert_eq!(ftree.value_subtype(), Some("f64"));
    /// assert_eq!(ftree.get_feed("temp").unwrap().len(), 2);
    /// ```
    pub fn new_stamped(name: &str) -> FeedTree<StampedSample<T>> {
        Self::new_stamped_with_subtype(name, T::SUBTYPE)
    }
}

impl<T: Serializable> FeedTree<StampedSample<T>> {
    /// Creates an `"Object"` `FeedTree` configured for timestamped custom structs.
    pub fn new_stamped_object(name: &str) -> FeedTree<StampedSample<T>> {
        Self::new_stamped_with_subtype(name, OBJECT_SUBTYPE)
    }

    fn new_stamped_with_subtype(name: &str, value_subtype: &str) -> FeedTree<StampedSample<T>> {
        let mut md = HashMap::new();
        md.insert(String::from("Name"), String::from(name));
        md.insert(String::from("SubType"), String::from(STAMPED_SAMPLE_SUBTYPE));
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
    pub fn write_sample(
        &mut self,
        key: &str,
        timestamp: i64,
        value: T,
    ) -> Result<(), CalcifyError> {
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
            Err(CalcifyError::MissingKey(String::from(key)))
        }
    }
}

struct NoDupMapSeed<V>(PhantomData<V>);

impl<'de, V: Deserialize<'de>> DeserializeSeed<'de> for NoDupMapSeed<V> {
    type Value = HashMap<String, V>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct NoDupMapVisitor<V>(PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for NoDupMapVisitor<V> {
            type Value = HashMap<String, V>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map without duplicate keys")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut out = HashMap::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, val)) = map.next_entry::<String, V>()? {
                    if out.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate key `{}`", key)));
                    }
                    out.insert(key, val);
                }
                Ok(out)
            }
        }

        deserializer.deserialize_map(NoDupMapVisitor(PhantomData))
    }
}

impl<T: Serializable + Serialize> Serialize for FeedTree<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.metadata.len() + 1))?;
        for (key, val) in &self.metadata {
            map.serialize_entry(key, val)?;
        }
        map.serialize_entry("datafeeds", &self.datafeeds)?;
        map.end()
    }
}

impl<'de, T: Serializable + Deserialize<'de>> Deserialize<'de> for FeedTree<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FeedTreeVisitor<T>(PhantomData<T>);

        impl<'de, T: Serializable + Deserialize<'de>> Visitor<'de> for FeedTreeVisitor<T> {
            type Value = FeedTree<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a FeedTree JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<FeedTree<T>, A::Error> {
                let mut metadata: HashMap<String, String> = HashMap::new();
                let mut datafeeds: Option<HashMap<String, Collection<T>>> = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "datafeeds" {
                        if datafeeds.is_some() {
                            return Err(de::Error::duplicate_field("datafeeds"));
                        }
                        datafeeds = Some(
                            map.next_value_seed(NoDupMapSeed::<Collection<T>>(PhantomData))?,
                        );
                    } else {
                        if metadata.contains_key(&key) {
                            return Err(de::Error::custom(format!(
                                "duplicate metadata key `{}`",
                                key
                            )));
                        }
                        let val = map.next_value::<String>()?;
                        metadata.insert(key, val);
                    }
                }
                let datafeeds = datafeeds.ok_or_else(|| de::Error::missing_field("datafeeds"))?;
                Ok(FeedTree {
                    metadata,
                    datafeeds,
                })
            }
        }

        deserializer.deserialize_map(FeedTreeVisitor(PhantomData))
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
        write_map_len(&mut buf, (self.metadata.len() + 1) as u32)?;
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
        for (key, val_str) in json_object_entries(s)? {
            match key.as_str() {
                "datafeeds" => {
                    has_datafeeds = true;
                    for (fkey, fval) in json_object_entries(val_str)? {
                        let feed = Collection::<T>::from_json(fval)?;
                        datafeeds.insert(fkey, feed);
                    }
                }
                _ => {
                    let val = String::from_json(val_str)?;
                    metadata.insert(key, val);
                }
            }
        }
        if !has_datafeeds {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "FeedTree JSON missing 'datafeeds' field",
            ))));
        }
        Ok(FeedTree {
            metadata,
            datafeeds,
        })
    }

    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut datafeeds: HashMap<String, Collection<T>> = HashMap::new();
        let mut has_datafeeds = false;
        let len = read_map_len(&mut bytes).map_err(|e| {
            Box::new(CalcifyError::ParseError(format!(
                "invalid MessagePack FeedTree map: {}",
                e
            ))) as Box<dyn error::Error>
        })?;
        for _ in 0..len {
            let (key, v_rest) = read_str_from_slice(bytes).map_err(|e| {
                Box::new(CalcifyError::ParseError(format!(
                    "invalid MessagePack FeedTree key: {}",
                    e
                ))) as Box<dyn error::Error>
            })?;
            match key {
                "datafeeds" => {
                    if has_datafeeds {
                        return Err(Box::new(CalcifyError::ParseError(String::from(
                            "duplicate 'datafeeds' key in FeedTree",
                        ))));
                    }
                    has_datafeeds = true;
                    let mut cur = v_rest;
                    let flen = read_map_len(&mut cur).map_err(|e| {
                        Box::new(CalcifyError::ParseError(format!(
                            "invalid MessagePack datafeeds map: {}",
                            e
                        ))) as Box<dyn error::Error>
                    })?;
                    for _ in 0..flen {
                        let (fkey, fv_rest) = read_str_from_slice(cur).map_err(|e| {
                            Box::new(CalcifyError::ParseError(format!(
                                "invalid MessagePack feed key: {}",
                                e
                            ))) as Box<dyn error::Error>
                        })?;
                        let (value, rest) = Collection::<T>::from_msg(fv_rest)?;
                        cur = rest;
                        if datafeeds.insert(String::from(fkey), value).is_some() {
                            return Err(Box::new(CalcifyError::ParseError(format!(
                                "duplicate feed key '{}' in MessagePack FeedTree",
                                fkey
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
                            "duplicate metadata key '{}' in MessagePack FeedTree",
                            key
                        ))));
                    }
                }
            }
        }
        if !has_datafeeds {
            return Err(Box::new(CalcifyError::ParseError(String::from(
                "MessagePack FeedTree missing 'datafeeds' field",
            ))));
        }
        Ok((
            FeedTree {
                metadata,
                datafeeds,
            },
            bytes,
        ))
    }
}

impl<T: Serializable> ToFile for FeedTree<T> {}
impl<T: Serializable + Deserializable> FromFile for FeedTree<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreeVec;
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;

    fn sample_3vecs() -> Collection<ThreeVec> {
        let mut col = Collection::empty();
        for i in 0..9 {
            let v = i as f64;
            col.push(ThreeVec::new(v, v + 1.0, v + 2.0));
        }
        col
    }

    #[test]
    fn test_ftree_write() -> Result<(), Box<dyn error::Error>> {
        fs::create_dir_all("./scratch")?;
        let col_3v = sample_3vecs();
        let mut ttree = FeedTree::<ThreeVec>::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v)?;
        ttree.write_msg("./scratch/test_ftree.msg")?;
        Ok(())
    }

    #[test]
    fn test_ftree_read() -> Result<(), Box<dyn error::Error>> {
        fs::create_dir_all("./scratch")?;
        let col_3v = sample_3vecs();
        let mut ttree = FeedTree::<ThreeVec>::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v)?;
        let path = PathBuf::from("./scratch/test_ftree_read.msg");
        ttree.write_msg(&path)?;

        let ftree: FeedTree<ThreeVec> = FeedTree::read_msg(&path)?;
        assert_eq!(ftree, ttree);
        Ok(())
    }

    #[test]
    fn test_ftree_json() -> Result<(), Box<dyn error::Error>> {
        let col_3v = sample_3vecs();
        let mut ttree = FeedTree::<ThreeVec>::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v.clone())?;
        ttree.add_feed("fcol1", col_3v.clone())?;
        ttree.add_feed("fcol2", col_3v)?;
        let pp = ttree.to_json();
        let oo = FeedTree::<ThreeVec>::from_json(&pp)?;
        assert_eq!(oo, ttree);

        let serde_rt: FeedTree<ThreeVec> =
            serde_json::from_str(&serde_json::to_string(&ttree)?)?;
        assert_eq!(serde_rt, ttree);
        Ok(())
    }

    #[test]
    fn test_ftree_msg() -> Result<(), Box<dyn error::Error>> {
        let col_3v = sample_3vecs();
        let mut ttree = FeedTree::<ThreeVec>::new("Test_Tree");
        ttree.add_field("Desc", "This is a Tree for testing.")?;
        ttree.add_feed("fcol", col_3v.clone())?;
        ttree.add_feed("fcol2", col_3v)?;
        let pp = ttree.to_msg()?;
        let (oo, _) = FeedTree::<ThreeVec>::from_msg(&pp)?;
        assert_eq!(oo, ttree);
        Ok(())
    }

    #[test]
    fn test_ftree_immutability_errors_and_introspection() {
        let mut ftree = FeedTree::<f64>::new("Metrics");
        ftree.add_field("Unit", "MeV").unwrap();
        assert_eq!(
            ftree.add_field("Unit", "GeV"),
            Err(CalcifyError::KeyAlreadyExists(String::from("Unit")))
        );
        assert_eq!(ftree.get_field("Unit"), Some("MeV"));

        let f1 = Collection::from(vec![1.0, 2.0]);
        let f2 = Collection::from(vec![99.0]);
        ftree.add_feed("a", f1.clone()).unwrap();
        assert_eq!(
            ftree.add_feed("a", f2),
            Err(CalcifyError::KeyAlreadyExists(String::from("a")))
        );
        assert_eq!(ftree.get_feed("a"), Some(&f1));

        ftree.add_feed("b", Collection::from(vec![3.0])).unwrap();
        assert_eq!(
            ftree.write("missing", 5.0),
            Err(CalcifyError::MissingKey(String::from("missing")))
        );

        let ro: &FeedTree<f64> = &ftree;
        assert_eq!(ro.subtype(), Some("f64"));
        assert_eq!(ro.get_feed("a"), Some(&f1));
        assert_eq!(ro.field_count(), 3);
        assert!(ro.contains_field("Name"));
        assert!(ro.contains_field("SubType"));
        assert!(ro.contains_field("Unit"));
        assert!(!ro.contains_field("Missing"));
        let fnames: HashSet<&str> = ro.field_names().collect();
        assert_eq!(fnames, HashSet::from(["Name", "SubType", "Unit"]));
        assert_eq!(ro.fields().count(), 3);

        assert_eq!(ro.feed_count(), 2);
        assert!(ro.contains_feed("a"));
        assert!(!ro.contains_feed("c"));
        let dnames: HashSet<&str> = ro.feed_names().collect();
        assert_eq!(dnames, HashSet::from(["a", "b"]));
        assert_eq!(ro.feeds().count(), 2);
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
            for (k, v) in json_object_entries(s)? {
                match k.as_str() {
                    "id" => id = Some(u64::from_json(v)?),
                    "reading" => reading = Some(f64::from_json(v)?),
                    _ => {
                        return Err(Box::new(CalcifyError::ParseError(format!(
                            "unexpected CustomEvent field '{}'",
                            k
                        ))));
                    }
                }
            }
            match (id, reading) {
                (Some(id), Some(reading)) => Ok(CustomEvent { id, reading }),
                _ => Err(Box::new(CalcifyError::ParseError(String::from(
                    "missing CustomEvent field",
                )))),
            }
        }

        fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
            if let Ok(2) = read_array_len(&mut bytes) {
                let id = read_int(&mut bytes)?;
                let reading = read_f64(&mut bytes)?;
                Ok((CustomEvent { id, reading }, bytes))
            } else {
                Err(Box::new(CalcifyError::ParseError(String::from(
                    "invalid MessagePack CustomEvent",
                ))))
            }
        }
    }

    #[test]
    fn test_ftree_stamped_builtins_and_helpers() -> Result<(), Box<dyn error::Error>> {
        fs::create_dir_all("./scratch")?;
        // f64 stream with empty feed & error checks
        let mut ftree_f64: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("Scalar_Feed");
        assert_eq!(ftree_f64.get_field("Name"), Some("Scalar_Feed"));
        assert_eq!(ftree_f64.subtype(), Some("StampedSample"));
        assert_eq!(ftree_f64.value_subtype(), Some("f64"));

        assert_eq!(
            ftree_f64.write_sample("missing", 1, 0.0),
            Err(CalcifyError::MissingKey(String::from("missing")))
        );
        assert_eq!(
            ftree_f64.write_samples("missing", vec![StampedSample::new(1, 0.0)]),
            Err(CalcifyError::MissingKey(String::from("missing")))
        );

        ftree_f64.add_stream("empty_stream")?;
        ftree_f64.add_stream("temps")?;
        assert_eq!(
            ftree_f64.add_stream("temps"),
            Err(CalcifyError::KeyAlreadyExists(String::from("temps")))
        );
        ftree_f64.write_sample("temps", 100, 21.5)?;
        ftree_f64.write_samples(
            "temps",
            vec![
                StampedSample::new(101, 22.0),
                StampedSample::new(102, 22.25),
            ],
        )?;
        assert_eq!(ftree_f64.get_feed("temps").unwrap().len(), 3);
        ftree_f64
            .get_feed_mut("temps")
            .unwrap()
            .push_sample(103, 23.0);

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
        assert_eq!(
            serde_json::from_str::<FeedTree<StampedSample<f64>>>(&serde_json::to_string(
                &ftree_f64
            )?)?,
            ftree_f64
        );

        // ThreeVec stream
        let mut ftree_3v: FeedTree<StampedSample<ThreeVec>> = FeedTree::new_stamped("Vec_Feed");
        ftree_3v.add_stream("pos")?;
        ftree_3v.write_sample("pos", -5, ThreeVec::new(1.0, 2.0, 3.0))?;
        ftree_3v.write_sample("pos", 5, ThreeVec::new(4.0, 5.0, 6.0))?;
        assert_eq!(
            FeedTree::<StampedSample<ThreeVec>>::from_json(&ftree_3v.to_json())?,
            ftree_3v
        );
        let raw_3v = ftree_3v.to_msg()?;
        let (msg_3v, rem_3v) = FeedTree::<StampedSample<ThreeVec>>::from_msg(&raw_3v)?;
        assert!(rem_3v.is_empty());
        assert_eq!(msg_3v, ftree_3v);

        // String stream
        let mut ftree_str: FeedTree<StampedSample<String>> = FeedTree::new_stamped("Str_Feed");
        ftree_str.add_stream("logs")?;
        ftree_str.write_sample("logs", 10, String::from("ok, {done}"))?;
        ftree_str.write_sample("logs", 20, String::from("quote \"here\""))?;
        assert_eq!(
            FeedTree::<StampedSample<String>>::from_json(&ftree_str.to_json())?,
            ftree_str
        );
        let raw_str = ftree_str.to_msg()?;
        let (msg_str, rem_str) = FeedTree::<StampedSample<String>>::from_msg(&raw_str)?;
        assert!(rem_str.is_empty());
        assert_eq!(msg_str, ftree_str);

        // Empty FeedTree (no feeds yet)
        let empty_ft: FeedTree<StampedSample<f64>> = FeedTree::new_stamped("Empty");
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
        fs::create_dir_all("./scratch")?;
        let mut ftree: FeedTree<StampedSample<CustomEvent>> =
            FeedTree::new_stamped_object("Events");
        ftree.add_stream("ev")?;
        ftree.write_sample(
            "ev",
            1_700_000_000,
            CustomEvent {
                id: 1,
                reading: 9.5,
            },
        )?;
        ftree.write_sample(
            "ev",
            1_700_000_001,
            CustomEvent {
                id: 2,
                reading: 10.5,
            },
        )?;

        assert_eq!(ftree.subtype(), Some("StampedSample"));
        assert_eq!(ftree.value_subtype(), Some("Object"));

        let json_rt = FeedTree::<StampedSample<CustomEvent>>::from_json(&ftree.to_json())?;
        assert_eq!(json_rt, ftree);

        ftree.write_msg("./scratch/test_stamped_ftree_obj.msg")?;
        let file_rt = FeedTree::<StampedSample<CustomEvent>>::read_msg(
            "./scratch/test_stamped_ftree_obj.msg",
        )?;
        assert_eq!(file_rt, ftree);

        Ok(())
    }
}
