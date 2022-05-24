use reqwest::{header::CONTENT_TYPE, RequestBuilder};
use serde::Serialize;
use std::borrow::Borrow;

pub trait AudioSource {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize)]
pub struct UrlSource<'a> {
    pub url: &'a str,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BufferSource<'a, B: Into<reqwest::Body>> {
    pub buffer: B,
    pub mimetype: Option<&'a str>,
}

impl<'a, B: Borrow<UrlSource<'a>>> AudioSource for B {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.json(self.borrow())
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
