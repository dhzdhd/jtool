use std::cell::OnceCell;

use fancy_regex::{Captures, Regex};
use serde_json::{Map, Value};

use crate::error::Error;

const TRAILING_QUOTE_REGEX: OnceCell<Regex> = OnceCell::new();
const UNESCAPE_REGEX: OnceCell<Regex> = OnceCell::new();

pub fn parse(input: String) -> Result<Value, Error> {
    parse_value(Value::String(input.clone()))
}

fn parse_value(val: Value) -> Result<Value, Error> {
    let binding = TRAILING_QUOTE_REGEX;
    let trailing_regex = binding.get_or_init(|| Regex::new(r#"(^")|("$)"#).unwrap());
    let binding = UNESCAPE_REGEX;
    let unescape_regex = binding.get_or_init(|| Regex::new(r#"(?<!\\)((?:\\\\)*)\\""#).unwrap());

    match val {
        Value::String(val) => {
            let replaced = trailing_regex.replace_all(&val, "").to_string();
            let replaced = unescape_regex
                .replace_all(&replaced, |caps: &Captures| {
                    format!(
                        r#"{}{}"#,
                        caps.get(1).map_or("", |m| m.as_str()).replace("\\\\", "\\"),
                        r#"""#
                    )
                })
                .to_string();

            match serde_json::from_str::<Value>(&replaced) {
                Ok(parsed) => parse_value(parsed),
                Err(err) => {
                    // Error struct does not expose this specfic error
                    match err.to_string().contains("expected value") {
                        true => Ok(Value::String(replaced)),
                        _ => Err(Error::JSONParsing(err)),
                    }
                }
            }
        }
        Value::Array(arr) => arr.iter().map(|x| parse_value(x.clone())).collect(),
        Value::Object(obj) => {
            let iter_map: Vec<(String, Value)> = obj
                .iter()
                .map(|(k, v)| parse_value(v.clone()).map(|val| (k.clone(), val)))
                .collect::<Result<Vec<(String, Value)>, Error>>()?;
            Ok(Value::Object(Map::from_iter(iter_map)))
        }
        x => Ok(x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_json_str() {
        let sample = String::from(r#"{"name": "John", "age": 30}"#);
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30});

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_parse_json_str_with_list() {
        let sample = String::from(r#"{"name": "John", "age": 30, "l": ["hi", "hello", 2]}"#);
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30, "l": ["hi", "hello", 2]});

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_parse_nested_json() {
        let sample = String::from(
            r#"{"name": "John", "age": 30, "l": ["hi", "hello", 2], "nested": {"a": 2, "b": "hi", "c": ["hi"]}}"#,
        );
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30, "l": ["hi", "hello", 2], "nested": {"a": 2, "b": "hi", "c": ["hi"]}});

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_parse_json_with_trailing_quotes() {
        let sample = String::from(r#""{"name": "John", "age": 30}""#);
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30});

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_parse_stringified_json() {
        let sample = String::from("{\"name\": \"John\", \"age\": 30}");
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30});

        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_parse_nested_stringified_json() {
        let sample = String::from(
            r#"{\"name\": \"John\", \"age\": 30, \"l\": [\"hi\", \"hello\", 2], \"nested\": {\"a\": 2, \"b\": \"hi\", \"c\": [\"hi\"]}}"#,
        );
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30, "l": ["hi", "hello", 2], "nested": {"a": 2, "b": "hi", "c": ["hi"]}});

        // assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_parse_nested_doubly_stringified_json() {
        let sample = String::from(
            r#""{\"name\": \"John\", \"age\": 30, \"l\": [\"hi\", \"hello\", 2], \"nested\": \"{\\\"a\\\": 2, \\\"b\\\": \\\"hi\\\", \\\"c\\\": [\\\"hi\\\"]}\"}""#,
        );
        let actual = parse(sample);

        let expected = json!({"name": "John", "age": 30, "l": ["hi", "hello", 2], "nested": {"a": 2, "b": "hi", "c": ["hi"]}});

        // assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_incorrect_json() {
        let sample = String::from(
            r#"{\"name\": \"John\", \"age\": 30, \"l\": [\"hi\", \"hello", 2], \"nested\": \"{\\\"a\\": 2, \\\"b\\": \\\"hi\\\", \\\"c\\\": [\\\"hi\\\"]}\"}"#,
        );
        let actual = parse(sample);

        println!("{:?}", actual);
        assert_eq!(true, actual.is_err());
    }
}
