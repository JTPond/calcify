use std::collections::HashMap;

use super::collection;

use collection::Collection;
use collection::Serializable;

extern crate rmp;
use rmp::encode::*;

/// Feed
///
/// Trait that gives datafeeds data stack functionality
pub trait Feed<T: Serializable> : Serializable {
    /// Push a new item on the Feed's underlying stack-like container
    fn record(&mut self, item: T);
    /// Pass Serializable::to_json up to the Feed trait
    fn export_json(&self) -> String {
        Serializable::to_json(self)
    }
    /// Pass Serializable::to_jsonc up to the Feed trait
    fn export_jsonc(&self) -> String {
        Serializable::to_jsonc(self)
    }
    /// Pass Serializable::to_msg up to the Feed trait
    ///
    /// #Errors
    /// * The rmp library returns `ValueWriteError` on write errors
    fn export_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        Serializable::to_msg(self)
    }
}


impl<T: Serializable> Feed<T> for Collection<T> {
    fn record(&mut self, item: T) {
        self.push(item);
    }
}


pub struct FeedTree<T: Serializable> {
    metadata: HashMap<&'static str,&'static str>,
    datafeeds: HashMap<&'static str,Box<dyn Feed<T>>>,
}

impl<T: 'static + Serializable> FeedTree<T> {
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
    /// let f_col: Collection<f64> = Collection::from_vec(vec![0.0,0.0]);
    /// let mut ftree: FeedTree<f64> = FeedTree::new("Test_Tree","f64");
    /// ftree.add_field("Desc", "This is a FeedTree for testing.");
    /// ftree.add_feed("fcol", f_col);
    /// ftree.write("fcol", 1.0);
    /// ```
    pub fn new(name: &'static str, subtype: &'static str) -> FeedTree<T> {
        let mut md = HashMap::new();
        md.insert("Name",name);
        md.insert("SubType",subtype);
        let df = HashMap::new();
        FeedTree {
            metadata: md,
            datafeeds: df,
        }
    }

    pub fn add_field(&mut self, key: &'static str, f: &'static str) {
        self.metadata.insert(key,f);
    }

    /// Inserts new Collection<T> into FeedTree.
    ///
    /// # Arguments
    ///
    /// * `key` - Hash key, &'static str
    /// * `f` - Collection<T: Serializable>
    pub fn add_feed(&mut self, key: &'static str, f: Collection<T>) {
        self.datafeeds.insert(key,Box::new(f));
    }

    pub fn write(&mut self, key: &'static str, data: T) {
        self.datafeeds.get_mut(key).unwrap().record(data);
    }
}

impl<T: Serializable> Serializable for FeedTree<T> {
    fn to_json(&self) -> String {
        let mut out = String::from("{");
        for (key, val) in &self.metadata {
            out.push_str(format!("\"{}\":\"{}\",",key,val).as_str());
        }
        out.push_str("\"datafeeds\":{");
        for (key, val) in &self.datafeeds {
            out.push_str(format!("\"{}\":{},",key,val.export_json()).as_str());
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
        out.push_str("\"datafeeds\":{");
        for (key, val) in &self.datafeeds {
            out.push_str(format!("\"{}\":{},",key,val.export_jsonc()).as_str());
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
        write_str(&mut buf, "datafeeds")?;
        write_map_len(&mut buf, (self.datafeeds.len()) as u32)?;
        for (key, val) in &self.datafeeds {
            write_str(&mut buf, key)?;
            buf.append(&mut val.export_msg()?);
        }
        Ok(buf)
    }
}
