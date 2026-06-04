use std::collections::HashMap;

use crate::parser::{FunDefinition, Statement, TypedValue, ValueSource};

#[derive(Clone, Copy, Debug, PartialEq)]
enum RuntimeValue {
    Number(isize),
    Object(RuntimeID),
    None,
}

#[derive(Clone, Debug)]
enum RuntimeObject {
    List(Vec<RuntimeValue>),
    Struct(Vec<(String, RuntimeValue)>),
    Function(String),
    Enum(String, Option<RuntimeValue>),
}

#[derive(Eq, Hash, Clone, Copy, Debug, PartialEq)]
struct RuntimeID(usize);

#[derive(Clone, Default)]
struct State {
    functions: HashMap<String, FunDefinition>,

    vars: HashMap<String, RuntimeValue>,

    values: HashMap<RuntimeID, RuntimeObject>,
    curr_index: usize,
}

impl State {
    fn add_object(&mut self, object: RuntimeObject) -> RuntimeValue {
        let index = RuntimeID(self.curr_index);
        self.curr_index += 1;

        // let runtime_value = values.iter().map(|x| calc_value(x, state)).collect();

        assert!(self.values.insert(index, object).is_none());

        RuntimeValue::Object(index)
    }
}

pub fn top_level_exec(statements: &Vec<Statement>) {
    assert_eq!(exec(statements, &mut State::default()), RuntimeValue::None);
}

fn exec(statements: &Vec<Statement>, mut state: &mut State) -> RuntimeValue {
    for statement in statements {
        match statement {
            Statement::Value(value) => {
                assert_eq!(RuntimeValue::None, calc_value(value, &mut state))
            }
            Statement::VarDefinition(name, value) => {
                let val = calc_value(value, &mut state);

                assert!(state.vars.insert(name.clone(), val).is_none())
            }
            Statement::FunDefinition(name, def) => {
                assert!(state.functions.insert(name.clone(), def.clone()).is_none());
            }
            Statement::Ret(value) => return calc_value(value, &mut state),
            Statement::StructDefinition(name, members) => {}
            Statement::EnumDefinition(name, members) => {}
        }
    }

    RuntimeValue::None
}

fn calc_value(value: &TypedValue, state: &mut State) -> RuntimeValue {
    match &value.val_source {
        ValueSource::FunctionCall { name, args } => {
            let mut args: Vec<_> = args.iter().map(|x| calc_value(x, state)).collect();

            match name.as_str() {
                "print" => {
                    for x in args {
                        match x {
                            RuntimeValue::Number(num) => print!("{num}"),
                            RuntimeValue::Object(id) => {
                                print!("{:?}", state.values.get(&id).unwrap())
                            }
                            RuntimeValue::None => todo!(),
                        }
                    }

                    RuntimeValue::None
                }

                "add" => {
                    let [RuntimeValue::Number(x), RuntimeValue::Number(y)] = args[..] else {
                        unimplemented!()
                    };

                    RuntimeValue::Number(x + y)
                }

                "subtract" => {
                    let [RuntimeValue::Number(x), RuntimeValue::Number(y)] = args[..] else {
                        unimplemented!()
                    };

                    RuntimeValue::Number(x - y)
                }

                "index" => {
                    let [RuntimeValue::Object(x), RuntimeValue::Number(y)] = args[..] else {
                        unimplemented!()
                    };

                    let RuntimeObject::List(list) = state.values.get(&x).unwrap() else {
                        unimplemented!("{:?}", state.values.get(&x))
                    };

                    *(list.get(y as usize).unwrap_or(&RuntimeValue::None))
                }

                func => {
                    // TODO
                    let func = if let Some(fun) = state.functions.get(func) {
                        fun
                    } else if let Some(RuntimeValue::Object(id)) = state.vars.get(func) {
                        let RuntimeObject::Function(fun) = state.values.get(id).unwrap() else {
                            unimplemented!()
                        };

                        state.functions.get(fun).unwrap()
                    } else {
                        unimplemented!()
                    };

                    let mut new_state = state.clone();

                    let mut arg_names = func.args.iter();
                    let mut args_vals = args.into_iter();

                    while let (Some(arg), Some(val)) = (arg_names.next(), args_vals.next()) {
                        assert!(new_state.vars.insert(arg.0.clone(), val).is_none());
                    }

                    exec(&func.statements, &mut new_state)
                }
            }
        }

        ValueSource::NumberLiteral(x) => RuntimeValue::Number(*x),
        ValueSource::Variable(name) => *state.vars.get(name).unwrap(),
        ValueSource::None => RuntimeValue::None,
        ValueSource::ListLiteral(values) => {
            let values = values.iter().map(|x| calc_value(x, state)).collect();

            state.add_object(RuntimeObject::List(values))
        }
        ValueSource::MemberAccess { name, item } => {
            let RuntimeValue::Object(id) = calc_value(item, state) else {
                unimplemented!()
            };

            let RuntimeObject::Struct(values) = state.values.get(&id).unwrap() else {
                unimplemented!()
            };

            values.iter().find(|x| x.0 == *name).unwrap().1
        }
        ValueSource::StructConstruction(items) => {
            let struc = items
                .iter()
                .map(|(name, val)| (name.clone(), calc_value(val, state)))
                .collect::<Vec<_>>();

            state.add_object(RuntimeObject::Struct(struc))
        }
        ValueSource::Function(x) => state.add_object(RuntimeObject::Function(x.clone())),
        ValueSource::EnumConstruction(enum_member, op) => {
            let object = RuntimeObject::Enum(
                enum_member.clone(),
                if let Some(val) = op {
                    Some(calc_value(val, state))
                } else {
                    None
                },
            );
            state.add_object(object)
        }
        ValueSource::Match(typed_value, items) => {
            let RuntimeValue::Object(index) = calc_value(typed_value, state) else {
                unimplemented!()
            };

            let RuntimeObject::Enum(enum_member, option_val) = state.values.get(&index).unwrap()
            else {
                unimplemented!()
            };

            let match_arm = items.iter().find(|x| &x.0 == enum_member).unwrap();

            if let Some(enum_var) = &match_arm.1 {
                assert!(
                    state
                        .vars
                        .insert(enum_var.clone(), option_val.unwrap())
                        .is_none()
                );

                let val = exec(&match_arm.2, state);

                assert!(state.vars.remove(enum_var).is_some());

                val
            } else {
                exec(&match_arm.2, state)
            }
        }
    }
}
