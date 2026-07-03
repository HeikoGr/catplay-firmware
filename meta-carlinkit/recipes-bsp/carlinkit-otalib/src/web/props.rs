extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt::Write;

use crate::nostd::SmallFd;

pub const PROP_DEFS_PATH: &str = "/etc/carlinkit/props.tsv";
pub const PROP_VALUES_PATH: &str = "/tmp/carlinkit-props.tsv";

const BUILTIN_PROP_DEFS: &str = ""; // include_str!("../../props.tsv");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropType {
    Enum { values: Vec<String> },
    Bool,
    Int { min: i32, max: i32 },
    String { min_len: usize, max_len: usize },
}

#[derive(Clone, Debug)]
pub struct Prop {
    pub name: String,
    pub ty: PropType,
    pub value: String,
}

pub struct PropStore {
    props: Vec<Prop>,
    persist_path: &'static str,
}

impl PropStore {
    pub fn load() -> Self {
        Self::load_with_paths(PROP_DEFS_PATH, PROP_VALUES_PATH)
    }

    fn load_with_paths(def_path: &str, persist_path: &'static str) -> Self {
        let defs_text = read_file_text(def_path).unwrap_or_else(|_| BUILTIN_PROP_DEFS.to_string());
        let mut props = parse_props_file(&defs_text).unwrap_or_else(|_| parse_props_file(BUILTIN_PROP_DEFS).unwrap_or_else(|_| vec![]));

        if let Ok(values_text) = read_file_text(persist_path)
            && let Ok(overrides) = parse_props_file(&values_text)
        {
            apply_overrides(&mut props, &overrides);
        }

        Self { props, persist_path }
    }

    pub fn all(&self) -> &[Prop] {
        &self.props
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.props.iter().find(|prop| prop.name == name).map(|prop| prop.value.as_str())
    }

    pub fn set(&mut self, name: &str, value: &str) -> Result<&str, String> {
        let Some(idx) = self.props.iter().position(|prop| prop.name == name) else {
            return Err("unknown prop".to_string());
        };

        let normalized = validate_value(&self.props[idx].ty, value).map_err(ToString::to_string)?;
        let old = self.props[idx].value.clone();
        self.props[idx].value = normalized;

        if let Err(err) = write_props_file(self.persist_path, &self.props) {
            self.props[idx].value = old;
            return Err(format!("persist failed: {err}"));
        }

        Ok(self.props[idx].value.as_str())
    }
}

pub fn props_to_json(props: &[Prop]) -> String {
    let mut out = String::with_capacity(1024);
    out.push('[');

    for (idx, prop) in props.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }

        out.push('{');
        out.push_str("\"name\":\"");
        push_json_escaped(&mut out, &prop.name);
        out.push_str("\",\"value\":\"");
        push_json_escaped(&mut out, &prop.value);
        out.push('"');

        match &prop.ty {
            PropType::Enum { values } => {
                out.push_str(",\"type\":\"enum\",\"values\":[");
                for (v_idx, value) in values.iter().enumerate() {
                    if v_idx > 0 {
                        out.push(',');
                    }
                    out.push('"');
                    push_json_escaped(&mut out, value);
                    out.push('"');
                }
                out.push(']');
            }
            PropType::Bool => {
                out.push_str(",\"type\":\"bool\"");
            }
            PropType::Int { min, max } => {
                let _ = write!(&mut out, ",\"type\":\"int\",\"min\":{},\"max\":{}", min, max);
            }
            PropType::String { min_len, max_len } => {
                let _ = write!(&mut out, ",\"type\":\"string\",\"min_len\":{},\"max_len\":{}", min_len, max_len);
            }
        }

        out.push('}');
    }

    out.push(']');
    out
}

fn apply_overrides(props: &mut [Prop], overrides: &[Prop]) {
    for override_prop in overrides {
        let Some(target) = props.iter_mut().find(|x| x.name == override_prop.name) else {
            continue;
        };

        if !same_type(&target.ty, &override_prop.ty) {
            continue;
        }

        if let Ok(normalized) = validate_value(&target.ty, &override_prop.value) {
            target.value = normalized;
        }
    }
}

fn same_type(a: &PropType, b: &PropType) -> bool {
    match (a, b) {
        (PropType::Bool, PropType::Bool) => true,
        (PropType::Int { min: amin, max: amax }, PropType::Int { min: bmin, max: bmax }) => amin == bmin && amax == bmax,
        (
            PropType::String {
                min_len: amin,
                max_len: amax,
            },
            PropType::String {
                min_len: bmin,
                max_len: bmax,
            },
        ) => amin == bmin && amax == bmax,
        (PropType::Enum { values: a }, PropType::Enum { values: b }) => a == b,
        _ => false,
    }
}

fn parse_props_file(input: &str) -> Result<Vec<Prop>, &'static str> {
    let mut out = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let prop = parse_line(line)?;
        out.push(prop);
    }

    Ok(out)
}

fn parse_line(line: &str) -> Result<Prop, &'static str> {
    let mut parts = line.splitn(4, '\t');

    let name = parts.next().ok_or("missing name")?;
    let ty = parts.next().ok_or("missing type")?;
    let spec = parts.next().ok_or("missing spec")?;
    let value = parts.next().ok_or("missing value")?;

    validate_name(name)?;
    let parsed_type = parse_type(ty, spec)?;
    let normalized = validate_value(&parsed_type, value)?;

    Ok(Prop {
        name: name.to_string(),
        ty: parsed_type,
        value: normalized,
    })
}

fn parse_type(ty: &str, spec: &str) -> Result<PropType, &'static str> {
    match ty {
        "enum" => {
            let mut values = Vec::new();
            for item in spec.split(',') {
                if item.is_empty() {
                    continue;
                }
                validate_token(item)?;
                values.push(item.to_string());
            }
            if values.is_empty() {
                return Err("enum requires at least one value");
            }
            Ok(PropType::Enum { values })
        }
        "bool" => {
            if spec != "-" {
                return Err("bool spec must be '-'");
            }
            Ok(PropType::Bool)
        }
        "int" => {
            let (min, max) = parse_range_i32(spec)?;
            Ok(PropType::Int { min, max })
        }
        "string" => {
            let (min_len, max_len) = parse_range_usize(spec)?;
            Ok(PropType::String { min_len, max_len })
        }
        _ => Err("unknown type"),
    }
}

fn parse_range_i32(spec: &str) -> Result<(i32, i32), &'static str> {
    let (min_s, max_s) = parse_range_parts(spec)?;
    let min = min_s.parse::<i32>().map_err(|_| "invalid int range")?;
    let max = max_s.parse::<i32>().map_err(|_| "invalid int range")?;
    if min > max {
        return Err("invalid int range");
    }
    Ok((min, max))
}

fn parse_range_usize(spec: &str) -> Result<(usize, usize), &'static str> {
    let (min_s, max_s) = parse_range_parts(spec)?;
    let min = min_s.parse::<usize>().map_err(|_| "invalid string range")?;
    let max = max_s.parse::<usize>().map_err(|_| "invalid string range")?;
    if min > max {
        return Err("invalid string range");
    }
    Ok((min, max))
}

fn parse_range_parts(spec: &str) -> Result<(&str, &str), &'static str> {
    let mut parts = spec.splitn(2, ':');
    let a = parts.next().ok_or("missing range")?;
    let b = parts.next().ok_or("missing range")?;
    Ok((a, b))
}

fn validate_value(ty: &PropType, value: &str) -> Result<String, &'static str> {
    match ty {
        PropType::Enum { values } => {
            if values.iter().any(|x| x == value) {
                Ok(value.to_string())
            } else {
                Err("enum value is not allowed")
            }
        }
        PropType::Bool => match value {
            "true" | "1" => Ok("true".to_string()),
            "false" | "0" => Ok("false".to_string()),
            _ => Err("bool accepts only true/false/1/0"),
        },
        PropType::Int { min, max } => {
            let parsed = value.parse::<i32>().map_err(|_| "invalid int")?;
            if parsed < *min || parsed > *max {
                return Err("int out of range");
            }
            Ok(parsed.to_string())
        }
        PropType::String { min_len, max_len } => {
            validate_token(value)?;
            let len = value.len();
            if len < *min_len || len > *max_len {
                return Err("string length out of range");
            }
            Ok(value.to_string())
        }
    }
}

fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("empty prop name");
    }
    validate_token(name)
}

fn validate_token(value: &str) -> Result<(), &'static str> {
    if value.chars().any(|c| c == '\t' || c == '\r' || c == '\n') {
        return Err("value contains forbidden control char");
    }
    Ok(())
}

fn serialize_props_file(props: &[Prop]) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str("# name\ttype\tspec\tvalue\n");

    for prop in props {
        let (ty_str, spec) = type_to_fields(&prop.ty);
        out.push_str(&prop.name);
        out.push('\t');
        out.push_str(ty_str);
        out.push('\t');
        out.push_str(&spec);
        out.push('\t');
        out.push_str(&prop.value);
        out.push('\n');
    }

    out
}

fn type_to_fields(ty: &PropType) -> (&'static str, String) {
    match ty {
        PropType::Enum { values } => ("enum", values.join(",")),
        PropType::Bool => ("bool", "-".to_string()),
        PropType::Int { min, max } => ("int", format!("{}:{}", min, max)),
        PropType::String { min_len, max_len } => ("string", format!("{}:{}", min_len, max_len)),
    }
}

fn write_props_file(path: &str, props: &[Prop]) -> Result<(), &'static str> {
    let body = serialize_props_file(props);
    let fd = SmallFd::create(path)?;
    fd.truncate(body.len())?;

    let mut off = 0usize;
    let bytes = body.as_bytes();
    while off < bytes.len() {
        let n = fd.write(&bytes[off..])?;
        if n == 0 {
            return Err("short write");
        }
        off += n;
    }

    Ok(())
}

fn read_file_text(path: &str) -> Result<String, &'static str> {
    let fd = SmallFd::open_readonly(path)?;
    let st = fd.stat()?;
    let size = st.st_size.max(0) as usize;

    if size == 0 {
        return Ok(String::new());
    }

    let mut buf = vec![0u8; size];
    let mut off = 0usize;
    while off < size {
        let n = fd.read(&mut buf[off..])?;
        if n == 0 {
            break;
        }
        off += n;
    }
    buf.truncate(off);

    core::str::from_utf8(&buf).map(ToString::to_string).map_err(|_| "props file is not utf8")
}

fn push_json_escaped(out: &mut String, input: &str) {
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push('?'),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PropType, parse_props_file, serialize_props_file, validate_value};

    #[test]
    fn parse_file_and_normalize() {
        let text = "wifi.enabled\tbool\t-\t1\n";
        let props = parse_props_file(text).unwrap();
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].value, "true");
    }

    #[test]
    fn validate_int_range() {
        let ty = PropType::Int { min: 0, max: 10 };
        assert!(validate_value(&ty, "-1").is_err());
        assert_eq!(validate_value(&ty, "7").unwrap(), "7");
    }

    #[test]
    fn serialize_roundtrip() {
        let text = "wifi.band\tenum\t2.4g,5g,auto\tauto\n";
        let props = parse_props_file(text).unwrap();
        let out = serialize_props_file(&props);
        assert!(out.contains("wifi.band\tenum\t2.4g,5g,auto\tauto"));
    }
}
