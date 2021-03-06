use clap::{App, Arg};
use serde_json::Value;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::num;
use std::path::Path;
use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
enum YajqError {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),

    #[error("Json Error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("Filtering Error: {0}")]
    Filtering(String),

    #[error("Parsing Error: {0}")]
    Parsing(#[from] num::ParseIntError),
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
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .long("file")
                .takes_value(true),
        )
        .get_matches();
    let data = parse_data(matches.value_of("file"))?;
    let filtered = match matches.value_of("expression") {
        Some(expr) => {
            let tokens = parse_expression(expr);
            filter(&data, tokens)?
        }
        None => data,
    };
    println!("{}", serde_json::to_string_pretty(&filtered)?);
    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Token<'a> {
    Any,
    Key(&'a str),
}

fn parse_expression(expression: &str) -> Vec<Token> {
    expression
        .split('.')
        .into_iter()
        .map(|element| match element {
            "*" => Token::Any,
            _ => Token::Key(element),
        })
        .collect()
}

fn filter(data: &Value, tokens: Vec<Token>) -> Result<Value> {
    if tokens.is_empty() {
        Ok(data.to_owned())
    } else {
        return match tokens[0] {
            Token::Any => match data {
                Value::Array(array) => {
                    let result: Result<Vec<Value>> = array
                        .iter()
                        .map(|element| filter(element, tokens[1..].to_vec()))
                        .collect();
                    Ok(Value::Array(result?))
                }
                _ => Err(YajqError::Filtering(
                    "Can't use * on non array".to_string(),
                )),
            },
            Token::Key(key) => filter(
                match data {
                    Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                        Err(YajqError::Filtering(format!(
                            "Unit can't be filtered for key {}",
                            key
                        )))
                    }
                    Value::Object(object) => Ok(object.get(key).ok_or_else(|| {
                        YajqError::Filtering(format!("Key {} not in dict", key))
                    })?),
                    Value::Array(array) => Ok(&array[key.parse::<usize>()?]),
                }?,
                tokens[1..].to_vec(),
            ),
        };
    }
}

fn parse_data(path: Option<&str>) -> Result<Value> {
    match path {
        Some(path) => {
            let file = File::open(Path::new(path))?;
            let reader = BufReader::new(file);
            Ok(serde_json::from_reader(reader)?)
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(serde_json::from_str(&buffer)?)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_expression() {
        assert_eq!(
            parse_expression("a.12.*.c"),
            vec![
                Token::Key("a"),
                Token::Key("12"),
                Token::Any,
                Token::Key("c")
            ]
        );
    }

    fn filter_(data: &str, expression: &str) -> Value {
        filter(
            &serde_json::from_str(data).unwrap(),
            parse_expression(expression),
        )
        .unwrap()
        .to_owned()
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
    #[test]
    fn test_filter_multiple_stars() {
        assert_eq!(
            filter_(
                r#"{"x": [[{"name": "value1"}], [{"name": "value2"}]]}"#,
                "x.*.*.name"
            ),
            parse_data_(r#"[["value1"], ["value2"]]"#)
        )
    }
}
