use crate::ast::{Record, Value};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub enum EvalError {
    InvalidFunction,
    Eval(Box<dyn Error>),
}

pub type ValueCallFn = fn(&Value) -> Result<Value, EvalError>;

pub fn eval(root: &Value, functions: &HashMap<String, ValueCallFn>) -> Result<Value, EvalError> {
    match root {
        Value::Call(ref call) => {
            if let Some(call_fn) = functions.get(&call.function) {
                let value = eval(call.value.as_ref(), functions)?;
                call_fn(&value)
            } else {
                Err(EvalError::InvalidFunction)
            }
        }
        Value::Object(object) => {
            let mut obj = Vec::new();
            for record in object {
                obj.push(Record {
                    id: record.id.to_owned(),
                    value: eval(&record.value, functions)?,
                })
            }

            Ok(Value::Object(obj))
        }
        Value::Float(_) | Value::Number(_) | Value::String(_) => Ok(root.clone()),
    }
}

mod tests {
    use super::*;
    use crate::ast::Call;

    #[test]
    fn eval_literal() -> Result<(), EvalError> {
        let root = Value::Number(25);
        let result = eval(&root, &HashMap::new())?;
        assert_eq!(root, result);

        Ok(())
    }

    #[test]
    fn eval_call() -> Result<(), EvalError> {
        let call: ValueCallFn = |v| return Ok(v.clone());
        let mut functions = HashMap::new();
        functions.insert("call".to_string(), call);
        let value = Value::Number(2);
        let root = Value::Call(Call {
            function: "call".to_string(),
            value: Box::new(value.clone()),
        });

        let result = eval(&root, &functions)?;

        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn eval_call_inside_object() -> Result<(), EvalError> {
        let call: ValueCallFn = |v| return Ok(v.clone());
        let mut functions = HashMap::new();
        functions.insert("call".to_string(), call);
        let value = Value::Number(2);
        let root = Value::Object(vec![Record {
            id: "some".to_string(),
            value: Value::Call(Call {
                function: "call".to_string(),
                value: Box::new(value.clone()),
            }),
        }]);

        let result = eval(&root, &functions)?;

        assert_eq!(
            result,
            Value::Object(vec![Record {
                id: "some".to_string(),
                value: value
            }])
        );

        Ok(())
    }
}
