use regorus::Engine;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::Executable;
use crate::Runner;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct REGOFilter {
    pub expr: String,
    pub args: Option<HashMap<String, Value>>,
}

impl Executable for REGOFilter {
    fn execute(&self, _: &Runner) -> Value {
        let mut engine = Engine::new();
        let args_json = serde_json::to_string(&self.args).unwrap();
        engine
            .add_data(regorus::Value::from_json_str(&args_json).unwrap())
            .unwrap();
        let r = engine.eval_query(self.expr.clone(), true).unwrap();
        let rego_value = &r.result[0].expressions[0].value;
        println!("{:?}", rego_value);
        serde_json::from_str(&rego_value.to_json_str().unwrap()).expect("REGO -> SERDE error!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rego_filter() {
        let f = REGOFilter {
            expr: "1 + data.number".to_string(),
            args: Some(vec![("number".to_string(), 2.into())].into_iter().collect()),
        };

        assert_eq!(f.execute(&Runner::new()), 3);
    }
}
