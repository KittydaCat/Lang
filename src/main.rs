mod interpreter;
mod parser;

use std::fs::read_to_string;

fn main() {
    interpreter::top_level_exec(dbg!(&parser::dew_it(
        &read_to_string("./examples/main.lang").unwrap()
    )));
}
