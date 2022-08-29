use lazy_static::lazy_static;
use regex::Regex;

pub type StringMap = std::collections::HashMap<String, String>;

lazy_static! {
    static ref RE: Regex = Regex::new(r"\{{1,2}(\w*)\}{1,2}")
        .expect("Failed to create regex for Request data replacement");
}

// TODO: Expand
#[derive(Debug, Clone, Copy)]
pub enum Verb {
    GET,
    POST,
    DELETE,
}

#[derive(Debug)]
pub struct Request {
    uri: String,
    properties: StringMap,
    headers: StringMap,
    body: Vec<u8>,
    verb: Verb,
    extract: StringMap,
}

pub struct RequestBuilder {
    uri: String,
    global_properties: StringMap,
    local_properties: StringMap,
    test_properties: StringMap,
    headers: StringMap,
    body: Vec<u8>,
    verb: Verb,
    extract: StringMap,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder {
            uri: String::new(),
            global_properties: StringMap::new(),
            local_properties: StringMap::new(),
            test_properties: StringMap::new(),
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

    pub fn global_properties(mut self, global: StringMap) -> RequestBuilder {
        self.global_properties = global;
        self
    }

    pub fn local_properties(mut self, local: StringMap) -> RequestBuilder {
        self.local_properties = local;
        self
    }

    pub fn test_properties(mut self, test: StringMap) -> RequestBuilder {
        self.test_properties = test;
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
        let mut merged_properties = self.global_properties.clone();
        merged_properties.extend(self.local_properties);
        merged_properties.extend(self.test_properties);
        Request {
            uri: self.uri,
            properties: merged_properties,
            headers: self.headers,
            body: self.body,
            verb: self.verb,
            extract: self.extract,
        }
    }
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

    fn replace_text(&self, text: &String) -> String {
        RE.replace_all(text, |caps: &regex::Captures| {
            if caps[0].starts_with("{{") && caps[0].ends_with("}}") {
                // Escaped {} string, just return the inner string
                caps[0][1..caps[0].len() - 1].to_owned()
            } else {
                // Normal replacement of a variable
                match self.properties.get(&caps[1]) {
                    Some(value) => value.clone(),
                    None => caps[0].to_owned(), // Just return the matched string instead
                }
            }
        })
        .into_owned()
    }

    // TODO: Replace these with in pre-prepared versions?
    pub fn replaced_uri(&self) -> String {
        self.replace_text(&self.uri)
    }

    pub fn replaced_headers(&self) -> StringMap {
        let mut map = StringMap::new();
        self.headers.iter().for_each(|(key, value)| {
            map.insert(key.clone(), self.replace_text(&value));
        });

        map
    }
}

#[cfg(test)]
mod test {
    use super::StringMap;
    use super::RequestBuilder;

    #[test]
    fn new_builder_is_empty() {
        let builder = RequestBuilder::new();
        assert!(builder.uri.is_empty());
        assert!(builder.global_properties.is_empty());
        assert!(builder.local_properties.is_empty());
        assert!(builder.test_properties.is_empty());
        assert!(builder.headers.is_empty());
    }

    #[test]
    fn properties_are_merged() {
        let mut global = StringMap::new();
        global.insert("Global_Key".to_owned(), "Global Value".to_owned());
        global.insert("Local_Key".to_owned(), "Global Value".to_owned());
        global.insert("Test_Key".to_owned(), "Global Value".to_owned());

        let mut local = StringMap::new();
        local.insert("Local_Key".to_owned(), "Local Value".to_owned());
        local.insert("Test_Key".to_owned(), "Local Value".to_owned());

        let mut test = StringMap::new();
        test.insert("Test_Key".to_owned(), "Test Value".to_owned());

        let request = RequestBuilder::new()
            .global_properties(global)
            .local_properties(local)
            .test_properties(test)
            .build();

        assert_eq!(
            request.properties.get("Global_Key"),
            Some(&"Global Value".to_owned())
        );
        assert_eq!(
            request.properties.get("Local_Key"),
            Some(&"Local Value".to_owned())
        );
        assert_eq!(
            request.properties.get("Test_Key"),
            Some(&"Test Value".to_owned())
        );
    }

    #[test]
    fn uri_replaced() {
        let mut global = StringMap::new();
        global.insert("some_key".to_owned(), "some_value".to_owned());

        let request = RequestBuilder::new()
            .uri("URI/{some_key}/URI".to_owned())
            .global_properties(global)
            .build();

        assert_eq!(request.replaced_uri(), "URI/some_value/URI");
    }

    #[test]
    fn uri_replaced_escaped_braces() {
        // It should escape the key even if it's available in the property maps
        let mut global = StringMap::new();
        global.insert("some_key".to_owned(), "some_value".to_owned());

        let request = RequestBuilder::new()
            .uri("URI/{{some_key}}/URI".to_owned())
            .global_properties(global)
            .build();

        assert_eq!(request.replaced_uri(), "URI/{some_key}/URI");
    }

    #[test]
    fn uri_replaced_empty_escaped_braces() {
        let request = RequestBuilder::new().uri("URI/{{}}/URI".to_owned()).build();

        assert_eq!(request.replaced_uri(), "URI/{}/URI");
    }

    #[test]
    fn uri_replaced_unknown_key() {
        let request = RequestBuilder::new()
            .uri("URI/{some_key}/URI".to_owned())
            .build();

        assert_eq!(request.replaced_uri(), "URI/{some_key}/URI");
    }

    #[test]
    fn uri_replaced_empty_key() {
        let request = RequestBuilder::new().uri("URI/{}/URI".to_owned()).build();

        assert_eq!(request.replaced_uri(), "URI/{}/URI");
    }

    #[test]
    fn headers_replaced() {
        let mut global = StringMap::new();
        global.insert("some_key".to_owned(), "some_value".to_owned());

        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{some_key}".to_owned())
            .global_properties(global)
            .build();

        assert_eq!(
            request.replaced_headers().get("A_HEADER"),
            Some(&"some_value".to_owned())
        );
    }

    #[test]
    fn headers_replaced_unknown_key() {
        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{some_key}".to_owned())
            .build();

        assert_eq!(
            request.replaced_headers().get("A_HEADER"),
            Some(&"{some_key}".to_owned())
        );
    }

    #[test]
    fn headers_replaced_empty_key() {
        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{}".to_owned())
            .build();

        assert_eq!(
            request.replaced_headers().get("A_HEADER"),
            Some(&"{}".to_owned())
        );
    }

    #[test]
    fn headers_replaced_escaped_key() {
        let request = RequestBuilder::new()
            .header("A_HEADER".to_owned(), "{{some_key}}".to_owned())
            .build();

        assert_eq!(
            request.replaced_headers().get("A_HEADER"),
            Some(&"{some_key}".to_owned())
        );
    }
}
