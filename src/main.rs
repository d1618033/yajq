extern crate clap;

use clap::{App, Arg};
use serde_json;
use serde_json::Value;
use std::io;
use std::io::Read;
use std::num;
use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
enum YajqError {
    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),

    #[error("Json Error: {0}")]
    JsonParsingError(#[from] serde_json::Error),

    #[error("Filtering Error: {0}")]
    FilteringError(String),

    #[error("Parsing Error: {0}")]
    ParsingError(#[from] num::ParseIntError),
}

type Result<T> = result::Result<T, YajqError>;


fn main() {
    if let Err(e) = run() {
        println!("{}", e);
    }
}

fn run() -> Result<()> {
    let matches = App::new("YAJQ")
        .version("1.0")
        .author("David Sternlicht <d1618033@gmail.com>")
        .about("Yet Another Json Query Language")
        .arg(Arg::with_name("expression"))
        .get_matches();
    let data = parse_data()?;
    let filtered = match matches.value_of("expression") {
        Some(expr) => {
            let tokens = parse_expression(expr);
            filter(data, tokens)?
        }
        None => data,
    };
    println!("{}", serde_json::to_string_pretty(&filtered)?);
    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Expression<'a> {
    Any,
    Key(&'a str),
}

fn parse_expression(expression: &str) -> Vec<Expression> {
    expression
        .split(".")
        .into_iter()
        .map(|element| match element {
            "*" => Expression::Any,
            _ => Expression::Key(element),
        })
        .collect()
}

fn filter(data: Value, tokens: Vec<Expression>) -> Result<Value> {
    let mut current = data.clone();
    for (i, expr) in tokens.iter().enumerate() {
        match *expr {
            Expression::Any => {
                return match current {
                    Value::Array(array) => {
                        let result: Result<Vec<Value>> = array
                            .iter()
                            .map(|element| filter(element.clone(), tokens[i + 1..].to_vec()))
                            .collect();
                        Ok(Value::Array(result?))
                    }
                    _ => Err(YajqError::FilteringError(format!(
                        "Can't use * on non array"
                    ))),
                };
            }
            Expression::Key(expr) => {
                current = match current {
                    Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                        Err(YajqError::FilteringError(format!(
                            "Unit can't be filtered for key {}",
                            expr
                        )))
                    }
                    Value::Object(object) => Ok(object
                        .get(expr)
                        .ok_or(YajqError::FilteringError(format!(
                            "Key {} not in dict",
                            expr
                        )))?
                        .to_owned()),
                    Value::Array(array) => Ok(array[expr.parse::<usize>()?].to_owned()),
                }?
            }
        }
    }
    Ok(current)
}

fn parse_data() -> Result<Value> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_expression() {
        assert_eq!(
            parse_expression("a.12.*.c"),
            vec![
                Expression::Key("a"),
                Expression::Key("12"),
                Expression::Any,
                Expression::Key("c")
            ]
        );
    }

    fn filter_(data: &str, expression: &str) -> Value {
        filter(
            serde_json::from_str(data).unwrap(),
            parse_expression(expression),
        )
        .unwrap()
    }
    fn parse_data_(data: &str) -> Value {
        serde_json::from_str(data).unwrap()
    }
    #[test]
    fn test_filter_simple() {
        assert_eq!(filter_(r#"{"x": "value"}"#, "x"), parse_data_(r#""value""#))
    }
    #[test]
    fn test_filter_multiple_keys() {
        assert_eq!(
            filter_(r#"{"x": {"y": "value"}}"#, "x.y"),
            parse_data_(r#""value""#)
        )
    }
    #[test]
    fn test_filter_index() {
        assert_eq!(
            filter_(r#"{"x": ["value"]}"#, "x.0"),
            parse_data_(r#""value""#)
        )
    }
    #[test]
    fn test_filter_star() {
        assert_eq!(
            filter_(
                r#"{"x": [{"name": "value1"}, {"name": "value2"}]}"#,
                "x.*.name"
            ),
            parse_data_(r#"["value1", "value2"]"#)
        )
    }
}
