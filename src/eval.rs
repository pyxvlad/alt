use crate::ast::{Record, Typed, Value};
use std::error::Error as StdError;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidFunction,
    Eval(Box<dyn StdError>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFunction => write!(f, "invalid function"),
            Self::Eval(err) => write!(f, "function eval error: {err}"),
        }
    }
}

impl StdError for Error {}

pub type ValueCallFn<'a> = &'a dyn Fn(&Value) -> Result<Value, Error>;

pub fn eval<'a, F>(root: &'a Value, functions: &F) -> Result<Value, Error>
where
    F: Fn(&str) -> Option<ValueCallFn<'a>>,
{
    match root {
        Value::Call(ref call) => {
            if let Some(call_fn) = functions(&call.function) {
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
        Value::Typed(t) => {
            let value = eval(&t.value, functions)?;
            Ok(Value::Typed(Typed {
                kind: t.kind.clone(),
                value: Box::new(value),
            }))
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
        let result = eval(&root, &|_| None)?;
        assert_eq!(root, result);

        Ok(())
    }

    #[test]
    fn eval_call() -> Result<(), Error> {
        let call: ValueCallFn = &|v| return Ok(v.clone());
        let mut functions = HashMap::new();
        functions.insert("call".to_string(), call);
        let value = Value::Number(2);
        let root = Value::Call(Call {
            function: "call".to_string(),
            value: Box::new(value.clone()),
        });

        let result = eval(&root, &move |s| functions.get(s).copied())?;

        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn eval_call_inside_object() -> Result<(), Error> {
        let call: ValueCallFn = &|v| return Ok(v.clone());
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

        let result = eval(&root, &move |s| functions.get(s).copied())?;

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
