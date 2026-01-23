use serde_json::Value;

use crate::error::Error;

fn sort_by_period_count(paths: Vec<&str>) -> Vec<&str> {
    let mut buf = paths;
    buf.sort_by(|a, b| b.matches(".").count().cmp(&a.matches(".").count()));

    buf
}

fn edit_val(val: &mut Value, sequence: Vec<&str>) -> Result<(), Error> {
    match sequence.len() {
        0 => Ok(()),
        1 => {
            if let Value::Object(obj) = val {
                let value = obj.get(sequence[0]).unwrap();
                obj.insert(
                    sequence[0].to_string(),
                    Value::String(serde_json::to_string(value).unwrap()),
                );

                Ok(())
            } else {
                Err(Error::JSONStringify)
            }
        }
        _ => edit_val(
            val.get_mut(sequence[0]).ok_or(Error::JSONStringify)?,
            sequence,
        ),
    }
}

pub fn stringify(val: Value, paths: Option<Vec<&str>>) -> Result<String, Error> {
    let sorted_paths = sort_by_period_count(paths.unwrap_or(Vec::new()));

    let mut buf = val;
    for path in sorted_paths {
        let sequence: Vec<&str> = path.split('.').collect();
        edit_val(&mut buf, sequence)?;
    }

    Ok(serde_json::to_string(&buf).map_err(|_| Error::JSONStringify)?)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_stringify_simple_json() {
        let json = json!(
            {"a": "b"}
        );
        let expected = "{\"a\":\"b\"}";

        let actual = stringify(json, None);
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_stringify_simple_json_with_path() {
        let json = json!(
            {"a": "b", "c": {"d": "e"}}
        );
        let path = vec!["c"];
        let expected = r#"{"a":"b","c":"{\"d\":\"e\"}"}"#;

        let actual = stringify(json, Some(path));
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_sort_paths() {
        let expected = vec!["a.b.c.d", "b.c.d", "a.c", "a"];
        let data = vec!["a.c", "a.b.c.d", "a", "b.c.d"];
        let actual = sort_by_period_count(data.iter().map(|x| *x).collect());

        assert_eq!(expected, actual)
    }
}
