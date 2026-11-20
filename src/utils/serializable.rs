use std::collections::HashSet;
use std::error;
use std::fmt;
use std::marker::Sized;

use rmp::decode::*;
use rmp::encode::*;
use rmp::Marker;
use serde::de::{MapAccess, Visitor};
use serde::Deserializer;
use serde_json::value::RawValue;

use crate::utils::errors::CalcifyError;
/// Serialization trait which all types you intend to put in a Tree need to implement.
pub trait Serializable {
    /// Return object intensive json string.
    /// # I.E.
    /// `FourVec -> {"x0":1.0,"x1":0.0,"x2":0.0,"x3":0.0}`
    ///
    /// Note: Non-finite `f64` values (`NaN`, `inf`, `-inf`) serialize as JSON `null`
    /// via `serde_json`, which `f64::from_json` rejects on deserialization.
    fn to_json(&self) -> String;
    /// Return Result wrapped Vec<u8> in MsgPack
    /// Format is *not* like to_json it is array intensive not object
    ///
    /// #Errors
    /// * The rmp library returns `ValueWriteError` on write errors
    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError>;
}
/// Deserialization trait which all types you intend to get out of a Tree need to implement.
/// Really only designed to work with data that was serialized with the Calcify::Serializable trait
/// and will not work on arbitrarily modified tree files
pub trait Deserializable {
    /// Return Self from string
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>>
    where
        Self: Sized;
    /// Return a tuple of Self and a &[u8] of remaining unparsed bytes from a byte array
    fn from_msg(bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>>
    where
        Self: Sized;
}

impl Serializable for u64 {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_uint(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for u64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(string)?)
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        Ok((read_int(&mut bytes)?, bytes))
    }
}

impl Serializable for i64 {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_sint(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for i64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(string)?)
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        Ok((read_int(&mut bytes)?, bytes))
    }
}

impl Serializable for f64 {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_f64(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for f64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(string)?)
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        Ok((read_f64(&mut bytes)?, bytes))
    }
}

pub(crate) fn json_array_elements(s: &str) -> Result<Vec<&str>, serde_json::Error> {
    let elems: Vec<&RawValue> = serde_json::from_str(s)?;
    Ok(elems.into_iter().map(RawValue::get).collect())
}

pub(crate) fn json_object_entries(s: &str) -> Result<Vec<(String, &str)>, serde_json::Error> {
    struct EntriesVisitor;

    impl<'de> Visitor<'de> for EntriesVisitor {
        type Value = Vec<(String, &'de str)>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a JSON object")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
            let mut seen = HashSet::with_capacity(map.size_hint().unwrap_or(0));
            while let Some((key, raw)) = map.next_entry::<String, &'de RawValue>()? {
                if !seen.insert(key.clone()) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate key `{}`",
                        key
                    )));
                }
                entries.push((key, raw.get()));
            }
            Ok(entries)
        }
    }

    let mut de = serde_json::Deserializer::from_str(s);
    let entries = de.deserialize_map(EntriesVisitor)?;
    de.end()?;
    Ok(entries)
}

fn take_bytes<'a>(cur: &mut &'a [u8], n: usize) -> Result<&'a [u8], CalcifyError> {
    if cur.len() < n {
        return Err(CalcifyError::ParseError(String::from(
            "unexpected end of MessagePack input",
        )));
    }
    let (head, tail) = cur.split_at(n);
    *cur = tail;
    Ok(head)
}

pub(crate) fn split_msgpack_value(input: &[u8]) -> Result<(&[u8], &[u8]), CalcifyError> {
    let mut cur = input;
    let mut remaining: usize = 1;

    while remaining > 0 {
        remaining -= 1;
        let marker = read_marker(&mut cur)
            .map_err(|e| CalcifyError::ParseError(format!("invalid MessagePack marker: {:?}", e)))?;
        match marker {
            Marker::FixPos(_)
            | Marker::FixNeg(_)
            | Marker::Null
            | Marker::False
            | Marker::True => {}
            Marker::U8 | Marker::I8 => {
                take_bytes(&mut cur, 1)?;
            }
            Marker::U16 | Marker::I16 => {
                take_bytes(&mut cur, 2)?;
            }
            Marker::U32 | Marker::I32 | Marker::F32 => {
                take_bytes(&mut cur, 4)?;
            }
            Marker::U64 | Marker::I64 | Marker::F64 => {
                take_bytes(&mut cur, 8)?;
            }
            Marker::FixStr(len) => {
                take_bytes(&mut cur, len as usize)?;
            }
            Marker::Str8 | Marker::Bin8 => {
                let len = take_bytes(&mut cur, 1)?[0] as usize;
                take_bytes(&mut cur, len)?;
            }
            Marker::Str16 | Marker::Bin16 => {
                let b = take_bytes(&mut cur, 2)?;
                let len = u16::from_be_bytes([b[0], b[1]]) as usize;
                take_bytes(&mut cur, len)?;
            }
            Marker::Str32 | Marker::Bin32 => {
                let b = take_bytes(&mut cur, 4)?;
                let len = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
                take_bytes(&mut cur, len)?;
            }
            Marker::FixArray(len) => {
                remaining = remaining
                    .checked_add(len as usize)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
            }
            Marker::Array16 => {
                let b = take_bytes(&mut cur, 2)?;
                let len = u16::from_be_bytes([b[0], b[1]]) as usize;
                remaining = remaining
                    .checked_add(len)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
            }
            Marker::Array32 => {
                let b = take_bytes(&mut cur, 4)?;
                let len = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
                remaining = remaining
                    .checked_add(len)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
            }
            Marker::FixMap(len) => {
                let items = (len as usize)
                    .checked_mul(2)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
                remaining = remaining
                    .checked_add(items)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
            }
            Marker::Map16 => {
                let b = take_bytes(&mut cur, 2)?;
                let items = (u16::from_be_bytes([b[0], b[1]]) as usize)
                    .checked_mul(2)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
                remaining = remaining
                    .checked_add(items)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
            }
            Marker::Map32 => {
                let b = take_bytes(&mut cur, 4)?;
                let items = (u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
                    .checked_mul(2)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
                remaining = remaining
                    .checked_add(items)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
            }
            Marker::FixExt1 => {
                take_bytes(&mut cur, 2)?;
            }
            Marker::FixExt2 => {
                take_bytes(&mut cur, 3)?;
            }
            Marker::FixExt4 => {
                take_bytes(&mut cur, 5)?;
            }
            Marker::FixExt8 => {
                take_bytes(&mut cur, 9)?;
            }
            Marker::FixExt16 => {
                take_bytes(&mut cur, 17)?;
            }
            Marker::Ext8 => {
                let len = take_bytes(&mut cur, 1)?[0] as usize;
                take_bytes(&mut cur, 1 + len)?;
            }
            Marker::Ext16 => {
                let b = take_bytes(&mut cur, 2)?;
                let len = u16::from_be_bytes([b[0], b[1]]) as usize;
                take_bytes(&mut cur, 1 + len)?;
            }
            Marker::Ext32 => {
                let b = take_bytes(&mut cur, 4)?;
                let len = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
                let total = len
                    .checked_add(1)
                    .ok_or_else(|| CalcifyError::ParseError(String::from("MessagePack size overflow")))?;
                take_bytes(&mut cur, total)?;
            }
            Marker::Reserved => {
                return Err(CalcifyError::ParseError(String::from(
                    "reserved MessagePack marker",
                )));
            }
        }
    }

    let consumed = input.len() - cur.len();
    Ok((&input[..consumed], cur))
}

/// Wraps the String in quotes("").
impl Serializable for String {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_str(&mut buf, self.as_str())?;
        Ok(buf)
    }
}

impl Deserializable for String {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        Ok(serde_json::from_str(string)?)
    }

    fn from_msg(bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let (s, rest) = read_str_from_slice(bytes)
            .map_err(|e| Box::new(CalcifyError::ParseError(format!("invalid MessagePack string: {}", e))) as Box<dyn error::Error>)?;
        Ok((s.to_owned(), rest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i64_roundtrip() {
        let cases = [
            0i64,
            -1,
            -32,
            -33,
            -128,
            -129,
            -32768,
            -32769,
            -2147483648,
            -2147483649,
            127,
            128,
            255,
            256,
            65535,
            65536,
            4294967295,
            4294967296,
            1_700_000_000_123_456_789i64,
            i64::MIN,
            i64::MAX,
        ];
        for val in cases {
            let json = val.to_json();
            assert_eq!(i64::from_json(&json).unwrap(), val);
            assert_eq!(i64::from_json(&format!("  {}  ", json)).unwrap(), val);

            let msg = val.to_msg().unwrap();
            let (decoded, rest) = i64::from_msg(&msg).unwrap();
            assert_eq!(decoded, val);
            assert!(rest.is_empty());
        }
    }

    #[test]
    fn test_f64_non_finite_json() {
        assert_eq!(f64::NAN.to_json(), "null");
        assert_eq!(f64::INFINITY.to_json(), "null");
        assert_eq!(f64::NEG_INFINITY.to_json(), "null");
        assert!(f64::from_json("null").is_err());
        assert_eq!(f64::from_json("1.5").unwrap(), 1.5);
    }

    #[test]
    fn test_string_roundtrip() {
        let cases = [
            "",
            "hello",
            "a,b,c",
            "{\"k\": \"v\"}",
            "quote \" and backslash \\ and newline \n and tab \t and cr \r",
            "ctrl \x08 \x0c \x01",
            "unicode π 🚀",
        ];
        for s in cases {
            let orig = s.to_string();
            let json = orig.to_json();
            assert_eq!(String::from_json(&json).unwrap(), orig);

            let msg = orig.to_msg().unwrap();
            let (decoded, rest) = String::from_msg(&msg).unwrap();
            assert_eq!(decoded, orig);
            assert!(rest.is_empty());
        }
        assert_eq!(
            String::from_json(r#""\u0041\ud83d\ude80""#).unwrap(),
            "A🚀"
        );
        assert!(String::from_json(r#"unquoted"#).is_err());
        assert!(String::from_json(r#""bad\xescape""#).is_err());
        assert!(String::from_json(r#""unpaired\ud800""#).is_err());
        assert!(String::from_json(r#""unescaped"quote""#).is_err());
    }

    #[test]
    fn test_json_array_elements() {
        assert_eq!(json_array_elements("[]").unwrap(), Vec::<&str>::new());
        assert_eq!(json_array_elements("  [  ]  ").unwrap(), Vec::<&str>::new());
        assert_eq!(json_array_elements("[1, 2, 3]").unwrap(), vec!["1", "2", "3"]);
        assert_eq!(
            json_array_elements(r#"[{"a":[1,2],"b":"x,y}"}, {"a":[]}]"#).unwrap(),
            vec![r#"{"a":[1,2],"b":"x,y}"}"#, r#"{"a":[]}"#]
        );
        assert!(json_array_elements("[1, 2, ]").is_err());
        assert!(json_array_elements("[,1]").is_err());
        assert!(json_array_elements("[1,,2]").is_err());
        assert!(json_array_elements("[1],[2]").is_err());
        assert!(json_array_elements("[}]").is_err());
        assert!(json_array_elements("[{1]").is_err());
        assert!(json_array_elements("[\"unterminated]").is_err());
    }

    #[test]
    fn test_json_object_entries() {
        assert_eq!(
            json_object_entries("{}").unwrap(),
            Vec::<(String, &str)>::new()
        );
        assert_eq!(
            json_object_entries(r#" { "a:b" : [1, 2] , "c" : {"x": 3}, "d": "a\"b" } "#).unwrap(),
            vec![
                ("a:b".to_string(), "[1, 2]"),
                ("c".to_string(), "{\"x\": 3}"),
                ("d".to_string(), "\"a\\\"b\""),
            ]
        );
        assert!(json_object_entries(r#"{"a":1,"a":2}"#).is_err());
        assert!(json_object_entries(r#"{"a": 1,}"#).is_err());
        assert!(json_object_entries(r#"{"a":}"#).is_err());
        assert!(json_object_entries(r#"{"a"}"#).is_err());
        assert!(json_object_entries(r#"{"a":1},{"b":2}"#).is_err());
    }

    #[test]
    fn test_split_msgpack_value() {
        let mut buf = Vec::new();
        write_array_len(&mut buf, 2).unwrap();
        write_sint(&mut buf, -42).unwrap();
        write_map_len(&mut buf, 1).unwrap();
        write_str(&mut buf, "k").unwrap();
        write_f64(&mut buf, 3.5).unwrap();
        let first_len = buf.len();
        write_str(&mut buf, "trailing").unwrap();

        let (first, rest) = split_msgpack_value(&buf).unwrap();
        assert_eq!(first.len(), first_len);
        let (trailing, rem) = String::from_msg(rest).unwrap();
        assert_eq!(trailing, "trailing");
        assert!(rem.is_empty());

        // Ext and reserved / truncated
        let ext_buf = [0xd4u8, 1, 0xaa, 0x05]; // fixext1 (type 1, data 0xaa) + fixpos 5
        let (ext_val, ext_rest) = split_msgpack_value(&ext_buf).unwrap();
        assert_eq!(ext_val, &[0xd4u8, 1, 0xaa]);
        assert_eq!(ext_rest, &[0x05]);

        assert!(split_msgpack_value(&[0xc1]).is_err());
        assert!(split_msgpack_value(&[0x92, 0x01]).is_err());
        assert!(split_msgpack_value(&[]).is_err());
    }
}
