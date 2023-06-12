use serde::ser::SerializeMap;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(i32),
    Float(f32),
    String(String),
    ObjectWithCalls(Vec<RecordOrCall>),
    Object(Vec<Record>),
    Array(Vec<Value>),
    Call(Call),
    Typed(Typed),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Number(n) => serializer.serialize_i32(*n),
            Self::Float(f) => serializer.serialize_f32(*f),
            Self::String(s) => serializer.serialize_str(s),
            Self::ObjectWithCalls(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for record in v {
                    match record {
                        RecordOrCall::Call(c) => map.serialize_entry(&c.function, &c.value)?,
                        RecordOrCall::Record(r) => map.serialize_entry(&r.id, &r.value)?,
                    }
                }
                map.end()
            }
            Self::Object(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for record in v {
                    map.serialize_entry(&record.id, &record.value)?;
                }
                map.end()
            }
            Self::Call(_) => {
                unimplemented!("calls should be evaluated");
            }
            Self::Typed(t) => t.serialize(serializer),
            Self::Array(a) => serializer.collect_seq(a.iter()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Typed {
    pub kind: String,
    pub value: Box<Value>,
}

impl Serialize for Typed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Call {
    pub function: String,
    pub value: Box<Value>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Record {
    pub id: String,
    pub value: Value,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(untagged)]
pub enum RecordOrCall {
    Record(Record),
    Call(Call),
}

impl From<Record> for RecordOrCall {
    fn from(value: Record) -> Self {
        Self::Record(value)
    }
}

impl From<Call> for RecordOrCall {
    fn from(value: Call) -> Self {
        Self::Call(value)
    }
}
