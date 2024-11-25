use clap::Parser;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::fs::read_to_string;
use std::io::{self, Write};
use thiserror::Error;

const VERSION: &str = "0.1.0";

#[derive(Parser, Debug)]
#[command(
    name = "Lazo",
    version = VERSION,
    author = "梶塚太智, kajizukataichi@outlook.jp",
    about = "Lisp like programming language that can give type annotation for gradual typing",
)]
struct Cli {
    /// Script file to be running
    #[arg(index = 1)]
    file: Option<String>,

    /// Run code quickly
    #[arg(short = 'l', long, name = "CODE")]
    one_liner: Option<String>,
}

fn main() {
    let mut scope: Scope = stdlib();
    let args = Cli::parse();

    if let Some(path) = args.file {
        if let Ok(code) = read_to_string(path) {
            if let Ok(lines) = tokenize(code) {
                for line in lines {
                    if let Ok(ast) = parse(line) {
                        ast.eval(&mut scope).unwrap();
                    }
                }
            }
        } else {
            eprintln!("Error! opening file is fault");
        }
    } else if let Some(code) = args.one_liner {
        if let Ok(lines) = tokenize(code) {
            for line in lines {
                if let Ok(ast) = parse(line) {
                    ast.eval(&mut scope).unwrap();
                }
            }
        }
    } else {
        println!("Lazo {VERSION}");
        if let Ok(mut rl) = DefaultEditor::new() {
            loop {
                match rl.readline("> ") {
                    Ok(code) => {
                        rl.add_history_entry(&code).unwrap_or_default();
                        match tokenize(code) {
                            Ok(lines) => {
                                for line in lines {
                                    match parse(line) {
                                        Ok(ast) => match ast.eval(&mut scope) {
                                            Ok(result) => println!("{result:?}"),
                                            Err(err) => println!("{err}"),
                                        },
                                        Err(err) => println!("{err}"),
                                    }
                                }
                            }
                            Err(err) => println!("{err}"),
                        }
                    }
                    Err(err) => println!("{err}"),
                }
            }
        }
    }
}

fn stdlib() -> Scope {
    HashMap::from([
        (
            "+".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                let params = {
                    let mut new = vec![];
                    for i in params {
                        new.push(i.eval(scope)?)
                    }
                    new
                };

                if params.len() >= 1 {
                    let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                    let mut result: f64 = params[0];
                    for i in params[1..params.len()].to_vec().iter() {
                        result = result + i.clone();
                    }
                    Ok(Type::Number(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "-".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() >= 1 {
                    let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                    if params.len() >= 2 {
                        let mut result: f64 = params[0];
                        for i in params[1..params.len()].to_vec().iter() {
                            result -= i.clone();
                        }
                        Ok(Type::Number(result))
                    } else {
                        Ok(Type::Number(-params[0]))
                    }
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "*".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                let params = {
                    let mut new = vec![];
                    for i in params {
                        new.push(i.eval(scope)?)
                    }
                    new
                };

                if params.len() >= 1 {
                    let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                    let mut result: f64 = params[0];
                    for i in params[1..params.len()].to_vec().iter() {
                        result *= i.clone();
                    }
                    Ok(Type::Number(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "/".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                let params = {
                    let mut new = vec![];
                    for i in params {
                        new.push(i.eval(scope)?)
                    }
                    new
                };

                if params.len() >= 1 {
                    let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                    let mut result: f64 = params[0];
                    for i in params[1..params.len()].to_vec().iter() {
                        result /= i.clone();
                    }
                    Ok(Type::Number(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "%".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 1 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                    let mut result: f64 = params[0];
                    for i in params[1..params.len()].to_vec().iter() {
                        result %= i;
                    }
                    Ok(Type::Number(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "^".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                let params = {
                    let mut new = vec![];
                    for i in params {
                        new.push(i.eval(scope)?)
                    }
                    new
                };

                if params.len() >= 1 {
                    let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                    let mut result: f64 = params[0];
                    for i in params[1..params.len()].to_vec().iter() {
                        result = result.powf(i.to_owned());
                    }
                    Ok(Type::Number(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "concat".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                Ok(Type::String({
                    let mut result = "".to_string();
                    for i in params {
                        result += &i.eval(scope)?.get_string();
                    }
                    result
                }))
            })),
        ),
        (
            "print".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                for i in params {
                    print!("{}", i.eval(scope)?.get_string())
                }
                Ok(Type::Null)
            })),
        ),
        (
            "format".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    Ok(Type::String(
                        params[0]
                            .get_string()
                            .replace("{}", &params[1].eval(scope)?.get_string()),
                    ))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "debug".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                for i in params {
                    println!("Debug: {i:?} = {:?}", i.load(scope))
                }
                Ok(Type::Null)
            })),
        ),
        (
            "input".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() <= 1 {
                    Ok(Type::String({
                        let mut input = String::new();
                        if let Some(prompt) = params.get(0) {
                            print!("{}", prompt.get_string());
                        }
                        io::stdout().flush().unwrap_or_default();
                        match io::stdin().read_line(&mut input) {
                            Ok(_) => input.trim().to_string(),
                            Err(_) => {
                                return Err(LazoError::Runtime(
                                    "reading line was fault".to_string(),
                                ))
                            }
                        }
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "=".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 2 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    Ok(Type::Bool({
                        let params: Vec<String> = params.iter().map(|i| format!("{i:?}")).collect();
                        params.windows(2).all(|window| window[0] == window[1])
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "!=".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 2 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    Ok(Type::Bool({
                        let params: Vec<String> = params.iter().map(|i| format!("{i:?}")).collect();
                        params.windows(2).all(|window| window[0] != window[1])
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            ">".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 2 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    Ok(Type::Bool({
                        let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                        params.windows(2).all(|window| window[0] > window[1])
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            ">=".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 2 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    Ok(Type::Bool({
                        let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                        params.windows(2).all(|window| window[0] >= window[1])
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "<".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 2 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    Ok(Type::Bool({
                        let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                        params.windows(2).all(|window| window[0] < window[1])
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "<=".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() >= 2 {
                    let params = {
                        let mut new = vec![];
                        for i in params {
                            new.push(i.eval(scope)?)
                        }
                        new
                    };

                    Ok(Type::Bool({
                        let params: Vec<f64> = params.iter().map(|i| i.get_number()).collect();
                        params.windows(2).all(|window| window[0] < window[1])
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "&".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() >= 2 {
                    Ok(Type::Bool({
                        let params: Vec<bool> = params.iter().map(|i| i.get_bool()).collect();
                        params.iter().all(|x| *x)
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "|".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() >= 2 {
                    Ok(Type::Bool({
                        let params: Vec<bool> = params.iter().map(|i| i.get_bool()).collect();
                        params.iter().any(|x| *x)
                    }))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "!".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() == 1 {
                    Ok(Type::Bool(!params[0].get_bool()))
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "cast".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() == 2 {
                    match params[1].get_string().as_str() {
                        "number" => Ok(Type::Number(params[0].get_number())),
                        "string" => Ok(Type::String(params[0].get_string())),
                        "bool" => Ok(Type::Bool(params[0].get_bool())),
                        "list" => Ok(Type::List(params[0].get_list())),
                        other => Err(LazoError::Runtime(format!("unknown type name `{other}`"))),
                    }
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "type".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() == 1 {
                    Ok(Type::String(params[0].get_type()))
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "eval".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                let mut result = Type::Null;
                for expr in params {
                    result = expr.eval(scope)?;
                }
                Ok(result)
            })),
        ),
        (
            "define".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                let value: Type;
                if params.len() >= 2 {
                    if let Type::List(args) | Type::Expr(args) = params[0].clone() {
                        value = Type::Function(Function::UserDefined(
                            args[1..].to_vec(),
                            params[1..].to_owned(),
                        ));
                        scope.insert(args[0].get_string(), value.clone());
                    } else {
                        value = params[1].to_owned();
                        scope.insert(params[0].get_string(), value.clone());
                    }
                } else {
                    return Err(LazoError::Function(params.len(), 2));
                }
                Ok(value)
            })),
        ),
        (
            "lambda".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() >= 2 {
                    Ok(Type::Function(Function::UserDefined(
                        params[0].get_list(),
                        params[1..].to_vec(),
                    )))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "if".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 3 {
                    if params[0].eval(scope)?.get_bool() {
                        params[1].eval(scope)
                    } else {
                        params[2].eval(scope)
                    }
                } else if params.len() == 2 {
                    if params[0].eval(scope)?.get_bool() {
                        params[1].eval(scope)
                    } else {
                        Ok(Type::Null)
                    }
                } else {
                    Err(LazoError::Function(params.len(), 3))
                }
            })),
        ),
        (
            "cond".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                for i in params {
                    if i.get_list()[0].eval(scope)?.get_bool() {
                        return Ok(i.get_list()[1].eval(scope)?);
                    }
                }
                Ok(Type::Null)
            })),
        ),
        (
            "car".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 1 {
                    Ok(params[0]
                        .eval(scope)?
                        .get_list()
                        .get(0)
                        .unwrap_or(&Type::Null)
                        .clone())
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "cdr".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 1 {
                    let list = params[0].eval(scope)?.get_list();
                    Ok(Type::List(list[1..list.len()].to_vec()))
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "range".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 1 {
                    let mut range: Vec<Type> = vec![];
                    let mut current: f64 = 0.0;
                    while current < params[0].eval(scope)?.get_number() {
                        range.push(Type::Number(current));
                        current += 1.0;
                    }
                    Ok(Type::List(range))
                } else if params.len() == 2 {
                    let mut range: Vec<Type> = vec![];
                    let mut current: f64 = params[0].eval(scope)?.get_number();
                    while current < params[1].eval(scope)?.get_number() {
                        range.push(Type::Number(current));
                        current += 1.0;
                    }
                    Ok(Type::List(range))
                } else if params.len() == 3 {
                    let mut range: Vec<Type> = vec![];
                    let mut current: f64 = params[0].eval(scope)?.get_number();
                    while current < params[1].eval(scope)?.get_number() {
                        range.push(Type::Number(current));
                        current += params[2].eval(scope)?.get_number();
                    }
                    Ok(Type::List(range))
                } else {
                    Err(LazoError::Function(params.len(), 3))
                }
            })),
        ),
        (
            "map".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    let mut result = vec![];
                    let func = params[1].eval(scope)?.clone();
                    for i in params[0].eval(scope)?.get_list() {
                        result.push(Type::Expr(vec![func.clone(), i]).eval(scope)?);
                    }
                    Ok(Type::List(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "for".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    let func = params[1].eval(scope)?.clone();
                    for i in params[0].eval(scope)?.get_list() {
                        Type::Expr(vec![func.clone(), i]).eval(scope)?;
                    }
                    Ok(Type::Null)
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "filter".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    let mut result = vec![];
                    let func = params[1].eval(scope)?.clone();
                    for i in params[0].eval(scope)?.get_list() {
                        if Type::Expr(vec![func.to_owned(), i.clone()])
                            .eval(scope)?
                            .get_bool()
                        {
                            result.push(i)
                        }
                    }
                    Ok(Type::List(result))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "reduce".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    let func = params[1].clone();
                    let list = params[0].eval(scope)?.get_list();
                    let mut result = if let Some(first) = list.get(0) {
                        first.clone()
                    } else {
                        return Err(LazoError::Runtime("passed list is empty".to_string()));
                    };

                    let mut scope = scope.clone();
                    for i in list[1..].to_vec() {
                        result =
                            Type::Expr(vec![func.clone(), result, i.clone()]).eval(&mut scope)?
                    }
                    Ok(result.to_owned())
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "reverse".to_string(),
            Type::Function(Function::BuiltIn(|params, _| {
                if params.len() == 1 {
                    let mut list = params[0].get_list();
                    list.reverse();
                    Ok(Type::List(list))
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "len".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 1 {
                    Ok(Type::Number(params[0].eval(scope)?.get_list().len() as f64))
                } else {
                    Err(LazoError::Function(params.len(), 1))
                }
            })),
        ),
        (
            "repeat".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    Ok(Type::String(
                        params[0]
                            .eval(scope)?
                            .get_string()
                            .repeat(params[1].eval(scope)?.get_number() as usize),
                    ))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "join".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    Ok(Type::String(
                        params[0]
                            .eval(scope)?
                            .get_list()
                            .iter()
                            .map(|i| i.get_string())
                            .collect::<Vec<String>>()
                            .join(&params[1].eval(scope)?.get_string()),
                    ))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "split".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    Ok(Type::List(
                        params[0]
                            .eval(scope)?
                            .get_string()
                            .split(&params[1].eval(scope)?.get_string())
                            .map(|i| Type::String(i.to_string()))
                            .collect::<Vec<Type>>(),
                    ))
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "error".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                Err(LazoError::Runtime(
                    params
                        .get(0)
                        .unwrap_or(&Type::String("Something went wrong".to_string()))
                        .eval(scope)?
                        .get_string(),
                ))
            })),
        ),
        (
            "try".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                if params.len() == 2 {
                    if let Ok(result) = params[0].eval(scope) {
                        Ok(result)
                    } else {
                        params[1].eval(scope)
                    }
                } else {
                    Err(LazoError::Function(params.len(), 2))
                }
            })),
        ),
        (
            "exit".to_string(),
            Type::Function(Function::BuiltIn(|params, scope| {
                std::process::exit(
                    params
                        .get(0)
                        .unwrap_or(&Type::Number(0.0))
                        .eval(scope)?
                        .get_number() as i32,
                )
            })),
        ),
        ("new-line".to_string(), Type::String("\n".to_string())),
        ("double-quote".to_string(), Type::String("\"".to_string())),
        ("tab".to_string(), Type::String("\t".to_string())),
    ])
}

#[derive(Debug, Error)]
enum LazoError {
    #[error("Runtime Error! {0}")]
    Runtime(String),

    #[error("Syntax Error! {0}")]
    Syntax(String),

    #[error("Function Error! the passed arguments length {0} is different to expected length {1} of the function's arguments")]
    Function(usize, usize),
}
type Scope = HashMap<String, Type>;

#[derive(Clone)]
enum Type {
    Function(Function),
    Expr(Vec<Type>),
    List(Vec<Type>),
    Symbol(String),
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug)]
enum Function {
    BuiltIn(fn(Vec<Type>, &mut Scope) -> Result<Type, LazoError>),
    UserDefined(Vec<Type>, Vec<Type>),
}

impl Type {
    fn get_number(&self) -> f64 {
        match &self {
            Type::Number(n) => n.to_owned(),
            Type::String(s) | Type::Symbol(s) => s.trim().parse().unwrap_or(0.0),
            Type::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Type::Expr(x) | Type::List(x) => x.get(0).unwrap_or(&Type::Null).get_number(),
            Type::Function(_) | Type::Null => 0.0,
        }
    }

    fn get_string(&self) -> String {
        match &self {
            Type::Number(n) => n.to_string(),
            Type::String(s) => s.to_owned(),
            Type::Bool(b) => b.to_string(),
            Type::Symbol(v) => v.to_owned(),
            other => format!("{other:?}"),
        }
    }

    fn get_bool(&self) -> bool {
        match &self {
            Type::Number(n) => *n != 0.0,
            Type::String(s) | Type::Symbol(s) => !s.is_empty(),
            Type::Expr(s) | Type::List(s) => !s.is_empty(),
            Type::Bool(b) => *b,
            Type::Function(_) | Type::Null => false,
        }
    }

    fn get_type(&self) -> String {
        match &self {
            Type::Number(_) => "number".to_string(),
            Type::String(_) => "string".to_string(),
            Type::Bool(_) => "bool".to_string(),
            Type::Expr(_) => "expr".to_string(),
            Type::Symbol(_) => "symbol".to_string(),
            Type::List(_) => "list".to_string(),
            Type::Null => "null".to_string(),
            Type::Function(_) => "function".to_string(),
        }
    }

    fn get_list(&self) -> Vec<Type> {
        match &self {
            Type::Expr(e) => e.to_owned(),
            Type::List(l) => l.to_owned(),
            other => vec![other.to_owned().to_owned()],
        }
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt = match &self {
            Type::String(s) => format!("\"{s}\""),
            Type::Number(n) => n.to_string(),
            Type::Bool(b) => b.to_string(),
            Type::Function(Function::UserDefined(args, code)) => {
                format!(
                    "(lambda ({}) {})",
                    args.iter()
                        .map(|i| format!("{i:?}"))
                        .collect::<Vec<String>>()
                        .join(" "),
                    code.iter()
                        .map(|i| format!("{i:?}"))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Type::Function(Function::BuiltIn(n)) => format!("function({n:?})"),
            Type::Symbol(v) => v.to_owned(),
            Type::List(l) => format!(
                "[{}]",
                l.iter()
                    .map(|x| format!("{x:?}"))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Type::Expr(l) => format!(
                "({})",
                l.iter()
                    .map(|x| format!("{x:?}"))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Type::Null => "null".to_string(),
        };
        write!(f, "{fmt}")
    }
}

impl Type {
    fn eval(&self, scope: &mut Scope) -> Result<Type, LazoError> {
        if let Type::Expr(expr) = self.load(scope) {
            let func = if let Some(func) = expr.first() {
                func.eval(scope)?
            } else {
                return Err(LazoError::Syntax(
                    "empty expression can't be evaluated".to_string(),
                ));
            };

            if let Type::Function(Function::BuiltIn(func)) = func {
                func(expr[1..].to_vec(), scope)
            } else if let Type::Function(Function::UserDefined(args, code)) = func {
                // Check arguments length
                if args.len() != expr[1..].len() {
                    return Err(LazoError::Function(expr[1..].len(), args.len()));
                }

                // Setting arguemnt and its value
                let mut func_scope = scope.clone();
                for (k, v) in args.iter().zip(expr[1..].to_vec()) {
                    // Setting argument by passed value
                    func_scope.insert(k.get_string(), v.load(scope));
                }

                // Execution of function's code
                let mut result = Type::Null;
                for line in code {
                    result = line.eval(&mut func_scope)?
                }
                Ok(result)
            } else {
                return Err(LazoError::Syntax(format!(
                    "first atom in expression should be function, but provided `{:?}` is not function",
                    expr.get(0).cloned().unwrap_or(Type::Null)
                )));
            }
        } else {
            let expr = self.clone();
            Ok(if let Type::Symbol(name) = expr.clone() {
                // Loading variable from scope
                if let Some(value) = scope.get(&name).to_owned() {
                    value.clone()
                } else {
                    expr
                }
            } else {
                expr
            })
        }
    }

    fn load(&self, scope: &mut Scope) -> Type {
        if let Type::Symbol(sym) = self {
            if let Some(val) = scope.get(sym) {
                val.clone()
            } else {
                self.clone()
            }
        } else {
            self.clone()
        }
    }
}

fn parse(token: String) -> Result<Type, LazoError> {
    let mut token = token.trim().to_string();
    Ok(
        // Number case
        if let Ok(n) = token.parse::<f64>() {
            Type::Number(n)
        // Bool calse
        } else if let Ok(b) = token.parse::<bool>() {
            Type::Bool(b)
        // Null calse
        } else if token == "null".to_string() {
            Type::Null
        // String calse
        } else if token.starts_with('"') && token.ends_with('"') {
            token.remove(0); // Removing outer syntax
            token.remove(token.len() - 1);
            Type::String(token)
        // Expression case
        } else if token.starts_with('(') && token.ends_with(')') {
            token.remove(0); // Removing outer syntax
            token.remove(token.len() - 1);
            let mut list = vec![];
            for i in tokenize(token)? {
                list.push(parse(i)?)
            }
            Type::Expr(list)
        // List case
        } else if token.starts_with("[") && token.ends_with(']') {
            token.remove(0); // Removing outer syntax
            token.remove(token.len() - 1);
            let mut list = vec![];
            for i in tokenize(token)? {
                list.push(parse(i)?)
            }
            Type::List(list)
        // Symbol that explicitly
        } else if token.starts_with("'") {
            token.remove(0); // Removing single quote
            Type::Symbol(token)
        // Other case will be symbol
        } else {
            Type::Symbol(token.clone())
        },
    )
}

fn tokenize(input: String) -> Result<Vec<String>, LazoError> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current_token = String::new();
    let mut in_parentheses: usize = 0;
    let mut in_quote = false;

    for c in input.chars() {
        match c {
            '(' | '[' if !in_quote => {
                current_token.push(c);
                in_parentheses += 1;
            }
            ')' | ']' if !in_quote => {
                current_token.push(c);
                if in_parentheses > 0 {
                    in_parentheses -= 1;
                } else {
                    return Err(LazoError::Syntax(
                        "there's duplicate end of the parentheses".to_string(),
                    ));
                }
            }
            ' ' | '　' | '\n' | '\t' | '\r' if !in_quote => {
                if in_parentheses != 0 {
                    current_token.push(c);
                } else if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            '"' => {
                in_quote = !in_quote;
                current_token.push(c);
            }
            _ => {
                current_token.push(c);
            }
        }
    }

    // Syntax error check
    if in_quote {
        return Err(LazoError::Syntax(
            "there's not end of the quote".to_string(),
        ));
    }
    if in_parentheses != 0 {
        return Err(LazoError::Syntax(
            "there's not end of the parentheses".to_string(),
        ));
    }

    if in_parentheses == 0 && !current_token.is_empty() {
        tokens.push(current_token.clone());
        current_token.clear();
    }
    Ok(tokens)
}
