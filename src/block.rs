use dependency_graph::{DependencyGraph, Node, Step};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Result, Value};
use simplelog::warn;

use crate::*;

use filter::Executable;
use probe::Runable;
use wrapper::Wrapping;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Test {
    id: String,
    blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Block {
    Probe(ProbeBlock),
    Filter(FilterBlock),
}

pub trait Entity {
    fn get_id(&self) -> &str;
}

impl Test {
    pub fn create_from_json(json_string: String) -> Result<Self> {
        let mut test: Self = serde_json::from_str(&json_string)?;
        let graph = DependencyGraph::from(test.blocks.as_slice());
        //graph.is_internally_resolvable();
        let _deps: Vec<_> = graph.unresolved_dependencies().collect();
        let mut ordered_block_ids = vec![];
        //dbg!(deps);
        for block in graph {
            match block {
                Step::Resolved(b) => {
                    //println!("Block id:{}, OK", b.get_id());
                    ordered_block_ids.push(b.get_id().to_string());
                }
                Step::Unresolved(b) => warn!("Block '{}', unsatified dependecies!", b),
            }
        }
        //dbg!(&ordered_block_ids);
        let mut sorted_blocks = vec![];
        for id in ordered_block_ids.iter() {
            let idx = test
                .blocks
                .iter()
                .position(|b| b.get_id() == id)
                .expect("That should not happen.");
            sorted_blocks.push(test.blocks.swap_remove(idx));
        }
        test.blocks = sorted_blocks;
        //dbg!(&test.blocks);
        Result::Ok(test)
    }

    pub fn execute(&self, runner: &mut Runner) -> Value {
        let mut results: Map<String, Value> = Map::new();
        let mut result = Value::Null;
        for block in &self.blocks {
            result = block.execute(runner, &results);
            results.insert(block.get_id().to_string(), result.clone());
        }
        debug!("Results: {:#?}", &results);
        result.into()
    }
}

impl Block {
    pub fn create_from_json(json_string: String) -> Result<Self> {
        serde_json::from_str(&json_string)
    }

    pub fn execute(&self, runner: &mut Runner, results: &Map<String, Value>) -> Value {
        match self {
            Block::Probe(block) => {
                let output = block.probe.run(runner);
                block.wrapper.wrap_all(&output)
            }
            Block::Filter(block) => block.filter.execute(runner, results),
        }
    }
}

impl Entity for Block {
    fn get_id(&self) -> &str {
        match &self {
            Block::Probe(block) => block.id.as_str(),
            Block::Filter(block) => block.id.as_str(),
        }
    }
}

impl Node for Block {
    type DependencyType = String;

    fn dependencies(&self) -> &[Self::DependencyType] {
        match self {
            Block::Probe(block) => &block.src[..],
            Block::Filter(block) => &block.src[..],
        }
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        match self {
            Block::Probe(block) => &block.id == dependency,
            Block::Filter(block) => &block.id == dependency,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProbeBlock {
    id: String,
    #[serde(default)]
    src: Vec<String>,
    probe: probe::Probe,
    #[serde(default)]
    wrapper: wrapper::Wrapper,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FilterBlock {
    id: String,
    #[serde(default)]
    src: Vec<String>,
    filter: filter::Filter,
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::{Version, VersionReq};

    #[test]
    fn test_create_from_json() {
        let json_s = r#"{
            "id": "test_id",
            "blocks": [
                {
                    "id": "block_1_id",
                    "probe": {
                        "process": {
                            "exec": "echo",
                            "args": ["{\"result\": true}"]
                        }
                    }
                },
                {
                    "id": "block_2_id",
                    "src": ["block_1_id"],
                    "filter": {
                        "cel": {
                            "expr": "1 + 1",
                            "args": null
                        }
                    }
                }
            ]
        }"#
        .to_string();

        let res: Result<Test> = Test::create_from_json(json_s);
        assert_eq!(
            res.expect("Oops!"),
            Test {
                id: "test_id".to_string(),
                blocks: vec![
                    Block::Probe(ProbeBlock {
                        id: "block_1_id".to_string(),
                        src: Default::default(),
                        probe: probe::Probe::Process(probe::ProcessProbe {
                            exec: "echo".to_string(),
                            args: vec!["{\"result\": true}".to_string()],
                        }),
                        wrapper: wrapper::Wrapper::default(),
                    }),
                    Block::Filter(FilterBlock {
                        id: "block_2_id".to_string(),
                        src: vec!["block_1_id".to_string()],
                        filter: filter::Filter::CEL(filter::cel::CELFilter {
                            expr: "1 + 1".to_string(),
                            args: None,
                        })
                    }),
                ]
            }
        )
    }

    #[test]
    fn block_create_from_json() {
        let json_s = r#"{
            "id": "block_id",
            "filter": {
                "cel": {
                    "expr": "1 + 1",
                    "args": null
                }
            }
        }"#
        .to_string();

        let res: Result<Block> = Block::create_from_json(json_s);
        assert_eq!(
            res.expect("Oops!"),
            Block::Filter(FilterBlock {
                id: "block_id".to_string(),
                src: Default::default(),
                filter: filter::Filter::CEL(filter::cel::CELFilter {
                    expr: "1 + 1".to_string(),
                    args: None,
                })
            })
        )
    }

    #[test]
    fn block_create_from_json_and_run_probe_file_raw_lines() {
        let json_s = r#"{
            "id": "block_id",
            "probe": {
                "file": {
                    "paths": ["/etc/fstab"]
                }
            },
            "wrapper": {
                "raw-lines": {}
            }
        }"#
        .to_string();

        let mut r = Runner::new();
        let b: Block = Block::create_from_json(json_s).expect("Can't create block from JSON");
        let s = b.execute(&mut r, &Map::new());
        println!("{:#?}", s);
    }

    #[test]
    fn block_create_from_json_and_run_probe_file_regexp() {
        let json_s = r#"{
            "id": "block_id",
            "probe": {
                "file": {
                    "paths": ["/etc/fstab"]
                }
            },
            "wrapper": {
                "regexp": {
                    "expr": "^(?:[^#])(?<fs_spec>\\S+)\\s+(?<fs_file>\\S+)\\s+(?<fs_vfstype>\\S+)\\s+(?<fs_mntops>\\S+)\\s*(?<fs_freq>\\S*)\\s*(?<fs_passno>\\S*)\\s*$",
                    "flags": "m"
                }
            }
        }"#
        .to_string();

        let mut r = Runner::new();
        let b: Block = Block::create_from_json(json_s).expect("Can't create block from JSON");
        let s = b.execute(&mut r, &Map::new());
        println!("{:#?}", s);
    }

    #[test]
    fn block_create_from_json_and_run_probe_process() {
        let json_s = r#"{
            "id": "block_id",
            "probe": {
                "process": {
                    "exec": "echo",
                    "args": ["{\"result\": true}"]
                }
            }
        }"#
        .to_string();

        let mut r = Runner::new();
        let b: Block = Block::create_from_json(json_s).expect("Can't create block from JSON");
        let s = b.execute(&mut r, &Map::new());
        println!("{:#?}", s);
    }

    #[test]
    fn runner_unglob() {
        let mut r = Runner::new();
        let result = r.unglob_path("/etc/fe*");
        println!("{:#?}", result);
    }

    #[test]
    fn semver() {
        let req = VersionReq::parse("0,4,>200").expect("Req!");
        dbg!(&req);
        let version = Version::parse("201.0.0").expect("Version!");
        dbg!(&version);
        // assert!(req.matches(&version));
    }
}
