pub mod ast;
pub mod eval;
pub mod goodies;
pub mod lexer;
pub mod parser;

type Version = f32;

// TODO: change this to some type supporting semver
const VERSION: Version = 1.0;
