/// Must return a json compliant String
pub trait Serializable {
    fn to_json(&self) -> String;
}

impl Serializable for u64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }
}

impl Serializable for f64 {
    fn to_json(&self) -> String {
        format!("{:.*}",5,self)
    }
}

/// Wraps the String in quotes("").
impl Serializable for String {
    fn to_json(&self) -> String {
        format!("\"{}\"",self)
    }
}
