use prospector::block::*;
use semver::{Version, VersionReq};
use serde_json::Map;

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
fn block_create_from_json_and_exec_filter_cel() {
    let json_s = r#"{
            "id": "block_id",
            "filter": {
                "cel": {
                    "expr": "1+1",
                    "args": {"num": 2}
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
