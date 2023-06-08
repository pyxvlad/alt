mod ast;
mod lexer;
mod parser;

use parser::{parse, ParseError};
use std::io;
use std::io::BufRead;

fn main() -> Result<(), ParseError> {
    println!("Hello, world!");

    let stdin = io::stdin();
    let s = stdin
        .lock()
        .lines()
        .map(|x| x.unwrap() + "\n")
        .collect::<Vec<String>>()
        .concat();
    let tokens = lexer::tokenize(&s);
    println!("{:?}", tokens);
    println!("I parsed:");

    let records = parse(&tokens)?;
    println!("{:?}", records);

    for ele in records {
        println!("\t\"{}\": {:?}", ele.id, ele.value);
    }

    Ok(())
}
