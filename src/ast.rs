use serde::ser::SerializeMap;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(i32),
    Float(f32),
    String(String),
    Object(Vec<Record>),
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
