use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq)]
enum Token {
    Name(String),

    Let,
    Fun,
    Ret,
    Struct,
    Enum,

    Number(isize),

    LParens,
    RParens,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LAngle,
    RAngle,

    Comma,
    Equals,
    SemiColon,
    Dot,
    Colon,

    Plus,
    Minus,
}

pub fn dew_it(file: &str) -> Vec<Statement> {
    parse(
        &mut dbg!(tokenize(file)).into_iter().peekable(),
        &mut State::new(),
        Type::None,
    )
}

fn tokenize(file: &str) -> Vec<Token> {
    let mut cars = file.chars();

    let mut string = String::new();

    let mut tokens = Vec::new();

    loop {
        let next = cars.next();

        if let Some(c @ ('0'..='9' | 'a'..='z' | 'A'..='Z' | '_')) = next {
            string.push(c);
        } else if !string.is_empty() {
            if matches!(string.chars().next().unwrap(), '0'..='9') {
                tokens.push(Token::Number(string.parse().unwrap()));

                string = String::new();
            } else {
                tokens.push(match string.as_str() {
                    "let" => Token::Let,
                    "fun" => Token::Fun,
                    "ret" => Token::Ret,
                    "struct" => Token::Struct,
                    "enum" => Token::Enum,

                    _ => Token::Name(string),
                });

                string = String::new();
            }
        }

        match next {
            Some('0'..='9' | 'a'..='z' | 'A'..='Z' | '_') => {}

            Some('(') => tokens.push(Token::LParens),
            Some(')') => tokens.push(Token::RParens),
            Some('{') => tokens.push(Token::LBrace),
            Some('}') => tokens.push(Token::RBrace),
            Some('[') => tokens.push(Token::LBracket),
            Some(']') => tokens.push(Token::RBracket),
            Some('<') => tokens.push(Token::LAngle),
            Some('>') => tokens.push(Token::RAngle),

            Some(',') => tokens.push(Token::Comma),
            Some(';') => tokens.push(Token::SemiColon),
            Some(':') => tokens.push(Token::Colon),

            Some('=') => tokens.push(Token::Equals),
            Some('+') => tokens.push(Token::Plus),
            Some('-') => tokens.push(Token::Minus),
            Some('.') => tokens.push(Token::Dot),

            Some('\t' | '\n' | ' ') => {}

            // unknown
            Some(x) => {
                todo!("{x}")
            }

            None => {
                break;
            }
        }
    }

    tokens
}

#[derive(Debug, Clone)]
pub enum Statement {
    Value(TypedValue),

    VarDefinition(String, TypedValue),
    FunDefinition(String, FunDefinition),

    StructDefinition(String, Vec<(String, Type)>),
    // EnumDefinition(String, Vec<(String, Type)>),
    Ret(TypedValue),
}

#[derive(Debug, Clone)]
pub struct FunDefinition {
    pub args: Vec<(String, Type)>,
    pub statements: Vec<Statement>,
    pub type_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TypedValue {
    pub val_type: Type,
    pub val_source: ValueSource,
}

#[derive(Debug, Clone)]
pub enum ValueSource {
    FunctionCall { name: String, args: Vec<TypedValue> },

    StructConstruction(Vec<(String, TypedValue)>),
    MemberAccess { name: String, item: Box<TypedValue> },

    NumberLiteral(isize),
    ListLiteral(Vec<TypedValue>),

    Variable(String),
    Function(String),
    None,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Type {
    Int,
    None,

    List(Box<Type>),
    StructName(String),
    Function(Box<Sig>),

    TypeVar(String),
}

impl Type {
    fn is_list(&self) -> bool {
        match self {
            Type::List(_) => true,
            _ => false,
        }
    }

    fn instantiate(&self, type_names: &[String], types: &[Type]) -> Type {
        match self {
            Type::List(x) => Type::List(Box::new(x.instantiate(type_names, types))),
            t @ Type::TypeVar(x) => {
                if let Some(new_type) = type_names.iter().position(|y| x == y) {
                    types[new_type].clone()
                } else {
                    t.clone()
                }
            }
            t @ (Type::Int | Type::None | Type::StructName(_)) => t.clone(),
            Type::Function(sig) => Type::Function(Box::new(sig.instantiate(type_names, types))),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Sig {
    args: Vec<Type>,
    ret_type: Type,
    type_names: Vec<String>,
}

impl Sig {
    fn matches(&self, args: &[TypedValue], type_params: &[Type]) -> bool {
        let mut expected_args = self
            .args
            .iter()
            .map(|x| x.instantiate(&self.type_names, type_params));

        let mut args_iter = args.iter().map(|x| &x.val_type);

        loop {
            if let Some(arg) = args_iter.next() {
                assert_eq!(arg, &expected_args.next().unwrap());
            } else {
                assert!(expected_args.next().is_none());
                return true;
            }
        }
    }

    fn instantiate(&self, type_names: &[String], types: &[Type]) -> Sig {
        Sig {
            args: self
                .args
                .iter()
                .map(|x| x.instantiate(type_names, types))
                .collect(),
            ret_type: self.ret_type.instantiate(type_names, types),
            type_names: self.type_names.clone(),
        }
    }
}

#[derive(Default, Debug)]
pub struct SubState {
    var_to_id: HashMap<String, Type>,
    type_params: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct State {
    states: Vec<SubState>,

    // functions
    function_to_sig: HashMap<String, Sig>,

    // structs
    struct_def: HashMap<String, Vec<(String, Type)>>,

    // inprogress types
    inprogress: HashSet<String>,
}

impl State {
    fn new() -> Self {
        let mut state = State {
            states: vec![SubState {
                ..Default::default()
            }],
            function_to_sig: {
                let mut hash = HashMap::new();

                hash.insert(
                    String::from("add"),
                    Sig {
                        args: vec![Type::Int, Type::Int],
                        ret_type: Type::Int,
                        type_names: Vec::new(),
                    },
                );

                hash.insert(
                    String::from("subtract"),
                    Sig {
                        args: vec![Type::Int, Type::Int],
                        ret_type: Type::Int,
                        type_names: Vec::new(),
                    },
                );

                hash.insert(
                    String::from("index"),
                    Sig {
                        args: vec![
                            Type::List(Box::new(Type::TypeVar(String::from("IndexType")))),
                            Type::Int,
                        ],
                        ret_type: Type::TypeVar(String::from("IndexType")),
                        type_names: vec![String::from("IndexType")],
                    },
                );

                hash.insert(
                    String::from("print"),
                    Sig {
                        args: vec![Type::TypeVar(String::from("PrintType"))],
                        ret_type: Type::None,
                        type_names: vec![String::from("PrintType")],
                    },
                );

                hash
            },
            ..Default::default()
        };

        state
    }

    fn var_to_id(&self, name: &str) -> Option<&Type> {
        self.states.iter().rev().find_map(|x| x.var_to_id.get(name))
    }

    fn get_sig(&self, name: &str) -> Option<&Sig> {
        if let Some(sig) = self.function_to_sig.get(name) {
            Some(sig)
        } else if let Some(Type::Function(sig)) = self.var_to_id(name) {
            Some(sig)
        } else {
            None
        }
    }

    // fn func_sig_to_ret(
    //     &self,
    //     name: &str,
    //     params: &mut impl Iterator<Item = Type>,
    //     type_params: &[Type],
    // ) -> Option<Type> {
    //     match name {
    //         "index" => {
    //             // assert_eq!(types.next().unwrap(), Type::LIST);
    //             let Type::List(list_type) = params.next().unwrap() else {
    //                 unimplemented!()
    //             };
    //             assert_eq!(params.next().unwrap(), Type::Int);
    //             assert!(params.next().is_none());
    //             Some(*list_type)
    //         }
    //
    //         "add" => {
    //             assert_eq!(params.next().unwrap(), Type::Int);
    //             assert_eq!(params.next().unwrap(), Type::Int);
    //             assert!(params.next().is_none());
    //             Some(Type::Int)
    //         }
    //
    //         "subtract" => {
    //             assert_eq!(params.next().unwrap(), Type::Int);
    //             assert_eq!(params.next().unwrap(), Type::Int);
    //             assert!(params.next().is_none());
    //             Some(Type::Int)
    //         }
    //
    //         "print" => Some(Type::None),
    //
    //         _ => {
    //             dbg!(name);
    //             if let Some(sig) = self.function_to_sig.get(name) {
    //                 assert!(sig.matches(params, type_params));
    //
    //                 if !type_params.is_empty() {
    //                     Some(sig.ret_type.instantiate(&sig.type_names, type_params))
    //                 } else {
    //                     Some(sig.ret_type.clone())
    //                 }
    //             } else if let Some(Type::Function(sig)) = self.var_to_id(name) {
    //                 assert!(sig.matches(params, type_params));
    //
    //                 if !type_params.is_empty() {
    //                     Some(sig.ret_type.instantiate(&sig.type_names, type_params))
    //                 } else {
    //                     Some(sig.ret_type.clone())
    //                 }
    //             } else {
    //                 None
    //             }
    //         }
    //     }
    // }

    fn id_to_struct(&self, struct_name: &str) -> Option<&[(String, Type)]> {
        self.struct_def.get(struct_name).map(|y| y.as_slice())
    }

    fn name_to_type(&self, type_name: &str) -> Option<Type> {
        if self.struct_def.get(type_name).is_some() {
            Some(Type::StructName(String::from(type_name)))
        } else if self.states.last().unwrap().type_params.contains(type_name) {
            Some(Type::TypeVar(String::from(type_name)))
        } else if self.inprogress.contains(type_name) {
            Some(Type::StructName(String::from(type_name)))
        } else {
            None
        }
    }

    fn push(&mut self) {
        self.states.push(SubState::default());
    }

    fn pop(&mut self) -> SubState {
        self.states.pop().unwrap()
    }
}

fn parse(
    tokens: &mut Peekable<IntoIter<Token>>,
    state: &mut State,
    ret_type: Type,
) -> Vec<Statement> {
    let mut statements = Vec::new();

    loop {
        dbg!(&statements);
        match tokens.peek() {
            Some(Token::Let) => {
                assert_eq!(Some(Token::Let), tokens.next());

                let Some(Token::Name(name)) = tokens.next() else {
                    unimplemented!()
                };

                assert_eq!(Some(Token::Colon), tokens.next());

                let val_type = parse_type(tokens, state);

                assert_eq!(Some(Token::Equals), tokens.next());

                let value = parse_value(tokens, state, &val_type);

                assert!(
                    state
                        .states
                        .last_mut()
                        .unwrap()
                        .var_to_id
                        .insert(name.clone(), value.val_type.clone())
                        .is_none()
                );

                statements.push(Statement::VarDefinition(name, value));
            }

            Some(Token::Fun) => {
                assert_eq!(Some(Token::Fun), tokens.next());

                let mut type_params = Vec::new();

                // get type_params
                if let Token::LAngle = tokens.peek().unwrap() {
                    assert_eq!(tokens.next().unwrap(), Token::LAngle);

                    while Token::RAngle != *tokens.peek().unwrap() {
                        let Token::Name(param) = tokens.next().unwrap() else {
                            unimplemented!()
                        };

                        type_params.push(param);

                        if Token::Comma == *tokens.peek().unwrap() {
                            assert_eq!(Token::Comma, tokens.next().unwrap());
                        }
                    }

                    assert_eq!(Token::RAngle, tokens.next().unwrap());
                }

                state.push();

                let hash_set = &mut state.states.last_mut().unwrap().type_params;

                for param in &type_params {
                    assert!(hash_set.insert(param.clone()));
                }

                let ret_type = parse_type(tokens, state);

                let Some(Token::Name(name)) = tokens.next() else {
                    unimplemented!()
                };

                assert_eq!(Some(Token::LParens), tokens.next());

                let mut args = Vec::new();

                while *tokens.peek().unwrap() != Token::RParens {
                    let arg_type = parse_type(tokens, state);

                    let Token::Name(arg) = tokens.next().unwrap() else {
                        unimplemented!()
                    };

                    args.push((arg.clone(), arg_type.clone()));

                    assert!(
                        state
                            .states
                            .last_mut()
                            .unwrap()
                            .var_to_id
                            .insert(arg, arg_type)
                            .is_none()
                    );

                    if Token::Comma == *tokens.peek().unwrap() {
                        assert_eq!(Some(Token::Comma), tokens.next());
                    }
                }

                assert_eq!(Some(Token::RParens), tokens.next());

                // parse the func statements

                assert_eq!(Some(Token::LBrace), tokens.next());

                let func_statements = parse(tokens, state, ret_type.clone());

                state.pop();

                assert_eq!(Some(Token::RBrace), tokens.next());

                let arg_args = Sig {
                    args: args.iter().map(|x| x.1.clone()).collect(),
                    type_names: type_params.clone(),
                    ret_type: ret_type.clone(),
                };

                assert!(
                    state
                        .function_to_sig
                        .insert(name.clone(), arg_args)
                        .is_none()
                );

                statements.push(Statement::FunDefinition(
                    name,
                    FunDefinition {
                        args,
                        statements: func_statements,
                        type_params,
                    },
                ));
            }

            Some(Token::Struct) => {
                assert_eq!(Some(Token::Struct), tokens.next());

                let Some(Token::Name(name)) = tokens.next() else {
                    unimplemented!()
                };

                assert!(state.inprogress.insert(name.clone()));

                let mut members = Vec::new();

                assert_eq!(Some(Token::LBrace), tokens.next());

                while Token::RBrace != *tokens.peek().unwrap() {
                    let var_type = parse_type(tokens, state);
                    let Token::Name(var_name) = tokens.next().unwrap() else {
                        unimplemented!();
                    };

                    members.push((var_name, var_type));
                    // args.push();

                    if Token::Comma == *tokens.peek().unwrap() {
                        assert_eq!(Token::Comma, tokens.next().unwrap());
                    }
                }

                assert!(state.inprogress.remove(&name));

                assert!(
                    state
                        .struct_def
                        .insert(name.clone(), members.clone())
                        .is_none()
                );

                statements.push(Statement::StructDefinition(name, members));

                assert_eq!(Some(Token::RBrace), tokens.next());
            }

            Some(Token::RBrace) | None => break,

            Some(Token::Ret) => {
                assert_eq!(Token::Ret, tokens.next().unwrap());

                let val = parse_value(tokens, state, &ret_type);

                // assert_eq!(val.val_type, ret_type);

                statements.push(Statement::Ret(val))
            }

            Some(_) => {
                let value = parse_value(tokens, state, &Type::None);

                assert_eq!(value.val_type, Type::None);

                statements.push(Statement::Value(value));
            }
        }

        assert_eq!(Some(Token::SemiColon), tokens.next(), "{tokens:?}");
    }

    statements
}

// this currently will fail if we do something stupid like [[1, 2, 3]][0]
fn parse_value(
    tokens: &mut Peekable<IntoIter<Token>>,
    state: &State,
    target_type: &Type,
) -> TypedValue {
    let mut first_val = match tokens.next().unwrap() {
        Token::Name(name) => {
            if let Some(Token::LParens) = tokens.peek() {
                let sig = state.get_sig(&name).unwrap().clone();

                let args = parse_function_call(tokens, state, &sig);

                assert!(sig.matches(args.as_slice(), &[]));

                TypedValue {
                    val_source: ValueSource::FunctionCall { name, args },
                    val_type: sig.ret_type,
                }
            } else if let Some(Token::LAngle) = tokens.peek() {
                assert_eq!(Token::LAngle, tokens.next().unwrap());

                let mut filled_types = Vec::new();

                while *tokens.peek().unwrap() != Token::RAngle {
                    filled_types.push(parse_type(tokens, state));

                    if Token::Comma == *tokens.peek().unwrap() {
                        assert_eq!(tokens.next().unwrap(), Token::Comma);
                    }
                }

                assert_eq!(tokens.next().unwrap(), Token::RAngle);

                let sig = state.get_sig(&name).unwrap();

                let args = parse_function_call(tokens, state, sig);

                assert!(sig.matches(args.as_slice(), filled_types.as_slice()));

                TypedValue {
                    val_source: ValueSource::FunctionCall { name, args },
                    val_type: sig.ret_type.clone(),
                }

                // let args = parse_function_call(tokens, state);
                //
                // TypedValue {
                //     val_source: ValueSource::FunctionCall { name, args },
                //     val_type: ret,
                // }
            } else if let Some(val_type) = state.var_to_id(&name) {
                TypedValue {
                    val_type: val_type.clone(),
                    val_source: ValueSource::Variable(name),
                }
            } else if let Some(_) = state.id_to_struct(&name) {
                // struct con
                assert_eq!(Token::LBrace, tokens.next().unwrap());

                let mut struct_def = state.id_to_struct(&name).unwrap().iter();

                let mut struct_con = Vec::new();

                while *tokens.peek().unwrap() != Token::RBrace {
                    let Token::Name(member_name) = tokens.next().unwrap() else {
                        unimplemented!()
                    };

                    let struct_item = struct_def.next().unwrap();

                    assert_eq!(&struct_item.0, &member_name);

                    assert_eq!(Token::Colon, tokens.next().unwrap());

                    let value = parse_value(tokens, state, &struct_item.1);

                    struct_con.push((member_name, value));

                    if Token::Comma == *tokens.peek().unwrap() {
                        assert_eq!(Token::Comma, tokens.next().unwrap());
                    }
                }

                assert!(struct_def.next().is_none());

                assert_eq!(Token::RBrace, tokens.next().unwrap());

                TypedValue {
                    val_type: Type::StructName(name),
                    val_source: ValueSource::StructConstruction(struct_con),
                }
            } else if let Some(sig) = state.function_to_sig.get(&name) {
                TypedValue {
                    val_type: Type::Function(Box::new(sig.clone())),
                    val_source: ValueSource::Function(name),
                }
            } else {
                unimplemented!("{tokens:?}")
            }
        }

        Token::Number(x) => TypedValue {
            val_type: Type::Int,
            val_source: ValueSource::NumberLiteral(x),
        },

        Token::LBracket => {
            let mut values = Vec::new();

            let Type::List(item_type) = target_type else {
                unimplemented!()
            };

            while Token::RBracket != *tokens.peek().unwrap() {
                let value = parse_value(tokens, state, item_type);

                // assert_eq!(value.val_type, Type::Int);

                values.push(value);

                if Token::RBracket != *tokens.peek().unwrap() {
                    assert_eq!(Token::Comma, tokens.next().unwrap());
                }
            }

            assert_eq!(Token::RBracket, tokens.next().unwrap());

            TypedValue {
                val_source: ValueSource::ListLiteral(values),
                val_type: target_type.clone(),
            }
        }

        x => {
            dbg!(tokens);
            todo!("{x:?}");
        }
    };

    loop {
        let updated = if let Token::Minus | Token::Plus = tokens.peek().unwrap() {
            let op = tokens.next().unwrap();

            let name = match op {
                Token::Minus => String::from("subtract"),
                Token::Plus => String::from("add"),
                _ => unreachable!(),
            };

            let args = vec![first_val, parse_value(tokens, state, &Type::Int)];

            assert_eq!(args[0].val_type, Type::Int);
            assert_eq!(args[1].val_type, Type::Int);

            TypedValue {
                val_source: ValueSource::FunctionCall { name, args },
                val_type: Type::Int,
            }
        } else if Token::LBracket == *tokens.peek().unwrap() {
            assert_eq!(Token::LBracket, tokens.next().unwrap());

            let Type::List(sub_type) = &first_val.val_type else {
                unimplemented!()
            };

            let value = parse_value(tokens, state, &Type::Int);

            assert_eq!(Token::RBracket, tokens.next().unwrap());

            TypedValue {
                val_source: ValueSource::FunctionCall {
                    name: String::from("index"),
                    args: vec![first_val.clone(), value],
                },
                val_type: *sub_type.clone(),
            }
        } else if Token::Dot == *tokens.peek().unwrap() {
            assert_eq!(Token::Dot, tokens.next().unwrap());

            let TypedValue {
                val_type: Type::StructName(struct_name),
                val_source: _,
            } = &first_val
            else {
                unimplemented!()
            };

            let Token::Name(name) = tokens.next().unwrap() else {
                unimplemented!()
            };

            TypedValue {
                val_type: state
                    .id_to_struct(struct_name)
                    .unwrap()
                    .iter()
                    .find(|x| x.0 == name)
                    .unwrap()
                    .1
                    .clone(),
                val_source: ValueSource::MemberAccess {
                    name,
                    item: Box::new(first_val),
                },
            }
        } else {
            assert_eq!(&first_val.val_type, target_type);

            return first_val;
        };

        first_val = updated;
    }
}

fn parse_type(tokens: &mut Peekable<IntoIter<Token>>, state: &State) -> Type {
    match tokens.next().unwrap() {
        Token::Name(name) if name == "none" => Type::None,
        Token::Name(name) if name == "int" => Type::Int,
        Token::Name(name) => state
            .name_to_type(&name)
            .unwrap_or_else(|| panic!("{name}")),

        Token::LBracket => {
            let sub_type = parse_type(tokens, state);

            assert_eq!(tokens.next().unwrap(), Token::RBracket);

            Type::List(Box::new(sub_type))
        }

        Token::LParens => {
            let mut args = Vec::new();

            while *tokens.peek().unwrap() != Token::RParens {
                args.push(parse_type(tokens, state));

                match tokens.peek().unwrap() {
                    Token::Comma => {
                        assert_eq!(tokens.next(), Some(Token::Comma));
                    }

                    Token::RParens => {}

                    _ => unimplemented!(),
                }
            }

            assert_eq!(tokens.next().unwrap(), Token::RParens);

            assert_eq!(tokens.next().unwrap(), Token::Minus);
            assert_eq!(tokens.next().unwrap(), Token::RAngle);

            let ret_type = parse_type(tokens, state);

            // TODO

            Type::Function(Box::new(Sig {
                args,
                ret_type,
                type_names: Vec::new(),
            }))
        }

        x => unimplemented!("{x:?}"),
    }
}

fn parse_function_call(
    tokens: &mut Peekable<IntoIter<Token>>,
    state: &State,
    sig: &Sig,
) -> Vec<TypedValue> {
    let mut args = Vec::new();

    assert_eq!(tokens.next(), Some(Token::LParens));

    let mut items = sig.args.iter();

    while *tokens.peek().unwrap() != Token::RParens {
        args.push(parse_value(tokens, state, items.next().unwrap()));

        match tokens.peek().unwrap() {
            Token::Comma => {
                assert_eq!(tokens.next(), Some(Token::Comma));
            }

            Token::RParens => {}

            _ => unimplemented!(),
        }
    }

    assert!(items.next().is_none());
    assert_eq!(tokens.next(), Some(Token::RParens));

    args
}
