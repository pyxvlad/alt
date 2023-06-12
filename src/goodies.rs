use crate::{
    ast::{Call, Record, Typed, Value},
    eval::{self, Error as EvalError},
    parser, Version, VERSION,
};
use core::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
enum Error {
    VersionMismatch(Version),
    InvalidUrl(Value),
    ExpectedObject(Value),

    InvalidEntry(String),
    InvalidFunction(String),
    InvalidData(Value),
    Eval(EvalError),
    Parse(parser::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionMismatch(ver) => {
                write!(
                    f,
                    "version mismatch: required {}, we are on {}",
                    ver, VERSION
                )
            }
            Self::InvalidUrl(v) => {
                write!(f, "invalid url \"{v:?}\"")
            }
            Self::ExpectedObject(v) => {
                write!(f, "expected object, found {v:?}")
            }
            Self::InvalidEntry(s) => write!(f, "invalid entry {s}"),
            Self::InvalidFunction(s) => write!(f, "invalid function {s}"),
            Self::InvalidData(d) => write!(f, "invalid data {d:?}"),
            Self::Eval(e) => write!(f, "eval error: {e}"),
            Self::Parse(e) => write!(f, "parsing error: {e}"),
        }
    }
}

impl StdError for Error {}

impl From<Error> for EvalError {
    fn from(value: Error) -> Self {
        EvalError::Eval(Box::new(value))
    }
}

impl From<EvalError> for Error {
    fn from(value: EvalError) -> Self {
        Self::Eval(value)
    }
}

impl From<parser::Error> for Error {
    fn from(value: parser::Error) -> Self {
        Self::Parse(value)
    }
}

fn meta_lang(value: &Value) -> Result<Option<Record>, EvalError> {
    match value {
        Value::Float(version) => {
            if *version != VERSION {
                Err(Error::VersionMismatch(*version).into())
            } else {
                Ok(None)
            }
        }
        _ => Err(Error::InvalidData(value.clone()).into()),
    }
}

fn url(value: &Value) -> Result<Value, EvalError> {
    match value {
        Value::String(_) => Ok(Value::Typed(Typed {
            value: Box::new(value.clone()),
            kind: "std_url".to_string(),
        })),
        _ => Err(Error::InvalidUrl(value.clone()).into()),
    }
}

#[derive(Default)]
pub struct Evaluator {
    value_functions: Vec<Record>,
    record_functions: Vec<Record>,
}

impl Evaluator {
    fn meta_eval(&mut self, value: &Value) -> Result<Option<Record>, EvalError> {
        match value {
            Value::Object(object) => {
                for record in object {
                    match record.id.as_str() {
                        "value" => {
                            if let Value::Object(ref data) = record.value {
                                self.value_functions = data.clone();
                            }
                        }
                        "record" => {
                            if let Value::Object(ref data) = record.value {
                                self.record_functions = data.clone();
                            }
                        }
                        _ => return Err(Error::InvalidEntry(record.id.clone()).into()),
                    };
                }
                Ok(None)
            }
            _ => Err(Error::ExpectedObject(value.clone()).into()),
        }
    }
}

impl eval::Evaluator for Evaluator {
    fn record_function_eval(&mut self, call: &Call) -> Result<Option<Record>, EvalError> {
        match call.function.as_str() {
            "meta-lang" => meta_lang(&call.value),
            "meta-eval" => self.meta_eval(&call.value),
            _ => Err(Error::InvalidFunction("#".to_string() + &call.function).into()),
        }
    }
    fn value_function_eval(&mut self, call: &Call) -> Result<Value, EvalError> {
        match call.function.as_str() {
            "std_url" => url(&call.value),
            _ => Err(Error::InvalidFunction("@".to_string() + &call.function).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::tokenize, parser::parse};

    use super::*;

    #[test]
    fn meta_eval() -> Result<(), Error> {
        use eval::Evaluator as EvalEvaluator;
        let mut evaluator: Evaluator = Default::default();

        let tokens = tokenize("#meta-eval {value = {std_url = @std_url \"localhost\"}}");
        let parsed = parse(&tokens)?;
        let evaluated = evaluator.eval(&parsed)?;
        println!("{evaluated:?}");

        Ok(())
    }
}
