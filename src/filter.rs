use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub mod cel;
#[cfg(feature = "rego")]
pub mod rego;

use crate::block::Runner;

use crate::filter::cel::CELFilter;
#[cfg(feature = "rego")]
use crate::filter::rego::REGOFilter;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    CEL(CELFilter),
    #[cfg(feature = "rego")]
    REGO(REGOFilter),
}

pub trait Executable {
    fn execute(&self, runner: &mut Runner, sources: &Map<String, Value>) -> Value;
}

impl Executable for Filter {
    fn execute(&self, runner: &mut Runner, sources: &Map<String, Value>) -> Value {
        match self {
            Filter::CEL(cel_filter) => cel_filter.execute(runner, sources),
            #[cfg(feature = "rego")]
            Filter::REGO(rego_filter) => rego_filter.execute(runner, sources),
        }
    }
}
