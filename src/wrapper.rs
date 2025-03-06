use std::{collections::HashSet, io::Cursor};

use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use simplelog::warn;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Wrapper {
    Json(Json),
    JsonSeq(JsonSeq),
    RawLines(RawLines),
    Regexp(Regexp),
    CmdLine(CmdLine),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Json {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct JsonSeq {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawLines {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Regexp {
    expr: String,
    flags: Option<String>,
    #[serde(default)]
    map_key_val: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CmdLine {}

pub trait Wrapping {
    fn wrap(&self, input: &str) -> Value;
    fn wrap_all(&self, inputs: &Vec<(String, String)>) -> Value {
        let mut container = Vec::new();
        for input in inputs {
            let mut entry: Map<String, Value> = Map::new();
            entry.insert("source".to_string(), input.0.clone().into());
            entry.insert("data".to_string(), self.wrap(&input.1));
            container.push(entry);
        }
        container.into()
    }
}

impl Default for Wrapper {
    fn default() -> Self {
        Wrapper::Json(Json {})
    }
}

// TODO: enum_dispatch
impl Wrapping for Wrapper {
    fn wrap(&self, input: &str) -> Value {
        match self {
            Self::Json(wrp) => wrp.wrap(input),
            Self::JsonSeq(wrp) => wrp.wrap(input),
            Self::RawLines(wrp) => wrp.wrap(input),
            Self::Regexp(wrp) => wrp.wrap(input),
            Self::CmdLine(wrp) => wrp.wrap(input),
        }
    }
    fn wrap_all(&self, input: &Vec<(String, String)>) -> Value {
        match self {
            Self::Json(wrp) => wrp.wrap_all(input),
            Self::JsonSeq(wrp) => wrp.wrap_all(input),
            Self::RawLines(wrp) => wrp.wrap_all(input),
            Self::Regexp(wrp) => wrp.wrap_all(input),
            Self::CmdLine(wrp) => wrp.wrap_all(input),
        }
    }
}

impl Wrapping for Json {
    fn wrap(&self, input: &str) -> Value {
        serde_json::from_str(&input).expect("Unable to wrap JSON output!")
    }
}

impl Wrapping for JsonSeq {
    fn wrap(&self, input: &str) -> Value {
        let mut container = Vec::new();
        let reader = jsonseq::JsonSeqReader::new(Cursor::new(input));
        for item in reader {
            if let Ok(value) = item {
                container.push(value);
            }
        }
        container.into()
    }
}

impl Wrapping for RawLines {
    fn wrap(&self, input: &str) -> Value {
        let mut container = Vec::new();
        for line in input.lines() {
            container.push(Value::String(line.to_string()));
        }
        container.into()
    }
}

impl Wrapping for Regexp {
    fn wrap(&self, input: &str) -> Value {
        let mut single_map = false;
        let mut rb = &mut RegexBuilder::new(&self.expr);
        rb = rb.multi_line(true);
        if let Some(flags) = &self.flags {
            for f in flags.chars() {
                rb = match f {
                    'M' => {
                        single_map = true;
                        rb.multi_line(false)
                    }
                    's' => rb.dot_matches_new_line(true),
                    'i' => rb.case_insensitive(true),
                    _ => rb,
                };
            }
        }
        let re = rb.build().expect("Can't build the expression!");
        let group_names: Vec<&str> = re.capture_names().skip(1).map(|x| x.unwrap()).collect();
        if self.map_key_val {
            let key_val: HashSet<_> =
                HashSet::from_iter(vec!["key".to_string(), "val".to_string()]);
            if !group_names.iter().all(|&item| key_val.contains(item)) {
                panic!("The 'map_key_val' option of the Regex Wrapper requires 'key' and 'val' named groups defined in the expression!");
            } else {
                let mut caps_map: Map<String, Value> = Map::new();
                for caps in re.captures_iter(input) {
                    let mut key = Value::Null;
                    let mut val = Value::Null;
                    for name in &group_names {
                        if let Some(m) = caps.name(name) {
                            match *name {
                                "key" => key = m.as_str().into(),
                                "val" => val = m.as_str().into(),
                                _ => {}
                            }
                        }
                    }
                    if key != Value::Null {
                        caps_map.insert(key.as_str().unwrap().to_string(), val);
                    } else {
                        // TODO: Should be error?
                        warn!("The val='{}' does not have a corresponding key!", val);
                    }
                }
                caps_map.into()
            }
        } else {
            let mut container: Vec<Value> = Vec::new();
            for caps in re.captures_iter(input) {
                if !&group_names.is_empty() {
                    let mut caps_map: Map<String, Value> = Map::new();
                    for name in &group_names {
                        if let Some(m) = caps.name(name) {
                            caps_map.insert(name.to_string(), m.as_str().to_string().into());
                        }
                    }
                    container.push(caps_map.into());
                } else {
                    container.push(
                        caps.iter()
                            .skip(1)
                            .map(|x| x.unwrap().as_str().to_string())
                            .collect(),
                    );
                }
            }
            if container.len() == 1 && single_map {
                container.pop().unwrap()
            } else {
                container.into()
            }
        }
    }
}

impl Wrapping for CmdLine {
    fn wrap(&self, input: &str) -> Value {
        let line = input
            .lines()
            .next()
            .expect("At least one line is expected!");
        let args = line.split_whitespace();
        let mut map = Map::new();
        for arg in args {
            let pair: Vec<&str> = arg.splitn(2, '=').collect();
            if pair.len() > 1 {
                map.insert(pair[0].to_string(), pair[1].into());
            } else {
                map.insert(pair[0].to_string(), Value::from(true));
            }
        }
        map.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regexp_wrap() {
        let w = Regexp {
            expr: "^(?<key>[^#\\s]+)=(?<val>[^#\\s]+)$".to_string(),
            flags: None,
            map_key_val: true,
        };

        let input = "CONFIG_ARCH_USE_MEMTEST=y\n\
                    # CONFIG_HYPERV_TESTING is not set\n\
                    # end of Kernel Testing and Coverage\n\
                    \n\
                    #\n\
                    # Rust hacking\n\
                    #\n\
                    # CONFIG_RUST_DEBUG_ASSERTIONS is not set\n\
                    CONFIG_RUST_OVERFLOW_CHECKS=y\n\
                    # CONFIG_RUST_BUILD_ASSERT_ALLOW is not set\n\
                    # end of Kernel hacking";

        let output = w.wrap(input);
        dbg!(&output);

        let mut map = Map::new();
        map.insert("CONFIG_ARCH_USE_MEMTEST".to_string(), "y".into());
        map.insert("CONFIG_RUST_OVERFLOW_CHECKS".to_string(), "y".into());

        assert_eq!(output, Value::from(map));
    }

    #[test]
    fn cmdline_wrap() {
        let w = CmdLine {};

        let input = "BOOT_IMAGE=(hd0,gpt2)/vmlinuz-6.13.5-200.fc41.x86_64 \
                     root=UUID=cb60f854-e863-43bd-91bf-f67c66277478 ro \
                     rd.luks.uuid=luks-3cc3f5ad-fd3c-4205-b3b4-1ef14b7c6c50 rhg";

        let output = w.wrap(input);
        dbg!(&output);

        let mut res = Map::new();
        res.insert(
            "BOOT_IMAGE".to_string(),
            "(hd0,gpt2)/vmlinuz-6.13.5-200.fc41.x86_64".into(),
        );
        res.insert(
            "root".to_string(),
            "UUID=cb60f854-e863-43bd-91bf-f67c66277478".into(),
        );
        res.insert(
            "rd.luks.uuid".to_string(),
            "luks-3cc3f5ad-fd3c-4205-b3b4-1ef14b7c6c50".into(),
        );
        res.insert("ro".to_string(), true.into());
        res.insert("rhg".to_string(), true.into());

        assert_eq!(output, Value::from(res));
    }
}
