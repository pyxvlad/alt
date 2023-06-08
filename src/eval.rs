use crate::ast::{Record, Value};
use std::collections::HashMap;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    InvalidFunction,
    Eval(Box<dyn StdError>),
}

pub type ValueCallFn = fn(&Value) -> Result<Value, Error>;

pub fn eval(root: &Value, functions: &HashMap<String, ValueCallFn>) -> Result<Value, Error> {
    match root {
        Value::Call(ref call) => {
            if let Some(call_fn) = functions.get(&call.function) {
                let value = eval(call.value.as_ref(), functions)?;
                call_fn(&value)
            } else {
                Err(Error::InvalidFunction)
            }
        }
        Value::Object(object) => {
            let mut obj = Vec::new();
            for record in object {
                obj.push(Record {
                    id: record.id.clone(),
                    value: eval(&record.value, functions)?,
                });
            }

            Ok(Value::Object(obj))
        }
        Value::Float(_) | Value::Number(_) | Value::String(_) => Ok(root.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::{eval, Error, Record, Value, ValueCallFn};
    use crate::ast::Call;
    use std::collections::HashMap;

    #[test]
    fn eval_literal() -> Result<(), Error> {
        let root = Value::Number(25);
        let result = eval(&root, &HashMap::new())?;
        assert_eq!(root, result);

        Ok(())
    }

    #[test]
    fn eval_call() -> Result<(), Error> {
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
    fn eval_call_inside_object() -> Result<(), Error> {
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
