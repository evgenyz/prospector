use cel_interpreter::extractors::This;
use cel_interpreter::{Context, ExecutionError, FunctionContext, Program};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::filter::Executable;
use crate::runner::Runner;

type Result<T> = std::result::Result<T, ExecutionError>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CELFilter {
    pub expr: String,
    pub args: Option<Map<String, Value>>,
}

impl Executable for CELFilter {
    fn execute(&self, _: &mut Runner, sources: &Map<String, Value>) -> Value {
        let program = Program::compile(&self.expr).unwrap();
        let mut context = Context::default();
        context.add_variable("args", &self.args).unwrap();
        for kv in sources.iter() {
            context.add_variable(kv.0, &kv.1).unwrap();
        }

        context.add_function("has_value_of", has_value_of);
        let value = program.execute(&context).unwrap();
        //println!("{:?}", value);
        value.json().expect("CEL -> SERDE error!")
    }
}

fn has_value_of(
    ftx: &FunctionContext,
    This(this): This<cel_interpreter::Value>,
    key: cel_interpreter::Value,
    value: cel_interpreter::Value,
) -> Result<bool> {
    let result = match this {
        cel_interpreter::Value::Map(m) => {
            if let Some(v) = m
                .map
                .get(&key.try_into().map_err(ExecutionError::UnsupportedKeyType)?)
            {
                match v {
                    cel_interpreter::Value::String(s) => {
                        if let cel_interpreter::Value::String(v) = value {
                            // FIXME: This is a hack!
                            s.contains(v.as_str())
                        } else {
                            false
                        }
                    }
                    cel_interpreter::Value::Int(n) => {
                        if let cel_interpreter::Value::Int(v) = value {
                            v == *n
                        } else {
                            false
                        }
                    }
                    // TODO: Error!
                    _ => false,
                }
            } else {
                false
            }
        }
        value => return Err(ftx.error(format!("Cannot process {:?}", value))),
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cel_filter() {
        let f = CELFilter {
            expr: "1 + int(args.number)".to_string(),
            args: Some(vec![("number".to_string(), 2.into())].into_iter().collect()),
        };

        let sources = Map::<String, Value>::new();
        assert_eq!(f.execute(&mut Runner::new(), &sources), 3);
    }

    #[test]
    fn cel_filter_with_source() {
        let f = CELFilter {
            expr: "1 + int(src_1)".to_string(),
            args: None,
        };

        let mut sources = Map::<String, Value>::new();
        sources.insert("src_1".to_string(), 3.into());
        assert_eq!(f.execute(&mut Runner::new(), &sources), 4);
    }

    #[test]
    fn cel_filter_has_value_of_a_1() {
        let f = CELFilter {
            expr: "{'a': 1}.has_value_of('a', 1)".to_string(),
            args: None,
        };

        let sources = Map::<String, Value>::new();
        assert_eq!(f.execute(&mut Runner::new(), &sources), true);
    }

    #[test]
    fn cel_filter_has_value_of_s_some() {
        let f = CELFilter {
            expr: "{'s': 'some,other'}.has_value_of('s', 'some')".to_string(),
            args: None,
        };

        let sources = Map::<String, Value>::new();
        assert_eq!(f.execute(&mut Runner::new(), &sources), true);
    }
}
