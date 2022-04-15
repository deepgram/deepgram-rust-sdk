use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::{Bytes, BytesMut};
// use futures::channel::mpsc;
use futures::stream::{Map, StreamExt};
use futures::Stream;
use thiserror::Error;
use tokio::fs::File;
use tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio_util::codec::FramedRead;

#[derive(Debug)]
pub struct Deepgram<'a> {
    api_key: &'a str,
}

// TODO sub-errors for the different types?
#[derive(Debug, Error)]
pub enum DeepgramError {
    #[error("something went wrong during I/O: {0}")]
    IoError(#[from] io::Error),
}

pub struct StreamRequestBuilder<'a, S>
where
    S: Stream<Item = Result<Bytes>>,
{
    config: &'a Deepgram<'a>,
    source: Option<S>,
}

pub struct StreamRequest;
type Result<T> = std::result::Result<T, DeepgramError>;

impl<'a> Deepgram<'a> {
    // AsRef<str>
    pub fn new(api_key: &'a str) -> Self {
        Deepgram { api_key }
    }

    pub fn stream_request<S: Stream<Item = Result<Bytes>>>(&self) -> StreamRequestBuilder<S> {
        StreamRequestBuilder {
            config: self,
            source: None,
        }
    }
}

fn codec_result_to_deepgram_result(
    res: std::result::Result<BytesMut, std::io::Error>,
) -> Result<Bytes> {
    res.map(|b| b.freeze()).map_err(|e| DeepgramError::from(e))
}

impl<'a>
    StreamRequestBuilder<
        'a,
        Map<
            FramedRead<File, LengthDelimitedCodec>,
            fn(std::result::Result<BytesMut, std::io::Error>) -> Result<Bytes>,
        >,
    >
{
    // Into<Path>
    pub async fn file(
        mut self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<
        StreamRequestBuilder<
            'a,
            Map<
                FramedRead<File, LengthDelimitedCodec>,
                fn(std::result::Result<BytesMut, std::io::Error>) -> Result<Bytes>,
            >,
        >,
    > {
        let file = File::open(filename).await?;
        let reader = LengthDelimitedCodec::builder()
            .length_field_length(0)
            .length_adjustment(frame_size as isize)
            .new_read(file)
            .map(
                codec_result_to_deepgram_result
                    as fn(std::result::Result<BytesMut, std::io::Error>) -> Result<Bytes>,
            );
        self.source = Some(reader);

        Ok(self)
    }
}

impl<S> StreamRequestBuilder<'_, S>
where
    S: Stream<Item = Result<Bytes>>,
{
    pub async fn start(&self) -> Result<StreamRequest> {
        Ok(StreamRequest)
    }
}

impl Stream for StreamRequest {
    type Item = i32;

    fn poll_next(self: Pin<&mut Self>, _context: &mut Context) -> Poll<Option<Self::Item>> {
        Poll::Ready(None)
    }
}
