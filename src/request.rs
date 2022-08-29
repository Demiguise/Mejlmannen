use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

pub type StringMap = std::collections::HashMap<String, String>;

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
    body: Vec<u8>,
    #[serde(default)]
    extract: StringMap,
}

impl Request {
    pub fn uri(&self) -> &String {
        &self.uri
    }
    pub fn headers(&self) -> &StringMap {
        &self.headers
    }
    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }
    pub fn verb(&self) -> Verb {
        self.verb
    }
    pub fn extract(&self) -> &StringMap {
        &self.extract
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
}

#[cfg(test)]
mod test {

    use super::{Request, StringMap, Verb};

    struct RequestBuilder {
        uri: String,
        properties: StringMap,
        headers: StringMap,
        body: Vec<u8>,
        verb: Verb,
        extract: StringMap,
    }

    impl RequestBuilder {
        pub fn new() -> Self {
            RequestBuilder {
                uri: String::new(),
                properties: StringMap::new(),
                headers: StringMap::new(),
                body: Vec::new(),
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

        pub fn body(mut self, body: Vec<u8>) -> RequestBuilder {
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
}
