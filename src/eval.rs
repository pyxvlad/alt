use crate::ast::{Record, RecordOrCall, Typed, Value};
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
pub type RecordCallFn<'a> = &'a dyn Fn(&Value) -> Result<Record, Error>;


pub fn eval<'a, VF, RF>(root: &'a Value, value_functions: &VF, record_functions: &RF) -> Result<Value, Error>
where
    VF: Fn(&str) -> Option<ValueCallFn<'a>>,
    RF: Fn(&str) -> Option<RecordCallFn<'a>>,
{
    match root {
        Value::Call(ref call) => {
            if let Some(call_fn) = value_functions(&call.function) {
                let value = eval(call.value.as_ref(), value_functions, record_functions)?;
                call_fn(&value)
            } else {
                Err(Error::InvalidFunction)
            }
        }
        Value::Object(object) => {
            let mut obj = Vec::new();
            for record in object {
                match record {
                    RecordOrCall::Record(record) => obj.push(
                        Record {
                            id: record.id.clone(),
                            value: eval(&record.value, value_functions, record_functions)?,
                        }
                        .into(),
                    ),
                    RecordOrCall::Call(call) => {
                        if let Some(call_fn) = record_functions(&call.function) {
                            let value = eval(call.value.as_ref(), value_functions, record_functions)?;
                            let rec = call_fn(&value)?;
                            obj.push(rec.into());
                        }
                    },
                }
            }

            Ok(Value::Object(obj))
        }
        Value::Typed(t) => {
            let value = eval(&t.value, value_functions, record_functions)?;
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
        let result = eval(&root, &|_| None, &|_|None)?;
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

        let result = eval(&root, &move |s| functions.get(s).copied(), &|_|None)?;

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
        }.into()]);

        let result = eval(&root, &move |s| functions.get(s).copied(), &|_|None)?;

        assert_eq!(
            result,
            Value::Object(vec![Record {
                id: "some".to_string(),
                value: value
            }.into()])
        );

        Ok(())
    }
}
