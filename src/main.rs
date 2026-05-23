mod interpreter;
mod parser;

use std::fs::read_to_string;

fn main() {
    interpreter::exec(dbg!(&parser::dew_it(
        &read_to_string("./examples/main.lang").unwrap()
    )));
}
