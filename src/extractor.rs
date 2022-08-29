use crate::{request::StringMap, response::Response};
use anyhow::{anyhow, Result};

pub fn extract(to_extract: &StringMap, response: &Response) -> Result<StringMap> {
    let mut map = StringMap::new();

    Err(anyhow!("NYI"))
}

#[cfg(test)]
mod test {}
