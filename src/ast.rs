#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(i32),
    Float(f32),
    String(String),
    Object(Vec<Record>),
    Call(Call),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub function: String,
    pub value: Box<Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Record {
    pub id: String,
    pub value: Value,
}
