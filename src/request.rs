use crate::common::StringMap;
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::__private::de::Content;
use serde::{Deserialize, Deserializer};
use std::fs;
use std::path::PathBuf;

lazy_static! {
    static ref RE: Regex = Regex::new(r"\{{1,2}(\w*)\}{1,2}")
        .expect("Failed to create regex for Request data replacement");
}

// TODO: Expand
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Verb {
    GET,
    POST,
    DELETE,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ContentType {
    String,
    Binary,
}
impl ContentType {
    pub fn default() -> Self {
        ContentType::String
    }
}

#[derive(Debug, Deserialize)]
pub struct Request {
    // Require properties
    uri: String,
    verb: Verb,

    // Defaults make these "Optional"
    #[serde(default)]
    properties: StringMap,
    #[serde(default)]
    headers: StringMap,
    #[serde(default)]
    body: String,
    #[serde(default)]
    extract: StringMap,
    #[serde(default = "ContentType::default")]
    content_type: ContentType,
}

impl Request {
    pub fn uri(&self) -> &String {
        &self.uri
    }
    pub fn headers(&self) -> &StringMap {
        &self.headers
    }
    pub fn body(&self) -> Vec<u8> {
        self.body.as_bytes().to_vec()
    }
    pub fn verb(&self) -> Verb {
        self.verb
    }
    pub fn extract(&self) -> &StringMap {
        &self.extract
    }
    pub fn content_type(&self) -> ContentType {
        self.content_type
    }

    /*
        Lifetime markers here as we need to say that cached_properties lives just as long,
        or longer, than self.
    */
    fn get_property<'a>(&'a self, name: &str, cached_properties: &'a StringMap) -> Option<&String> {
        match self.properties.get(name) {
            Some(value) => Some(value),
            None => match cached_properties.get(name) {
                Some(value) => Some(value),
                None => None,
            },
        }
    }

    fn replace_text(&self, text: &String, cached_properties: &StringMap) -> String {
        RE.replace_all(text, |caps: &regex::Captures| {
            if caps[0].starts_with("{{") && caps[0].ends_with("}}") {
                // Escaped {} string, just return the inner string
                caps[0][1..caps[0].len() - 1].to_owned()
            } else {
                // Normal replacement of a variable
                match self.get_property(&caps[1], cached_properties) {
                    Some(value) => value.clone(),
                    None => caps[0].to_owned(), // Just return the matched string instead
                }
            }
        })
        .into_owned()
    }

    // TODO: Replace these with in pre-prepared versions?
    pub fn replaced_uri(&self, cached_properties: &StringMap) -> String {
        self.replace_text(&self.uri, cached_properties)
    }

    pub fn replaced_headers(&self, cached_properties: &StringMap) -> StringMap {
        let mut map = StringMap::new();
        self.headers.iter().for_each(|(key, value)| {
            map.insert(key.clone(), self.replace_text(&value, cached_properties));
        });
        map
    }

    pub fn replaced_body(&self, cached_properties: &StringMap) -> Vec<u8> {
        match self.content_type {
            ContentType::String => {
                let replaced = self.replace_text(&self.body, cached_properties);
                replaced.into_bytes()
            }
            ContentType::Binary => self.body.clone().into_bytes(),
        }
    }

    // I don't like this but I'm not sure there's much other way
    pub fn update_body(&mut self, working_directory: &PathBuf) -> Result<()> {
        if self.body.starts_with("file:") {
            // Load the file that we need and replace the body with it
            let file_path = self.body.strip_prefix("file:").unwrap();
            let file_path = working_directory.join(file_path);

            let data = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to load {}", file_path.display()))?;

            self.body = data;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::PathBuf};

    use super::{ContentType, Request, StringMap, Verb};

    struct RequestBuilder {
        uri: String,
        properties: StringMap,
        headers: StringMap,
        body: String,
        verb: Verb,
        extract: StringMap,
    }

    impl RequestBuilder {
        pub fn new() -> Self {
            RequestBuilder {
                uri: String::new(),
                properties: StringMap::new(),
                headers: StringMap::new(),
                body: String::new(),
                verb: Verb::GET,
                extract: StringMap::new(),
            }
        }

        pub fn uri(mut self, uri: String) -> RequestBuilder {
            self.uri = uri;
            self
        }

        pub fn properties(mut self, test: StringMap) -> RequestBuilder {
            self.properties = test;
            self
        }

        pub fn header(mut self, header: String, value: String) -> RequestBuilder {
            self.headers.insert(header, value);
            self
        }

        pub fn header_map(mut self, headers: StringMap) -> RequestBuilder {
            self.headers.extend(headers);
            self
        }

        pub fn body(mut self, body: String) -> RequestBuilder {
            self.body = body;
            self
        }

        pub fn verb(mut self, verb: Verb) -> RequestBuilder {
            self.verb = verb;
            self
        }

        pub fn extract_map(mut self, extract: StringMap) -> RequestBuilder {
            self.extract = extract;
            self
        }

        pub fn build(self) -> Request {
            Request {
                uri: self.uri,
                properties: self.properties,
                headers: self.headers,
                body: self.body,
                verb: self.verb,
                extract: self.extract,
                content_type: ContentType::String
            }
        }
    }

    #[test]
    fn uri_replaced() {
        let mut props = StringMap::new();
        props.insert("some_key".to_owned(), "some_value".to_owned());

        let request = RequestBuilder::new()
            .uri("URI/{some_key}/URI".to_owned())
            .properties(props)
            .build();

        assert_eq!(
            request.replaced_uri(&StringMap::new()),
            "URI/some_value/URI"
        );
    }

    #[test]
    fn uri_replaced_escaped_braces() {
        // It should escape the key even if it's available in the property maps
        let mut props = StringMap::new();
        props.insert("some_key".to_owned(), "some_value".to_owned());

        let request = RequestBuilder::new()
            .uri("URI/{{some_key}}/URI".to_owned())
            .properties(props)
            .build();

        assert_eq!(
            request.replaced_uri(&StringMap::new()),
            "URI/{some_key}/URI"
        );
    }

    #[test]
    fn uri_replaced_empty_escaped_braces() {
        let request = RequestBuilder::new().uri("URI/{{}}/URI".to_owned()).build();

        assert_eq!(request.replaced_uri(&StringMap::new()), "URI/{}/URI");
    }

    #[test]
    fn uri_replaced_unknown_key() {
        let request = RequestBuilder::new()
            .uri("URI/{some_key}/URI".to_owned())
            .build();

        assert_eq!(
            request.replaced_uri(&StringMap::new()),
            "URI/{some_key}/URI"
        );
    }

    #[test]
    fn uri_replaced_empty_key() {
        let request = RequestBuilder::new().uri("URI/{}/URI".to_owned()).build();

        assert_eq!(request.replaced_uri(&StringMap::new()), "URI/{}/URI");
    }

    #[test]
    fn headers_replaced() {
        let mut props = StringMap::new();
        props.insert("some_key".to_owned(), "some_value".to_owned());

        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{some_key}".to_owned())
            .properties(props)
            .build();

        assert_eq!(
            request.replaced_headers(&StringMap::new()).get("A_HEADER"),
            Some(&"some_value".to_owned())
        );
    }

    #[test]
    fn headers_replaced_unknown_key() {
        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{some_key}".to_owned())
            .build();

        assert_eq!(
            request.replaced_headers(&StringMap::new()).get("A_HEADER"),
            Some(&"{some_key}".to_owned())
        );
    }

    #[test]
    fn headers_replaced_empty_key() {
        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{}".to_owned())
            .build();

        assert_eq!(
            request.replaced_headers(&StringMap::new()).get("A_HEADER"),
            Some(&"{}".to_owned())
        );
    }

    #[test]
    fn headers_replaced_escaped_key() {
        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{{some_key}}".to_owned())
            .build();

        assert_eq!(
            request.replaced_headers(&StringMap::new()).get("A_HEADER"),
            Some(&"{some_key}".to_owned())
        );
    }

    #[test]
    fn basic_serialisation() {
        let data = r#"{
            "uri": "http://some.website.com",
            "verb": "GET"
        }"#;

        let value = serde_json::from_str::<Request>(&data);
        assert!(
            value.is_ok(),
            "Failed to parse basic string: {}",
            value.unwrap_err()
        );
    }

    #[test]
    fn body_serialisation() {
        let data = r#"{
            "uri": "http://some.website.com",
            "verb": "GET",
            "body": "hello"
        }"#;

        let value = serde_json::from_str::<Request>(&data);
        assert!(
            value.is_ok(),
            "Failed to parse body string: {}",
            value.unwrap_err()
        );
        assert_eq!(value.unwrap().body(), "hello".as_bytes().to_vec());
    }

    #[test]
    fn body_file_serialisation() {
        let tmp_dir = PathBuf::from("target/tmp");
        fs::write(tmp_dir.join("hello.txt"), "hello").expect("Failed to write test file");
        let data = r#"{
            "uri": "http://some.website.com",
            "verb": "GET",
            "body": "file:hello.txt"
        }"#;

        let mut value = serde_json::from_str::<Request>(&data).unwrap();
        assert!(value.update_body(&tmp_dir).is_ok());
        assert_eq!(value.body(), "hello".as_bytes().to_vec());
    }

    #[test]
    fn body_binary_file_serialisation() {
        let tmp_dir = PathBuf::from("target/tmp");
        fs::write(tmp_dir.join("hello.bin"), "hello".as_bytes()).expect("Failed to write test file");
        let data = r#"{
            "uri": "http://some.website.com",
            "verb": "GET",
            "content_type": "Binary",
            "body": "file:hello.bin"
        }"#;

        let mut value = serde_json::from_str::<Request>(&data).unwrap();
        assert!(value.update_body(&tmp_dir).is_ok());
        assert_eq!(value.body(), "hello".as_bytes().to_vec());
    }
}
