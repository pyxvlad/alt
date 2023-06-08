
#[derive(Debug, PartialEq)]
pub enum Value {
    Number(i32),
    Float(f32),
    String(String),
    Object(Vec<Record>),
}

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: String,
    pub value: Value,
}

