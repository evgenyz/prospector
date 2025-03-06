use simplelog::*;

use block::{Fixture, Runner};
use serde_json;

pub mod block;
pub mod filter;
pub mod probe;
pub mod wrapper;

use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Prospector description goes here.
pub struct Options {
    /// target runner, optional
    #[argh(option, short = 'r')]
    target: Option<String>,

    /// fixture file, optional
    #[argh(option, short = 'f')]
    fixture: Option<String>,

    /// verbose output, optional
    #[argh(switch, short = 'V')]
    verbose: bool,

    /// test file(s), at least one is required
    #[argh(positional)]
    inputs: Vec<String>,
}

pub fn lib_main(opts: Options) {
    TermLogger::init(
        if opts.verbose {
            LevelFilter::max()
        } else {
            LevelFilter::Info
        },
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    info!("Target runner: {:?}", opts.target);
    info!("Inputs: {:?}", opts.inputs);
    info!("Fixture: {:?}", opts.fixture);

    let mut runner = Runner::new();
    if let Some(fixture) = opts.fixture {
        let f = Fixture::create_from_json(std::fs::read_to_string(fixture).unwrap()).unwrap();
        runner.set_fixture(f.get_cache());
    }
    let tests = load_tests(&opts.inputs);

    for test in tests {
        let result = test.execute(&mut runner);
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
}

fn load_tests(inputs: &Vec<String>) -> Vec<block::Test> {
    let tests: Vec<block::Test> = inputs
        .iter()
        .map(|i| block::Test::create_from_json(std::fs::read_to_string(i).unwrap()).unwrap())
        .collect();
    tests
}

pub trait Entity {
    fn get_id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_content_test_1() {
        let _tests = load_tests(&vec!["content/test_1.json".to_string()]);
    }
}
