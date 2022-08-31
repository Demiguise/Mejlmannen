use crate::{request::StringMap, response::Response};
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
    use anyhow::{anyhow, Result};

    pub fn extract(extract_string: &str, response: &Response) -> Result<String> {
        println!("JSON Parsing [{}]", extract_string);
        Err(anyhow!("NYI"))
    }
}

mod headers {
    use crate::response::Response;
    use anyhow::{anyhow, Result};

    pub fn extract(extract_string: &str, response: &Response) -> Result<String> {
        println!("Header Parsing [{}]", extract_string);
        Err(anyhow!("NYI"))
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
