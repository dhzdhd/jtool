use serde_json::Value;

use crate::error::Error;
use crate::parse::parse;
use crate::stringify::stringify;

pub fn remove_spaces_value(json: Value) -> Result<Value, Error> {
    let json_str = stringify(json, None)?;

    parse(json_str)
}

pub fn remove_spaces_str(str: String) -> Result<String, Error> {
    let json = parse(str)?;

    stringify(json, None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_spaces_from_json() {}

    #[test]
    fn test_remove_spaces_from_string() {}
}
