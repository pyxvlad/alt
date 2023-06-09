use crate::ast::{Call, Record, RecordOrCall, Typed, Value};
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

pub trait Evaluator {
    fn value_function_eval(&mut self, call: &Call) -> Result<Value, Error>;
    fn record_function_eval(&mut self, call: &Call) -> Result<Option<Record>, Error>;

    fn eval(&mut self, root: &Value) -> Result<Value, Error> {
        match root {
            Value::Call(ref call) => {
                let boxed = Box::new(self.eval(call.value.as_ref())?);
                self.value_function_eval(&Call {
                    value: boxed,
                    function: call.function.to_string(),
                })
            }
            Value::ObjectWithCalls(object) => {
                let mut obj = Vec::new();
                for record in object {
                    match record {
                        RecordOrCall::Record(record) => obj.push(
                            Record {
                                id: record.id.clone(),
                                value: self.eval(&record.value)?,
                            }
                            .into(),
                        ),
                        RecordOrCall::Call(call) => {
                            let boxed = Box::new(self.eval(call.value.as_ref())?);
                            let optional = self.record_function_eval(&Call {
                                function: call.function.to_string(),
                                value: boxed,
                            })?;
                            if let Some(rec) = optional {
                                obj.push(rec.into());
                            }
                        }
                    }
                }

                Ok(Value::Object(obj))
            }
            Value::Typed(t) => {
                let value = self.eval(&t.value)?;
                Ok(Value::Typed(Typed {
                    kind: t.kind.clone(),
                    value: Box::new(value),
                }))
            }
            Value::Float(_) | Value::Number(_) | Value::String(_) | Value::Object(_) => {
                Ok(root.clone())
            }
        }
    }
}

pub fn eval<'a, VF, RF>(
    root: &Value,
    value_functions: &mut VF,
    record_functions: &mut RF,
) -> Result<Value, Error>
where
    VF: FnMut(&Call) -> Result<Value, Error>,
    RF: FnMut(&Call) -> Result<Option<Record>, Error>,
{
    struct T<'a, TVF, TRF>
    where
        TVF: FnMut(&Call) -> Result<Value, Error>,
        TRF: FnMut(&Call) -> Result<Option<Record>, Error>,
    {
        vf: &'a mut TVF,
        rf: &'a mut TRF,
    }

    impl<'a, TVF, TRF> Evaluator for T<'_, TVF, TRF>
    where
        TVF: FnMut(&Call) -> Result<Value, Error>,
        TRF: FnMut(&Call) -> Result<Option<Record>, Error>,
    {
        fn value_function_eval(&mut self, call: &Call) -> Result<Value, Error> {
            (self.vf)(call)
        }
        fn record_function_eval(&mut self, call: &Call) -> Result<Option<Record>, Error> {
            (self.rf)(call)
        }
    }

    let mut t = T{vf: value_functions, rf: record_functions};
    t.eval(root)
}

#[cfg(test)]
mod tests {
    use super::{eval, Error, Record, Value};
    use crate::ast::Call;
    use std::collections::HashMap;

    #[test]
    fn eval_literal() -> Result<(), Error> {
        let root = Value::Number(25);
        type VFRet<'a> = Option<&'a dyn Fn(&Value) -> Result<Value, Error>>;

        let result = eval(&root, &mut |_| Err(Error::InvalidFunction), &mut |_| {
            Err(Error::InvalidFunction)
        })?;
        assert_eq!(root, result);

        Ok(())
    }

    #[test]
    fn eval_call() -> Result<(), Error> {
        let mut call = |v: &Value| return Ok(v.clone());
        let mut functions = HashMap::new();
        functions.insert("call".to_string(), &mut call);
        let value = Value::Number(2);
        let root = Value::Call(Call {
            function: "call".to_string(),
            value: Box::new(value.clone()),
        });
        let result = eval(
            &root,
            &mut |c| {
                functions
                    .get_mut(&c.function)
                    .and_then(|a| Some(a(&c.value)))
                    .ok_or(Error::InvalidFunction)?
            },
            &mut |_| Err(Error::InvalidFunction),
        )?;

        assert_eq!(result, value);
        Ok(())
    }

    #[test]
    fn eval_call_inside_object() -> Result<(), Error> {
        let mut call = |v: &Value| return Ok(v.clone());
        let mut functions = HashMap::new();
        functions.insert("call".to_string(), &mut call);
        let value = Value::Number(2);
        let root = Value::ObjectWithCalls(vec![Record {
            id: "some".to_string(),
            value: Value::Call(Call {
                function: "call".to_string(),
                value: Box::new(value.clone()),
            }),
        }
        .into()]);

        let result = eval(
            &root,
            &mut |c| {
                functions
                    .get_mut(&c.function)
                    .and_then(|a| Some(a(&c.value)))
                    .ok_or(Error::InvalidFunction)?
            },
            &mut |_| Err(Error::InvalidFunction),
        )?;

        assert_eq!(
            result,
            Value::Object(vec![Record {
                id: "some".to_string(),
                value: value
            }
            .into()])
        );

        Ok(())
    }
}
