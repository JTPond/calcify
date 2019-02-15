/// Must return a json complient String
pub trait Serializable {
    fn to_json(&self) -> String;
}
