// TODO: Split this all into modules

use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures::channel::mpsc::{self, Receiver};
use futures::stream::StreamExt;
use futures::{SinkExt, Stream};
use http::Request;
use pin_project::pin_project;
use reqwest::{header::CONTENT_TYPE, RequestBuilder};
use serde::Deserialize;
use thiserror::Error;
use tokio::fs::File;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_util::io::ReaderStream;
use tungstenite::handshake::client;
use url::Url;

#[derive(Debug)]
pub struct Deepgram<K>
where
    K: AsRef<str>,
{
    api_key: K,
    client: reqwest::Client,
}

#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Mimetype {
    audio_mpeg,
    audio_wav,
    // TODO: Add all mimetypes that Deepgram supports
}

pub trait Source {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder;
}

pub struct UrlSource<'a>(pub &'a str);

pub struct BufferSource<B: Into<reqwest::Body>> {
    pub buffer: B,
    pub mimetype: Mimetype,
}

#[derive(Debug)]
pub enum Model<'a> {
    General,
    Meeting,
    Phonecall,
    Voicemail,
    Finance,
    Conversational,
    Video,
    CustomId(&'a str),
}

#[derive(Debug)]
pub enum Redact {
    Pci,
    Numbers,
    Ssn,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Language {
    zh_CN,
    zh_TW,
    nl,
    en_US,
    en_AU,
    en_GB,
    en_IN,
    en_NZ,
    fr,
    fr_CA,
    de,
    hi,
    id,
    it,
    ja,
    ko,
    pt,
    pr_BR,
    ru,
    es,
    es_419,
    sv,
    tr,
    uk,
}

#[derive(Debug)]
pub enum Utterances {
    Disabled,
    Enabled { utt_split: u32 },
}

#[derive(Debug)]
pub struct OptionsBuilder<'a> {
    model: Option<Model<'a>>,
    version: Option<&'a str>,
    language: Option<Language>,
    punctuate: Option<bool>,
    profanity_filter: Option<bool>,
    redact: Vec<Redact>,
    diarize: Option<bool>,
    ner: Option<bool>,
    multichannel: Option<bool>,
    alternatives: Option<u32>,
    numerals: Option<bool>,
    search: Vec<&'a str>,
    callback: Option<&'a str>,
    keywords: Vec<&'a str>,
    utterances: Option<Utterances>,
    tag: Option<&'a str>,
}

// TODO sub-errors for the different types?
#[derive(Debug, Error)]
pub enum DeepgramError {
    #[error("No source was provided to the request builder.")]
    NoSource,
    #[error("Something went wrong when generating the http request: {0}")]
    HttpError(#[from] http::Error),
    #[error("Something went wrong when making the HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Something went wrong during I/O: {0}")]
    IoError(#[from] io::Error),
    #[error("Something went wrong with WS: {0}")]
    WsError(#[from] tungstenite::Error),
    #[error("Something went wrong during serialization/deserialization: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub struct StreamRequestBuilder<'a, S, K, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    K: AsRef<str>,
{
    config: &'a Deepgram<K>,
    source: Option<S>,
    encoding: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct Alternatives {
    pub transcript: String,
    pub words: Vec<Word>,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub alternatives: Vec<Alternatives>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StreamResponse {
    TranscriptResponse {
        duration: f64,
        is_final: bool,
        channel: Channel,
    },
    TerminalResponse {
        request_id: String,
        created: String,
        duration: f64,
        channels: u32,
    },
}

#[derive(Debug, Deserialize)]
pub struct PrerecordedResponse {
    // TODO: Define this struct
}

type Result<T> = std::result::Result<T, DeepgramError>;

#[pin_project]
struct FileChunker {
    chunk_size: usize,
    buf: BytesMut,
    #[pin]
    file: ReaderStream<File>,
}

impl FileChunker {
    fn new(file: File, chunk_size: usize) -> Self {
        FileChunker {
            chunk_size,
            buf: BytesMut::with_capacity(2 * chunk_size),
            file: ReaderStream::new(file),
        }
    }
}

impl Stream for FileChunker {
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        while this.buf.len() < *this.chunk_size {
            match Pin::new(&mut this.file).poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(next) => match next.transpose() {
                    Err(e) => return Poll::Ready(Some(Err(DeepgramError::from(e)))),
                    Ok(None) => {
                        if this.buf.len() == 0 {
                            return Poll::Ready(None);
                        } else {
                            return Poll::Ready(Some(Ok(this
                                .buf
                                .split_to(this.buf.len())
                                .freeze())));
                        }
                    }
                    Ok(Some(next)) => {
                        this.buf.extend_from_slice(&next);
                    }
                },
            }
        }

        Poll::Ready(Some(Ok(this.buf.split_to(*this.chunk_size).freeze())))
    }
}

impl<K> Deepgram<K>
where
    K: AsRef<str>,
{
    pub fn new(api_key: K) -> Self {
        Deepgram {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub fn stream_request<E, S: Stream<Item = std::result::Result<Bytes, E>>>(
        &self,
    ) -> StreamRequestBuilder<S, K, E> {
        StreamRequestBuilder {
            config: self,
            source: None,
            encoding: None,
            sample_rate: None,
            channels: None,
        }
    }

    pub async fn prerecorded_request(
        &self,
        source: impl Source,
        options: &OptionsBuilder<'_>,
    ) -> Result<PrerecordedResponse> {
        let request_builder = self
            .client
            .post("https://api.deepgram.com/v1/listen")
            .query(&options.to_query());
        let request_builder = source.fill_body(request_builder);

        Ok(request_builder.send().await?.json().await?)
    }
}

impl<'a> OptionsBuilder<'a> {
    pub fn new() -> Self {
        Self {
            model: None,
            version: None,
            language: None,
            punctuate: None,
            profanity_filter: None,
            redact: Vec::new(),
            diarize: None,
            ner: None,
            multichannel: None,
            alternatives: None,
            numerals: None,
            search: Vec::new(),
            callback: None,
            keywords: Vec::new(),
            utterances: None,
            tag: None,
        }
    }

    pub fn model(mut self, model: Model<'a>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    pub fn punctuate(mut self, punctuate: bool) -> Self {
        self.punctuate = Some(punctuate);
        self
    }

    pub fn profanity_filter(mut self, profanity_filter: bool) -> Self {
        self.profanity_filter = Some(profanity_filter);
        self
    }

    pub fn redact(mut self, redact: impl IntoIterator<Item = Redact>) -> Self {
        self.redact.extend(redact);
        self
    }

    pub fn diarize(mut self, diarize: bool) -> Self {
        self.diarize = Some(diarize);
        self
    }

    pub fn ner(mut self, ner: bool) -> Self {
        self.ner = Some(ner);
        self
    }

    pub fn multichannel(mut self, multichannel: bool) -> Self {
        self.multichannel = Some(multichannel);
        self
    }

    pub fn alternatives(mut self, alternatives: u32) -> Self {
        self.alternatives = Some(alternatives);
        self
    }

    pub fn numerals(mut self, numerals: bool) -> Self {
        self.numerals = Some(numerals);
        self
    }

    pub fn search(mut self, search: impl IntoIterator<Item = &'a str>) -> Self {
        self.search.extend(search);
        self
    }

    pub fn callback(mut self, callback: &'a str) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn keywords(mut self, keywords: impl IntoIterator<Item = &'a str>) -> Self {
        self.keywords.extend(keywords);
        self
    }

    pub fn utterances(mut self, utterances: Utterances) -> Self {
        self.utterances = Some(utterances);
        self
    }

    pub fn tag(mut self, tag: &'a str) -> Self {
        self.tag = Some(tag);
        self
    }

    fn to_query(&self) -> Vec<(&str, &str)> {
        // TODO: Generate a URL query string
        todo!()
    }
}

impl<'a, S, K, E> StreamRequestBuilder<'a, S, K, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    K: AsRef<str>,
{
    pub fn stream(mut self, stream: S) -> Self {
        self.source = Some(stream);

        self
    }

    pub fn encoding(mut self, encoding: String) -> Self {
        self.encoding = Some(encoding);

        self
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);

        self
    }

    pub fn channels(mut self, channels: u16) -> Self {
        self.channels = Some(channels);

        self
    }
}

impl<'a, K> StreamRequestBuilder<'a, Receiver<Result<Bytes>>, K, DeepgramError>
where
    K: AsRef<str>,
{
    pub async fn file(
        mut self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<StreamRequestBuilder<'a, Receiver<Result<Bytes>>, K, DeepgramError>> {
        let file = File::open(filename).await?;
        let mut chunker = FileChunker::new(file, frame_size);
        let (mut tx, rx) = mpsc::channel(1);
        let task = async move {
            while let Some(frame) = chunker.next().await {
                tokio::time::sleep(frame_delay).await;
                // This unwrap() is safe because application logic dictates that the Receiver won't
                // be dropped before the Sender.
                tx.send(frame).await.unwrap();
            }
        };
        tokio::spawn(task);

        self.source = Some(rx);
        Ok(self)
    }
}

impl<S, K, E> StreamRequestBuilder<'_, S, K, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Send + Unpin + 'static,
    K: AsRef<str>,
    E: Send + std::fmt::Debug,
{
    pub async fn start(self) -> Result<Receiver<Result<StreamResponse>>> {
        let StreamRequestBuilder {
            config,
            source,
            encoding,
            sample_rate,
            channels,
        } = self;
        let mut source = source
            .ok_or(DeepgramError::NoSource)?
            .map(|res| res.map(|bytes| Message::binary(Vec::from(bytes.as_ref()))));

        // This unwrap is safe because we're parsing a static.
        let mut base = Url::parse("wss://api.deepgram.com/v1/listen").unwrap();
        let mut pairs = base.query_pairs_mut();
        if let Some(encoding) = encoding {
            pairs.append_pair("encoding", &encoding);
        }
        if let Some(sample_rate) = sample_rate {
            pairs.append_pair("sample_rate", &sample_rate.to_string());
        }
        if let Some(channels) = channels {
            pairs.append_pair("channels", &channels.to_string());
        }

        let request = Request::builder()
            .method("GET")
            // TODO Hard-coded.
            .uri("wss://api.deepgram.com/v1/listen?encoding=linear16&sample_rate=44100&channels=2")
            .header(
                "authorization",
                format!("token {}", config.api_key.as_ref()),
            )
            .header("sec-websocket-key", client::generate_key())
            .header("host", "api.deepgram.com")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .body(())?;
        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (mut write, mut read) = ws_stream.split();
        let (mut tx, rx) = mpsc::channel::<Result<StreamResponse>>(1);

        let send_task = async move {
            loop {
                match source.next().await {
                    None => break,
                    Some(Ok(frame)) => {
                        // This unwrap is not safe.
                        write.send(frame).await.unwrap();
                    }
                    Some(e) => {
                        let _ = dbg!(e);
                        break;
                    }
                }
            }

            // This unwrap is not safe.
            write.send(Message::binary([])).await.unwrap();
        };

        let recv_task = async move {
            loop {
                match read.next().await {
                    None => break,
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(txt) => {
                                let resp = serde_json::from_str(&txt).map_err(DeepgramError::from);
                                tx.send(resp)
                                    .await
                                    // This unwrap is probably not safe.
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                    Some(e) => {
                        let _ = dbg!(e);
                        break;
                    }
                }
            }
        };

        tokio::spawn(async move {
            tokio::join!(send_task, recv_task);
        });

        Ok(rx)
    }
}

impl Source for UrlSource<'_> {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        let body = {
            let mut body = HashMap::new();
            body.insert("url", self.0);
            body
        };

        request_builder.json(&body)
    }
}

impl<B: Into<reqwest::Body>> Source for BufferSource<B> {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        // TODO: Set the correct mimetype
        let content_type: &str = todo!();

        request_builder
            .body(self.buffer)
            .header(CONTENT_TYPE, content_type)
    }
}
