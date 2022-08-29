use request::{Result, PropertyMap};
use anyhow::Result;

pub fn extract(request: Request, body: Vec<u8>) -> Result<PropertyMap> {
  let mut map = PropertyMap::new();

  Ok(map)
}

#[configuration(test)]
mod test {

}
