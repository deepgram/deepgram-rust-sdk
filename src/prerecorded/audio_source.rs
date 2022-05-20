use reqwest::{header::CONTENT_TYPE, RequestBuilder};
use std::borrow::Borrow;
use std::collections::HashMap;

pub trait AudioSource {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder;
}

pub struct UrlSource<'a>(pub &'a str);

pub struct BufferSource<'a, B: Into<reqwest::Body>> {
    pub buffer: B,
    pub mimetype: Option<&'a str>,
}

impl<'a, B: Borrow<UrlSource<'a>>> AudioSource for B {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        let body: HashMap<&str, &str> = HashMap::from([("url", self.borrow().0)]);

        request_builder.json(&body)
    }
}

impl<B: Into<reqwest::Body>> AudioSource for BufferSource<'_, B> {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        let request_builder = request_builder.body(self.buffer);

        if let Some(mimetype) = self.mimetype {
            request_builder.header(CONTENT_TYPE, mimetype)
        } else {
            request_builder
        }
    }
}
