use std::marker::Sized;
use std::error;

use rmp::Marker;
use rmp::encode::*;
use rmp::decode::*;

use crate::utils::errors::CalcifyError;
/// Serialization trait which all types you intend to put in a Tree need to implement.
pub trait Serializable {
    /// Return object intensive json string
    /// # I.E.
    /// `FourVec -> {"x0":1.0,"x1":0.0,"x2":0.0,"x3":0.0}`
    fn to_json(&self) -> String;
    /// Return Result wrapped Vec<u8> in MsgPack
    /// Format is *not* like to_json it is array intensive not object
    ///
    /// #Errors
    /// * The rmp library returns `ValueWriteError` on write errors
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> ;
}
/// Deserialization trait which all types you intend to get out of a Tree need to implement.
/// Really only designed to work with data that was serialized with the Calcify::Serializable trait
/// and will not work on arbitrarily modified tree files
pub trait Deserializable {
    /// Return Self from string
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>>
        where Self: Sized;
    /// Return a tuple of Self and a &[u8] of remaining unparsed bytes from a byte array
    fn from_msg(bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>>
        where Self: Sized;
}

impl Serializable for u64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }

    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_uint(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for u64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        string.trim().parse::<u64>().map_err(|e| e.into())
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        Ok((read_int(&mut bytes)?,bytes))
    }
}

impl Serializable for i64 {
    fn to_json(&self) -> String {
        format!("{}", self)
    }

    fn to_msg(&self) -> Result<Vec<u8>, ValueWriteError> {
        let mut buf = Vec::new();
        write_sint(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for i64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        string.trim().parse::<i64>().map_err(|e| e.into())
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        Ok((read_int(&mut bytes)?, bytes))
    }
}

impl Serializable for f64 {
    fn to_json(&self) -> String {
        format!("{}",self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_f64(&mut buf, *self)?;
        Ok(buf)
    }
}

impl Deserializable for f64 {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        string.trim().parse::<f64>().map_err(|e| e.into())
    }
    fn from_msg(mut bytes: &[u8]) -> Result<(Self,&[u8]), Box<dyn error::Error>> {
        Ok((read_f64(&mut bytes)?,bytes))
    }
}

pub(crate) fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_hex4(chars: &mut std::str::Chars<'_>) -> Result<u16, CalcifyError> {
    let mut buf = [0u8; 4];
    for slot in &mut buf {
        let ch = chars.next().ok_or(CalcifyError::ParseError)?;
        if !ch.is_ascii_hexdigit() {
            return Err(CalcifyError::ParseError);
        }
        *slot = ch as u8;
    }
    let hex_str = std::str::from_utf8(&buf).map_err(|_| CalcifyError::ParseError)?;
    u16::from_str_radix(hex_str, 16).map_err(|_| CalcifyError::ParseError)
}

pub(crate) fn unescape_json_string(s: &str) -> Result<String, CalcifyError> {
    let trimmed = s.trim();
    if trimmed.len() < 2 || !trimmed.starts_with('"') || !trimmed.ends_with('"') {
        return Err(CalcifyError::ParseError);
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Err(CalcifyError::ParseError),
            c if (c as u32) < 0x20 => return Err(CalcifyError::ParseError),
            '\\' => match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('b') => out.push('\x08'),
                Some('f') => out.push('\x0c'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let code = parse_hex4(&mut chars)?;
                    if (0xD800..=0xDBFF).contains(&code) {
                        if chars.next() != Some('\\') || chars.next() != Some('u') {
                            return Err(CalcifyError::ParseError);
                        }
                        let low = parse_hex4(&mut chars)?;
                        if !(0xDC00..=0xDFFF).contains(&low) {
                            return Err(CalcifyError::ParseError);
                        }
                        let cp = 0x10000 + (((code as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
                        let decoded = char::from_u32(cp).ok_or(CalcifyError::ParseError)?;
                        out.push(decoded);
                    } else if (0xDC00..=0xDFFF).contains(&code) {
                        return Err(CalcifyError::ParseError);
                    } else {
                        let decoded = char::from_u32(code as u32).ok_or(CalcifyError::ParseError)?;
                        out.push(decoded);
                    }
                }
                _ => return Err(CalcifyError::ParseError),
            },
            c => out.push(c),
        }
    }
    Ok(out)
}

fn split_top_level_comma(inner: &str) -> Result<Vec<&str>, CalcifyError> {
    let mut parts = Vec::new();
    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0;

    for (idx, ch) in inner.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => stack.push('}'),
            '[' => stack.push(']'),
            '}' | ']' => {
                if stack.pop() != Some(ch) {
                    return Err(CalcifyError::ParseError);
                }
            }
            ',' if stack.is_empty() => {
                let elem = inner[start..idx].trim();
                if elem.is_empty() {
                    return Err(CalcifyError::ParseError);
                }
                parts.push(elem);
                start = idx + 1;
            }
            _ => {}
        }
    }

    if in_string || !stack.is_empty() {
        return Err(CalcifyError::ParseError);
    }

    let tail = inner[start..].trim();
    if tail.is_empty() {
        return Err(CalcifyError::ParseError);
    }
    parts.push(tail);
    Ok(parts)
}

fn find_top_level_colon(entry: &str) -> Result<usize, CalcifyError> {
    let mut in_string = false;
    let mut escaped = false;
    let mut depth: usize = 0;
    for (idx, ch) in entry.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                if depth == 0 {
                    return Err(CalcifyError::ParseError);
                }
                depth -= 1;
            }
            ':' if depth == 0 => return Ok(idx),
            _ => {}
        }
    }
    Err(CalcifyError::ParseError)
}

pub(crate) fn split_json_array(s: &str) -> Result<Vec<&str>, CalcifyError> {
    let trimmed = s.trim();
    if trimmed.len() < 2 || !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(CalcifyError::ParseError);
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    split_top_level_comma(inner)
}

pub(crate) fn split_json_object(s: &str) -> Result<Vec<(String, &str)>, CalcifyError> {
    let trimmed = s.trim();
    if trimmed.len() < 2 || !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(CalcifyError::ParseError);
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    let entries = split_top_level_comma(inner)?;
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let colon_idx = find_top_level_colon(entry)?;
        let key_slice = entry[..colon_idx].trim();
        let val_slice = entry[colon_idx + 1..].trim();
        if val_slice.is_empty() {
            return Err(CalcifyError::ParseError);
        }
        let key = unescape_json_string(key_slice)?;
        out.push((key, val_slice));
    }
    Ok(out)
}

fn take_bytes<'a>(cur: &mut &'a [u8], n: usize) -> Result<&'a [u8], CalcifyError> {
    if cur.len() < n {
        return Err(CalcifyError::ParseError);
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
        let marker = read_marker(&mut cur).map_err(|_| CalcifyError::ParseError)?;
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
                    .ok_or(CalcifyError::ParseError)?;
            }
            Marker::Array16 => {
                let b = take_bytes(&mut cur, 2)?;
                let len = u16::from_be_bytes([b[0], b[1]]) as usize;
                remaining = remaining
                    .checked_add(len)
                    .ok_or(CalcifyError::ParseError)?;
            }
            Marker::Array32 => {
                let b = take_bytes(&mut cur, 4)?;
                let len = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
                remaining = remaining
                    .checked_add(len)
                    .ok_or(CalcifyError::ParseError)?;
            }
            Marker::FixMap(len) => {
                let items = (len as usize)
                    .checked_mul(2)
                    .ok_or(CalcifyError::ParseError)?;
                remaining = remaining
                    .checked_add(items)
                    .ok_or(CalcifyError::ParseError)?;
            }
            Marker::Map16 => {
                let b = take_bytes(&mut cur, 2)?;
                let items = (u16::from_be_bytes([b[0], b[1]]) as usize)
                    .checked_mul(2)
                    .ok_or(CalcifyError::ParseError)?;
                remaining = remaining
                    .checked_add(items)
                    .ok_or(CalcifyError::ParseError)?;
            }
            Marker::Map32 => {
                let b = take_bytes(&mut cur, 4)?;
                let items = (u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
                    .checked_mul(2)
                    .ok_or(CalcifyError::ParseError)?;
                remaining = remaining
                    .checked_add(items)
                    .ok_or(CalcifyError::ParseError)?;
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
                let total = len.checked_add(1).ok_or(CalcifyError::ParseError)?;
                take_bytes(&mut cur, total)?;
            }
            Marker::Reserved => return Err(CalcifyError::ParseError),
        }
    }

    let consumed = input.len() - cur.len();
    Ok((&input[..consumed], cur))
}

/// Wraps the String in quotes("").
impl Serializable for String {
    fn to_json(&self) -> String {
        escape_json_string(self)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::new();
        write_str(&mut buf, self.as_str())?;
        Ok(buf)
    }
}

impl Deserializable for String {
    fn from_json(string: &str) -> Result<Self, Box<dyn error::Error>> {
        unescape_json_string(string).map_err(|e| Box::new(e) as Box<dyn error::Error>)
    }

    fn from_msg(bytes: &[u8]) -> Result<(Self, &[u8]), Box<dyn error::Error>> {
        let (s, rest) = read_str_from_slice(bytes)
            .map_err(|_| Box::new(CalcifyError::ParseError) as Box<dyn error::Error>)?;
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
    fn test_split_json_array() {
        assert_eq!(split_json_array("[]").unwrap(), Vec::<&str>::new());
        assert_eq!(split_json_array("  [  ]  ").unwrap(), Vec::<&str>::new());
        assert_eq!(split_json_array("[1, 2, 3]").unwrap(), vec!["1", "2", "3"]);
        assert_eq!(
            split_json_array(r#"[{"a":[1,2],"b":"x,y}"}, {"a":[]}]"#).unwrap(),
            vec![r#"{"a":[1,2],"b":"x,y}"}"#, r#"{"a":[]}"#]
        );
        assert!(split_json_array("[1, 2, ]").is_err());
        assert!(split_json_array("[,1]").is_err());
        assert!(split_json_array("[1,,2]").is_err());
        assert!(split_json_array("[1],[2]").is_err());
        assert!(split_json_array("[}]").is_err());
        assert!(split_json_array("[{1]").is_err());
        assert!(split_json_array("[\"unterminated]").is_err());
    }

    #[test]
    fn test_split_json_object() {
        assert_eq!(split_json_object("{}").unwrap(), Vec::<(String, &str)>::new());
        assert_eq!(
            split_json_object(r#" { "a:b" : [1, 2] , "c" : {"x": 3} } "#).unwrap(),
            vec![
                ("a:b".to_string(), "[1, 2]"),
                ("c".to_string(), "{\"x\": 3}"),
            ]
        );
        assert!(split_json_object(r#"{"a": 1,}"#).is_err());
        assert!(split_json_object(r#"{"a":}"#).is_err());
        assert!(split_json_object(r#"{"a"}"#).is_err());
        assert!(split_json_object(r#"{"a":1},{"b":2}"#).is_err());
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
