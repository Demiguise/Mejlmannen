use crate::common::StringMap;
use crate::response::Response;
use anyhow::{anyhow, Result};

#[derive(Debug)]
enum ExtractorTypes {
    UNKNOWN,
    HEADER,
    JSON,
}

fn get_type(extract: &String) -> (ExtractorTypes, &str) {
    if extract.starts_with("json:") {
        return (ExtractorTypes::JSON, &extract[5..]);
    } else if extract.starts_with("header:") {
        return (ExtractorTypes::HEADER, &extract[7..]);
    }
    return (ExtractorTypes::UNKNOWN, extract);
}

mod json {
    use crate::response::Response;
    use anyhow::{anyhow, Context, Result};
    use lazy_static::lazy_static;
    use regex::Regex;
    use serde_json::Value;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"([A-Za-z]+)\[(\d+)\]")
            .expect("Failed to create regex for JSON index parsing");
    }

    pub fn extract(extract_string: &str, response: &Response) -> Result<String> {
        println!("JSON Parsing [{}]", extract_string);
        let body = String::from_utf8(response.body().clone())?;

        let mut v: Value = serde_json::from_str(&body.as_str())?;

        for original_token in extract_string.split('.') {
            let mut index: Option<usize> = None;
            /*
                Would be nice to not convert this to an owned string, but I can't
                figure out a way of letting the compiler know that the string WILL
                outlive the "Borrow" from the RE.captures.
            */
            let token = match RE.captures(&original_token) {
                Some(captures) => {
                    // Parse the second capture, the number, into a usize for later
                    match &captures[2].parse::<usize>() {
                        Ok(val) => {
                            index = Some(*val);
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Failed to parse [{}] as a usize: {}",
                                &captures[1],
                                e
                            ))
                        }
                    };

                    // Make the first capture group, the identifier, into a String
                    captures[1].to_owned()
                }
                None => original_token.to_owned(),
            };
            match v.get(token) {
                Some(value) => {
                    // TODO: Anyway to _not_ clone this value?
                    v = value.clone();
                }
                None => {
                    return Err(anyhow!(
                        "Couldn't find [{}] in the response body. Full Query [{}]",
                        original_token,
                        extract_string
                    ))
                }
            };

            if index.is_some() {
                v = v
                    .as_array()
                    .with_context(|| format!("Failed to parse {} as an array", original_token))?
                    .get(index.unwrap())
                    .expect("Failed to get index")
                    .clone();
            }
        }

        if v.is_string() {
            Ok(v.as_str().unwrap().to_owned())
        } else {
            Ok(v.to_string())
        }
    }

    #[cfg(test)]
    mod test {
        use crate::response::ResponseBuilder;

        fn get_basic_string() -> &'static str {
            r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#
        }

        #[test]
        fn basic_string() {
            let response = ResponseBuilder::new()
                .body(get_basic_string().as_bytes().to_vec())
                .build();
            let to_extract = "name";
            let value = super::extract(to_extract, &response);
            assert!(value.is_ok(), "Extracting failed: {:?}", value.unwrap_err());
            assert_eq!(value.unwrap(), "John Doe");
        }

        #[test]
        fn basic_int() {
            let response = ResponseBuilder::new()
                .body(get_basic_string().as_bytes().to_vec())
                .build();
            let to_extract = "age";
            let value = super::extract(to_extract, &response);
            assert!(value.is_ok(), "Extracting failed: {:?}", value.unwrap_err());
            assert_eq!(value.unwrap(), "43");
        }

        #[test]
        fn basic_index() {
            let response = ResponseBuilder::new()
                .body(get_basic_string().as_bytes().to_vec())
                .build();
            let to_extract = "phones[1]";
            let value = super::extract(to_extract, &response);
            assert!(value.is_ok(), "Extracting failed: {:?}", value.unwrap_err());
            assert_eq!(value.unwrap(), "+44 2345678");
        }

        fn get_deep_object() -> &'static str {
            r#"
        {
            "foo": {
                "bar": {
                    "baz": {
                        "a": {
                            "b": {
                                "c": {
                                    "d": 1066
                                },
                                "x": 6
                            },
                            "y": 5
                        },
                        "z": 4
                    },
                    "zab": 3
                },
                "rab": 2
            },
            "oof": 1
        }"#
        }

        #[test]
        fn deep_object() {
            let response = ResponseBuilder::new()
                .body(get_deep_object().as_bytes().to_vec())
                .build();
            let to_extract = "foo.bar.baz.a.b.c.d";
            let value = super::extract(to_extract, &response);
            assert!(value.is_ok(), "Extracting failed: {:?}", value.unwrap_err());
            assert_eq!(value.unwrap(), "1066");
        }
    }
}

mod headers {
    use crate::response::Response;
    use anyhow::{anyhow, Result};

    pub fn extract(extract_string: &str, response: &Response) -> Result<String> {
        println!("Header Parsing [{}]", extract_string);

        for (header, value) in response.headers() {
            if header == extract_string {
                return Ok(value.to_string());
            }
        }

        Err(anyhow!("Could not find header in map"))
    }
}

pub fn extract(to_extract: &StringMap, response: &Response) -> Result<StringMap> {
    let mut map = StringMap::new();

    for (prop, extract) in to_extract {
        let (extract_type, view) = get_type(extract);
        let result = match extract_type {
            ExtractorTypes::JSON => json::extract(view, response),
            ExtractorTypes::HEADER => headers::extract(view, response),
            _ => Err(anyhow!("Unknown extractor type")),
        };

        match result {
            Ok(value) => {
                println!("Extracted [{}={}] from response body", prop, value);
                map.insert(prop.clone(), value);
            }
            Err(e) => {
                println!("Failed to extract {} from response: {}", view, e);
            }
        };
    }

    Ok(map)
}

#[cfg(test)]
mod test {}
