use anyhow::{anyhow, Result};
use crate::response::Response;
use crate::common::StringMap;

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
    use anyhow::{anyhow, Result};
    use serde_json::Value;

    pub fn extract(
        extract_string: &str,
        response: &Response,
    ) -> Result<String> {
        println!("JSON Parsing [{}]", extract_string);
        let body = String::from_utf8(response.body().clone())?;
        let v: Value = serde_json::from_str(&body.as_str())?;

        match v.get(extract_string) {
            Some(value) => Ok(value.to_string()),
            None => Err(anyhow!(
                "Couldn't find [{}] in response body",
                extract_string
            )),
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
