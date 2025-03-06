use serde::{Deserialize, Serialize};

use crate::block::Runner;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Probe {
    Process(ProcessProbe),
    File(FileProbe),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProcessProbe {
    pub exec: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FileProbe {
    pub paths: Vec<String>,
}

pub trait Runable {
    fn run(&self, runner: &mut Runner) -> Vec<(String, String)>;
}

impl Runable for Probe {
    fn run(&self, runner: &mut Runner) -> Vec<(String, String)> {
        match self {
            Probe::Process(probe) => probe.run(runner),
            Probe::File(probe) => probe.run(runner),
        }
    }
}

impl Runable for FileProbe {
    fn run(&self, runner: &mut Runner) -> Vec<(String, String)> {
        self.paths
            .iter()
            .map(|path| (path.clone(), runner.cat(&vec![path.clone()])))
            .collect()
    }
}

impl Runable for ProcessProbe {
    fn run(&self, runner: &mut Runner) -> Vec<(String, String)> {
        vec![(self.exec.clone(), runner.run(&self.exec, &self.args))]
    }
}
