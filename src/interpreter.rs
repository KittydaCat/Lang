use crate::parser::{Statement, Value};

#[derive(Debug, PartialEq)]
enum RuntimeValue {
    Number(f64),
    None,
}

pub fn exec(statements: &Vec<Statement>) {
    for statement in statements {
        match statement {
            Statement::Value(value) => assert_eq!(RuntimeValue::None, calc_value(value)),
        }
    }
}

fn calc_value(value: &Value) -> RuntimeValue {
    match value {
        Value::Function { name, args } => {
            let args = args.iter().map(calc_value);

            match name.as_str() {
                "print" => {
                    args.for_each(|x| match x {
                        RuntimeValue::Number(num) => print!("{num}"),
                        RuntimeValue::None => todo!(),
                    });

                    RuntimeValue::None
                }

                _ => todo!(),
            }
        }
        Value::Number(x) => RuntimeValue::Number(*x),
    }
}
