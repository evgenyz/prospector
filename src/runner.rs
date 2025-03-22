use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use simplelog::info;
use std::{collections::HashMap, path::PathBuf, process};
use url_parse::core::Parser;

pub struct Runner {
    cache: HashMap<String, String>,
    plug: Plug,
}

pub enum Plug {
    Local(LocalPlug),
    Fixture(FixturePlug),
}

pub struct LocalPlug {}

pub struct FixturePlug {
    fixture: Fixture,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Fixture {
    id: String,
    src: HashMap<String, String>,
    result: Value,
}

impl Fixture {
    pub fn create_from_json(json_string: String) -> Result<Self> {
        serde_json::from_str(&json_string)
    }
}

trait Plugged {
    fn run(&self, cmd: &str, args: &Vec<String>) -> Vec<u8>;
    fn has_cache(&self) -> bool {
        return false;
    }
}

impl Plugged for Plug {
    fn run(&self, cmd: &str, args: &Vec<String>) -> Vec<u8> {
        match self {
            Plug::Local(_local) => {
                let p = process::Command::new(cmd)
                    .args(args)
                    .output()
                    .expect("Can't execute a command!");
                p.stdout
            }
            Plug::Fixture(fixture) => {
                let key = generate_key(cmd, args);
                fixture
                    .fixture
                    .src
                    .get(&key)
                    .expect(format!("Result for '{}' is not defined in the fixture!", key).as_str())
                    .clone()
                    .into()
            }
        }
    }
}

fn generate_key(cmd: &str, args: &Vec<String>) -> String {
    let mut key = String::new();
    key.push_str(cmd);
    key.push_str(" ");
    key.push_str(&args.join(" "));
    key
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            cache: Default::default(),
            plug: Plug::Local(LocalPlug {}),
        }
    }

    pub fn new_with_target(target: &Option<String>) -> Self {
        let plug = if let Some(url) = target {
            let mut supported_schemas = HashMap::new();
            supported_schemas.insert("local", (0, "Local"));
            supported_schemas.insert("fixture", (0, "Fixture Mock"));
            let result = Parser::new(Some(supported_schemas))
                .parse(url)
                .expect("Unsupported target URL!");
            match result.scheme.unwrap().as_str() {
                "fixture" => {
                    let path: PathBuf = result.path.unwrap().iter().collect();
                    let fixture =
                        Fixture::create_from_json(std::fs::read_to_string(path).unwrap()).unwrap();
                    Plug::Fixture(FixturePlug { fixture })
                }
                "local" => Plug::Local(LocalPlug {}),
                _ => panic!("Unknown scheme!"),
            }
        } else {
            Plug::Local(LocalPlug {})
        };

        Runner {
            cache: Default::default(),
            plug,
        }
    }

    pub fn add_value_to_cache(&mut self, k: &str, v: String) {
        if !self.plug.has_cache() {
            self.cache.insert(k.to_string(), v);
        }
    }

    pub fn get_value_from_cache(&self, k: &str) -> Option<&String> {
        if !self.plug.has_cache() {
            self.cache.get(k)
        } else {
            None
        }
    }

    pub fn unglob_path(&mut self, path: &str) -> Vec<String> {
        // TODO: Use shellescape
        let script = format!("compgen -G '{}'", path);
        self.sh(&script)
    }

    pub fn sh(&mut self, script: &str) -> Vec<String> {
        let output = self.run("/usr/bin/bash", &vec!["-c".to_string(), script.to_string()]);
        output.lines().map(|l| l.to_string()).collect()
    }

    pub fn cat(&mut self, path: &str) -> String {
        self.run("/usr/bin/cat", &vec![path.to_string()])
    }

    pub fn run(&mut self, cmd: &str, args: &Vec<String>) -> String {
        info!("Executing command: {} {:?}", cmd, args);
        let key = generate_key(cmd, args);

        if let Some(output) = self.get_value_from_cache(&key) {
            output.clone()
        } else {
            let raw_output = self.plug.run(cmd, args);
            let output = String::from_utf8(raw_output).expect("Can't decode output!");
            self.add_value_to_cache(&key, output.clone());
            output
        }
    }
}
