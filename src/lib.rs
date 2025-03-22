use simplelog::*;

use runner::Runner;
use serde_json;

pub mod block;
pub mod filter;
pub mod probe;
pub mod runner;
pub mod wrapper;

use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Prospector description goes here.
pub struct Options {
    /// target runner, optional, supported targets: [local://] local runner, default; [fixture:///path/to/fixture.json] mock runner
    #[argh(option, short = 'r')]
    target: Option<String>,

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

    let mut runner = Runner::new_with_target(&opts.target);
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
