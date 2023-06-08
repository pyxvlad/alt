#[derive(Debug, PartialEq)]
pub enum Value {
    Number(i32),
    Float(f32),
    String(String),
    Object(Vec<Record>),
    Call(Call),
}

#[derive(Debug, PartialEq)]
pub struct Call {
    pub function: String,
    pub value: Box<Value>,
}

#[derive(Debug, PartialEq)]
pub struct Record {
    pub id: String,
    pub value: Value,
}
