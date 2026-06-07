mod interpreter;
mod parser;
use std::fs::read_to_string;

struct TestIO {
    string: String,
}

impl interpreter::IO for TestIO {}

impl std::fmt::Write for TestIO {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.string.write_str(s)
    }
}

struct PrintIO {}

impl interpreter::IO for PrintIO {}

impl std::fmt::Write for PrintIO {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        print!("{s}");
        Ok(())
    }
}

fn main() {
    interpreter::top_level_exec(
        dbg!(&parser::dew_it(
            &read_to_string("./examples/main.lang").unwrap()
        )),
        &mut PrintIO {},
    );
}

fn test(code: &str, expected: &str) {
    let mut io = TestIO {
        string: String::new(),
    };

    interpreter::top_level_exec(dbg!(&parser::dew_it(code)), &mut io);

    assert_eq!(io.string, expected);
}

#[cfg(test)]
mod test {
    use crate::test;

    #[test]
    fn option() {
        test(
            "
let x: int = 2;
let y: int = 2;

print<int>(x + y);
",
            "4",
        )
    }
}
