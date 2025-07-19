#![allow(missing_docs)]
//! WebSocket TTS module

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    speak::options::{Encoding, Model},
    Deepgram, DeepgramError, Result, Speak,
};

use anyhow::anyhow;
use bytes::Bytes;
use futures::{select, SinkExt, Stream, StreamExt};
use http::Request;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::{handshake::client, Message};
use url::Url;
use uuid::Uuid;

static TTS_STREAM_PATH: &str = "v1/speak";

/// TODO docs
#[derive(Clone, Debug)]
pub struct WebsocketBuilder<'a> {
    deepgram: &'a Deepgram,
    encoding: Option<Encoding>,
    model: Option<Model>,
    sample_rate: Option<u32>,
}

impl<'a> WebsocketBuilder<'a> {
    pub fn as_url(&self) -> Result<Url, DeepgramError> {
        let mut url =
            self.deepgram.base_url.join(TTS_STREAM_PATH).expect(
                "base_url is checked to be a valid base_url when constructing Deepgram client",
            );

        match url.scheme() {
            "http" | "ws" => url.set_scheme("ws").expect("a valid conversion according to the .set_scheme docs"),
            "https" | "wss" => url.set_scheme("wss").expect("a valid conversion according to the .set_scheme docs"),
            _ => unreachable!("base_url is validated to have a scheme of http, https, ws, or wss when constructing Deepgram client"),
        }

        {
            let mut pairs = url.query_pairs_mut();

            if let Some(encoding) = self.encoding.as_ref() {
                pairs.append_pair("encoding", encoding.as_str());
            }

            if let Some(model) = self.model.as_ref() {
                pairs.append_pair("model", model.as_ref());
            }

            if let Some(sample_rate) = self.sample_rate {
                pairs.append_pair("sample_rate", sample_rate.to_string().as_str());
            }
        }

        Ok(url)
    }

    pub async fn handle(self) -> Result<WebsocketHandle> {
        WebsocketHandle::new(self).await
    }

    pub async fn stream<S, E>(self, stream: S) -> Result<SpeakAudioStream>
    where
        S: Stream<Item = Result<String, E>> + Send + Unpin + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let handle = self.handle().await?;
        let request_tx = handle.message_tx;
        let mut text_stream = stream.fuse();
        let mut response_rx = ReceiverStream::new(handle.response_rx).fuse();

        tokio::task::spawn(async move {
            loop {
                select! {
                    t = text_stream.next() => {
                        eprintln!("Text stream: {:?}", t);
                        match t {
                            Some(Ok(text)) => {
                                if let Err(_) = request_tx.send(SpeakWsMessage::Speak { text }).await {
                                    break;
                                }
                            }
                            Some(Err(_err)) => {
                                break;
                            }
                            None => {
                                //when the text input stream closes, queue a close command
                                //on the websocket channel
                                let _ = request_tx.send(SpeakWsMessage::Close).await;
                            }
                        }
                    }
                    r = response_rx.next() => {
                        eprintln!("Response: {:?}", r);
                    }
                }
            }
        });

        let audio_stream = SpeakAudioStream {
            rx: handle.audio_rx,
        };

        Ok(audio_stream)
    }
}

/// TODO docs
#[derive(Debug)]
pub struct WebsocketHandle {
    message_tx: mpsc::Sender<SpeakWsMessage>,
    response_rx: mpsc::Receiver<Result<SpeakResponse>>,
    audio_rx: mpsc::Receiver<Result<Bytes, DeepgramError>>,
    request_id: Uuid,
}

impl WebsocketHandle {
    async fn new(builder: WebsocketBuilder<'_>) -> Result<WebsocketHandle> {
        let url = builder.as_url()?;
        let host = url.host_str().ok_or(DeepgramError::InvalidUrl)?;

        let request = {
            let http_builder = Request::builder()
                .method("GET")
                .uri(url.to_string())
                .header("sec-websocket-key", client::generate_key())
                .header("host", host)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13");

            let builder = if let Some(auth) = &builder.deepgram.auth {
                http_builder.header("authorization", auth.header_value())
            } else {
                http_builder
            };
            builder.body(())?
        };

        eprintln!("WS Speech Request: {:?}", request);

        let (ws_stream, upgrade_response) = tokio_tungstenite::connect_async(request).await?;

        let request_id = upgrade_response
            .headers()
            .get("dg-request-id")
            .ok_or(DeepgramError::UnexpectedServerResponse(anyhow!(
                "Websocket upgrade headers missing request ID"
            )))?
            .to_str()
            .ok()
            .and_then(|req_header_str| Uuid::parse_str(req_header_str).ok())
            .ok_or(DeepgramError::UnexpectedServerResponse(anyhow!(
                "Received malformed request ID in websocket upgrade headers"
            )))?;

        let (message_tx, message_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(256);
        let (audio_tx, audio_rx) = mpsc::channel(256);

        tokio::task::spawn({
            let worker = WsWorker::new(ws_stream, message_rx, response_tx, audio_tx);

            async move {
                if let Err(err) = worker.run().await {
                    tracing::error!("speak websocket worker error: {:?}", err);
                }
            }
        });

        Ok(WebsocketHandle {
            message_tx,
            response_rx,
            audio_rx,
            request_id,
        })
    }

    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    pub async fn send_text(&self, text: String) -> Result<()> {
        eprintln!("Sending text: {}", text);
        if let Err(_) = self.message_tx.send(SpeakWsMessage::Speak { text }).await {
            return Err(DeepgramError::UnexpectedServerResponse(anyhow!(
                "websocket closed"
            )));
        }

        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let _ = self.message_tx.send(SpeakWsMessage::Flush).await;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SpeakAudioStream {
    rx: mpsc::Receiver<Result<Bytes, DeepgramError>>,
}

impl Stream for SpeakAudioStream {
    type Item = Result<Bytes, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().rx.poll_recv(cx)
    }
}

impl<'a> Speak<'a> {
    /// Opens a websocket connection to the Deepgram API to birectionally
    /// stream text input and audio output
    pub fn continuous_speak_to_stream(&self) -> WebsocketBuilder<'_> {
        WebsocketBuilder {
            deepgram: self.0,
            encoding: Some(Encoding::Linear16),
            model: Some(Model::CustomId("aura-2-thalia-en".to_string())),
            sample_rate: Some(24000),
        }
    }
}

/// TODO docs
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SpeakWsMessage {
    Speak { text: String },
    Flush,
    Clear,
    Close,
}

/// TODO docs
#[derive(Debug)]
pub enum StreamResponse {
    Audio(Bytes),
    Control(SpeakResponse),
}

/// TODO docs
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SpeakResponse {
    Flush {
        sequence_id: u64,
    },
    Clear {
        sequence_id: u64,
    },
    Close {
        sequence_id: u64,
    },
    StreamClosed {
        code: u64,
        reason: Option<String>,
    },
    Metadata {
        request_id: String,
        model_name: String,
    },
}

#[derive(Debug)]
pub struct WsWorker {
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    request_rx: mpsc::Receiver<SpeakWsMessage>,
    response_tx: mpsc::Sender<Result<SpeakResponse>>,
    audio_tx: mpsc::Sender<Result<Bytes, DeepgramError>>,
}

impl WsWorker {
    pub fn new(
        ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
        request_rx: mpsc::Receiver<SpeakWsMessage>,
        response_tx: mpsc::Sender<Result<SpeakResponse>>,
        audio_tx: mpsc::Sender<Result<Bytes, DeepgramError>>,
    ) -> Self {
        Self {
            ws_stream,
            request_rx,
            response_tx,
            audio_tx,
        }
    }

    async fn run(self) -> Result<()> {
        let (mut ws_stream_send, ws_stream_recv) = self.ws_stream.split();
        let mut ws_recv = ws_stream_recv.fuse();
        let mut request_rx = ReceiverStream::new(self.request_rx).fuse();

        loop {
            select! {
                response = ws_recv.next() => {
                    match response {
                        Some(Ok(Message::Text(response))) => {
                            eprintln!("Received text: {}", response);
                            match serde_json::from_str::<SpeakResponse>(&response) {
                                Ok(response) => {
                                    if (self.response_tx.send(Ok(response)).await).is_err() {
                                        break;
                                    }
                                }
                                Err(err) => {
                                    if (self.response_tx.send(Err(err.into())).await).is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Binary(audio))) => {
                            eprintln!("Received audio");
                            if (self.audio_tx.send(Ok(audio)).await).is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            return Ok(())
                        }
                        Some(Ok(Message::Ping(ping))) => {
                            // We don't really care if the server receives the pong.
                            let _ = ws_stream_send.send(Message::Pong(ping)).await;
                        }
                        Some(Ok(Message::Pong(_))) => { }
                        Some(Ok(Message::Frame(_))) => {
                            eprintln!("Received frame");
                            // We don't care about frames (I think).
                        }
                        Some(Err(err)) => {
                            if (self.response_tx.send(Err(err.into())).await).is_err() {
                                break;
                            }
                        }
                        None => {
                            return Ok(())
                        }
                    }
                }

                request = request_rx.next() => {
                    match request {
                        Some(request) => {
                            let msg = serde_json::to_string(&request)?;
                            eprintln!("Sending message: {}", msg);
                            if let Err(_) = ws_stream_send.send(Message::Text(msg.into())).await {
                                break;
                            }
                        }
                        None => {
                            return Ok(())
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
