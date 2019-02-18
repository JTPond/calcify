/// Must return a json compliant String
pub trait Serializable {
    fn to_json(&self) -> String;
}
