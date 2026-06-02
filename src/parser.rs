use std::collections::HashMap;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq)]
enum Token {
    Name(String),

    Let,
    Fun,
    Ret,
    Struct,

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
        TypeId::NONE,
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
    StructDefinition(String, Vec<(String, TypeId)>),

    Ret(TypedValue),
}

#[derive(Debug, Clone)]
pub struct FunDefinition {
    pub args: Vec<(String, TypeId)>,
    pub statements: Vec<Statement>,
    pub type_params: Vec<TypeId>,
}

#[derive(Debug, Clone)]
pub struct TypedValue {
    pub val_type: TypeId,
    pub val_source: ValueSource,
}

#[derive(Debug, Clone)]
pub enum ValueSource {
    Function { name: String, args: Vec<TypedValue> },

    StructConstruction(Vec<(String, TypedValue)>),
    MemberAccess { name: String, item: Box<TypedValue> },

    NumberLiteral(isize),
    ListLiteral(Vec<TypedValue>),

    Variable(String),
    None,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct TypeId(usize);

impl TypeId {
    const NONE: TypeId = TypeId(0);
    const INT: TypeId = TypeId(1);
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum Type {
    List(TypeId),
}

impl Type {
    fn is_list(&self) -> bool {
        match self {
            Type::List(type_id) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Sig {
    args: Vec<TypeId>,
    type_params: Vec<TypeId>,
    ret_type: TypeId,
}

impl Sig {
    fn matches(&self, args: &mut impl Iterator<Item = TypeId>) -> bool {
        let mut expected_args = self.args.iter();

        loop {
            if let Some(arg) = args.next() {
                assert_eq!(arg, *expected_args.next().unwrap());
            } else {
                assert!(expected_args.next().is_none());
                return true;
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct SubState {
    var_to_id: HashMap<String, TypeId>,
}

#[derive(Debug, Default)]
pub struct State {
    states: Vec<SubState>,

    // var types
    name_to_id: HashMap<String, TypeId>,

    // functions
    function_to_sig: HashMap<String, Sig>,

    // structs
    struct_def: HashMap<TypeId, Vec<(String, TypeId)>>,

    // types
    types_to_id: HashMap<Type, TypeId>,
    id_to_type: HashMap<TypeId, Type>,

    curr_type: usize,
}

impl State {
    fn new() -> Self {
        State {
            states: vec![SubState {
                ..Default::default()
            }],
            curr_type: 2,

            ..Default::default()
        }
    }

    fn name_to_id(&self, name: &str) -> Option<TypeId> {
        match name {
            _ => self.name_to_id.get(name).copied(),
        }
    }

    fn var_to_id(&self, name: &str) -> Option<TypeId> {
        self.states
            .iter()
            .rev()
            .find_map(|x| x.var_to_id.get(name))
            .copied()
    }

    fn func_sig_to_ret(
        &self,
        name: &str,
        types: &mut impl Iterator<Item = TypeId>,
    ) -> Option<TypeId> {
        match name {
            "index" => {
                // assert_eq!(types.next().unwrap(), TypeId::LIST);
                assert!(
                    self.id_to_type
                        .get(&types.next().unwrap())
                        .unwrap()
                        .is_list()
                );
                assert_eq!(types.next().unwrap(), TypeId::INT);
                assert!(types.next().is_none());
                Some(TypeId::INT)
            }

            "add" => {
                assert_eq!(types.next().unwrap(), TypeId::INT);
                assert_eq!(types.next().unwrap(), TypeId::INT);
                assert!(types.next().is_none());
                Some(TypeId::INT)
            }

            "subtract" => {
                assert_eq!(types.next().unwrap(), TypeId::INT);
                assert_eq!(types.next().unwrap(), TypeId::INT);
                assert!(types.next().is_none());
                Some(TypeId::INT)
            }

            "print" => Some(TypeId::NONE),

            _ => {
                if let Some(sig) = self.function_to_sig.get(name) {
                    assert!(sig.matches(types));
                    Some(sig.ret_type)
                } else {
                    None
                }
            }
        }
    }

    fn id_to_struct(&self, id: TypeId) -> Option<&[(String, TypeId)]> {
        self.struct_def.get(&id).map(|y| y.as_slice())
    }

    fn push(&mut self) {
        self.states.push(SubState::default());
    }

    fn pop(&mut self) -> SubState {
        self.states.pop().unwrap()
    }

    fn add_struct_name(&mut self, str: String) -> TypeId {
        let id = TypeId(self.curr_type);
        self.curr_type += 1;

        assert!(self.name_to_id.insert(str, id).is_none());

        id
    }

    fn get_or_add_type(&mut self, ty: Type) -> TypeId {
        if let Some(type_id) = self.types_to_id.get(&ty) {
            *type_id
        } else {
            let id = TypeId(self.curr_type);
            self.curr_type += 1;

            assert!(self.types_to_id.insert(ty.clone(), id).is_none());
            assert!(self.id_to_type.insert(id, ty).is_none());

            id
        }
    }
}

fn parse(
    tokens: &mut Peekable<IntoIter<Token>>,
    state: &mut State,
    ret_type: TypeId,
) -> Vec<Statement> {
    let mut statements = Vec::new();

    loop {
        match tokens.peek() {
            Some(Token::Let) => {
                assert_eq!(Some(Token::Let), tokens.next());

                let Some(Token::Name(name)) = tokens.next() else {
                    unimplemented!()
                };

                assert_eq!(Some(Token::Equals), tokens.next());

                let value = parse_value(tokens, state);

                let substate = state.states.last_mut().unwrap();

                substate.var_to_id.insert(name.clone(), value.val_type);

                statements.push(Statement::VarDefinition(name, value));
            }

            Some(Token::Fun) => {
                assert_eq!(Some(Token::Fun), tokens.next());

                let mut string_params = Vec::new();
                let mut type_params = Vec::new();

                // get type_params
                if let Token::LAngle = tokens.peek().unwrap() {
                    assert_eq!(tokens.next().unwrap(), Token::LAngle);

                    while Token::RAngle != *tokens.peek().unwrap() {
                        let Token::Name(param) = tokens.next().unwrap() else {
                            unimplemented!()
                        };

                        type_params.push(state.add_struct_name(param.clone()));

                        string_params.push(param);

                        if Token::Comma == *tokens.peek().unwrap() {
                            assert_eq!(Token::Comma, tokens.next().unwrap());
                        }
                    }

                    assert_eq!(Token::RAngle, tokens.next().unwrap());
                }

                // get inputs and outputs

                let ret = parse_type(tokens, state);

                let Some(Token::Name(name)) = tokens.next() else {
                    unimplemented!()
                };

                assert_eq!(Some(Token::LParens), tokens.next());

                let mut args = Vec::new();

                state.push();

                while *tokens.peek().unwrap() != Token::RParens {
                    let arg_type = parse_type(tokens, state);

                    let Token::Name(arg) = tokens.next().unwrap() else {
                        unimplemented!()
                    };

                    args.push((arg.clone(), arg_type));

                    assert!(
                        state
                            .states
                            .last_mut()
                            .unwrap()
                            .var_to_id
                            .insert(arg, arg_type.clone())
                            .is_none()
                    );

                    if Token::Comma == *tokens.peek().unwrap() {
                        assert_eq!(Some(Token::Comma), tokens.next());
                    }
                }

                assert_eq!(Some(Token::RParens), tokens.next());

                // parse the func statements

                assert_eq!(Some(Token::LBrace), tokens.next());

                let func_statements = parse(tokens, state, ret);

                state.pop();

                assert_eq!(Some(Token::RBrace), tokens.next());

                let arg_args = Sig {
                    args: args.iter().map(|x| x.1).collect(),
                    type_params,
                };

                assert!(
                    state
                        .function_to_sig
                        .insert(name.clone(), (ret, arg_args))
                        .is_none()
                );

                for param in string_params {
                    assert!(state.name_to_id.remove(&param).is_some());
                }

                statements.push(Statement::FunDefinition(
                    name,
                    FunDefinition {
                        args,
                        statements: func_statements,
                        type_params,
                    },
                ));
            }

            Some(Token::Ret) => {
                assert_eq!(Some(Token::Ret), tokens.next());

                let value = parse_value(tokens, state);

                assert_eq!(value.val_type, ret_type);

                statements.push(Statement::Ret(value));
            }

            Some(Token::Struct) => {
                assert_eq!(Some(Token::Struct), tokens.next());

                let Token::Name(name) = tokens.next().unwrap() else {
                    unimplemented!()
                };

                assert_eq!(Some(Token::LBrace), tokens.next());

                let mut members = Vec::new();

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

                let id = state.add_struct_name(name.clone());

                assert!(state.struct_def.insert(id, members.clone()).is_none());

                statements.push(Statement::StructDefinition(name, members));

                assert_eq!(Some(Token::RBrace), tokens.next());
            }

            Some(Token::RBrace) | None => break,

            Some(_) => {
                let value = parse_value(tokens, state);

                assert_eq!(value.val_type, TypeId::NONE);

                statements.push(Statement::Value(value));
            }
        }

        assert_eq!(Some(Token::SemiColon), tokens.next(), "{tokens:?}");
    }

    statements
}

fn parse_value(tokens: &mut Peekable<IntoIter<Token>>, state: &mut State) -> TypedValue {
    let mut first_val = match tokens.next().unwrap() {
        Token::Name(name) => {
            if let Some(Token::LParens) = tokens.peek() {
                let args = parse_function_call(tokens, state);

                let ret = state
                    .func_sig_to_ret(&name, &mut args.iter().map(|x| x.val_type))
                    .unwrap();

                TypedValue {
                    val_source: ValueSource::Function { name, args },
                    val_type: ret,
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

                let args = parse_function_call(tokens, state);

                let ret = state
                    .func_sig_to_ret(&name, &mut args.iter().map(|x| x.val_type))
                    .unwrap();

                TypedValue {
                    val_source: ValueSource::Function { name, args },
                    val_type: ret,
                }
            } else if let Some(val_type) = state.var_to_id(&name) {
                TypedValue {
                    val_type,
                    val_source: ValueSource::Variable(name),
                }
            } else if let Some(type_con) = state.name_to_id(&name) {
                assert_eq!(Token::LBrace, tokens.next().unwrap());

                let mut struct_con = Vec::new();

                while *tokens.peek().unwrap() != Token::RBrace {
                    let Token::Name(member_name) = tokens.next().unwrap() else {
                        unimplemented!()
                    };

                    assert_eq!(Token::Colon, tokens.next().unwrap());

                    let value = parse_value(tokens, state);

                    struct_con.push((member_name, value));

                    if Token::Comma == *tokens.peek().unwrap() {
                        assert_eq!(Token::Comma, tokens.next().unwrap());
                    }
                }

                let mut struct_def = state.id_to_struct(type_con).unwrap().iter();
                let mut struct_con_iter = struct_con.iter();

                // TODO this could be better
                loop {
                    match (struct_def.next(), struct_con_iter.next()) {
                        (Some(x), Some(y)) => {
                            assert_eq!(x.0, y.0);
                            assert_eq!(x.1, y.1.val_type);
                        }
                        (None, None) => break,
                        _ => unimplemented!(),
                    }
                }

                assert_eq!(Token::RBrace, tokens.next().unwrap());

                TypedValue {
                    val_type: type_con,
                    val_source: ValueSource::StructConstruction(struct_con),
                }
            } else {
                unimplemented!("{tokens:?}")
            }
        }

        Token::Number(x) => TypedValue {
            val_type: TypeId::INT,
            val_source: ValueSource::NumberLiteral(x),
        },

        Token::LBracket => {
            let mut values = Vec::new();

            while Token::RBracket != *tokens.peek().unwrap() {
                let value = parse_value(tokens, state);

                // assert_eq!(value.val_type, TypeId::INT);

                values.push(value);

                if Token::RBracket != *tokens.peek().unwrap() {
                    assert_eq!(Token::Comma, tokens.next().unwrap());
                }
            }

            assert_eq!(Token::RBracket, tokens.next().unwrap());

            let list_type = values.first().unwrap().val_type;
            assert!(values.iter().all(|x| x.val_type == list_type));

            TypedValue {
                val_source: ValueSource::ListLiteral(values),
                val_type: state.get_or_add_type(Type::List(list_type)),
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

            let args = vec![first_val, parse_value(tokens, state)];

            assert_eq!(args[0].val_type, TypeId::INT);
            assert_eq!(args[1].val_type, TypeId::INT);

            TypedValue {
                val_source: ValueSource::Function { name, args },
                val_type: TypeId::INT,
            }
        } else if Token::LBracket == *tokens.peek().unwrap() {
            assert_eq!(Token::LBracket, tokens.next().unwrap());

            let Type::List(sub_type) = state.id_to_type.get(&first_val.val_type).unwrap() else {
                unimplemented!()
            };

            let sub_type = *sub_type;

            let value = parse_value(tokens, state);

            assert_eq!(Token::RBracket, tokens.next().unwrap());

            TypedValue {
                val_source: ValueSource::Function {
                    name: String::from("index"),
                    args: vec![first_val, value],
                },
                val_type: sub_type,
            }
        } else if Token::Dot == *tokens.peek().unwrap() {
            assert_eq!(Token::Dot, tokens.next().unwrap());

            let TypedValue {
                val_type,
                val_source: _,
            } = &first_val;

            let Token::Name(name) = tokens.next().unwrap() else {
                unimplemented!()
            };

            TypedValue {
                val_type: state
                    .id_to_struct(*val_type)
                    .unwrap()
                    .iter()
                    .find(|x| x.0 == name)
                    .unwrap()
                    .1,
                val_source: ValueSource::MemberAccess {
                    name,
                    item: Box::new(first_val),
                },
            }
        } else {
            return first_val;
        };

        first_val = updated;
    }
}

fn parse_type(tokens: &mut Peekable<IntoIter<Token>>, state: &mut State) -> TypeId {
    match tokens.next().unwrap() {
        Token::Name(name) if name == "none" => TypeId::NONE,
        Token::Name(name) if name == "int" => TypeId::INT,
        Token::Name(name) => state.name_to_id(&name).unwrap(),

        Token::LBracket => {
            let sub_type = parse_type(tokens, state);

            assert_eq!(tokens.next().unwrap(), Token::RBracket);

            state.get_or_add_type(Type::List(sub_type))
        }

        _ => todo!(),
    }
}

fn parse_function_call(
    tokens: &mut Peekable<IntoIter<Token>>,
    state: &mut State,
) -> Vec<TypedValue> {
    let mut args = Vec::new();

    assert_eq!(tokens.next(), Some(Token::LParens));

    while *tokens.peek().unwrap() != Token::RParens {
        args.push(parse_value(tokens, state));

        match tokens.peek().unwrap() {
            Token::Comma => {
                assert_eq!(tokens.next(), Some(Token::Comma));
            }

            Token::RParens => {}

            _ => unimplemented!(),
        }
    }

    assert_eq!(tokens.next(), Some(Token::RParens));

    args
}
