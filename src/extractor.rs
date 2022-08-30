use crate::{request::StringMap, response::Response};
use anyhow::{anyhow, Result};

#[derive(Debug)]
enum ExtractorTypes {
    UNKNOWN,
    HEADER,
    JSON,
}

fn extract_type(extract: &String) -> (ExtractorTypes, &str) {
    if extract.starts_with("json:") {
        return (ExtractorTypes::JSON, &extract[5..]);
    } else if extract.starts_with("header:") {
        return (ExtractorTypes::HEADER, &extract[7..]);
    }
    return (ExtractorTypes::UNKNOWN, extract);
}

pub fn extract(to_extract: &StringMap, response: &Response) -> Result<StringMap> {
    let mut map = StringMap::new();

    for (prop, extract) in to_extract {
        let (extract_type, view) = extract_type(extract);

        println!("{:?} [{}]", extract_type, view);
    }

    Err(anyhow!("NYI"))
}

#[cfg(test)]
mod test {}
