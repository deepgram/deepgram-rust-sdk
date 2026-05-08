/* Expected result from running this example program.
🎤 Starting Voice Agent microphone session...
   Speak into your microphone. Press Ctrl+C to stop.

📊 Mic sample rate: 44100 Hz
Connected. dg-request-id: Some(<uuid>)
Welcome request_id: <uuid>
Settings applied
Conversation (Assistant): Hi! How can I help today?
[user speaks]
Conversation (User): What's the weather like?
Conversation (Assistant): I don't have live weather data, but...
*/

//! Two-way Voice Agent example with microphone input and audio playback.
//!
//! Captures mic audio via [`cpal`] and pipes it into
//! [`crate::agent::AgentHandle::send_data`]. Plays incoming audio chunks
//! back through the default output device via [`rodio`]. Press Ctrl+C
//! to stop.
//!
//! Audio capture and playback patterns are borrowed from
//! `microphone_flux.rs` and `text_to_speech_to_stream.rs` respectively.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features agent --example agent_microphone
//! ```

use std::env;
use std::thread;

use bytes::{BufMut, Bytes, BytesMut};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use crossbeam::channel::RecvError;
use futures::channel::mpsc::{self, Receiver as FuturesReceiver};
use futures::stream::StreamExt;
use futures::SinkExt;
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use deepgram::agent::{
    audio::{AudioConfig, AudioInput, AudioInputEncoding, AudioOutput, AudioOutputEncoding},
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
    AgentEvent, AgentResponse,
};
use deepgram::{Deepgram, DeepgramError};

/// Sample rate the agent will emit audio at. Linear16 mono.
static AGENT_OUTPUT_SAMPLE_RATE: u32 = 24_000;

macro_rules! create_stream {
    ($device:ident, $config:expr, $sync_tx:ident, $sample_type:ty) => {
        $device
            .build_input_stream(
                &$config.into(),
                move |data: &[$sample_type], _: &_| {
                    let mut bytes = BytesMut::with_capacity(data.len() * 2);
                    for sample in data {
                        bytes.put_i16_le(sample.to_sample());
                    }
                    $sync_tx.send(bytes.freeze()).unwrap();
                },
                |_| panic!(),
                None,
            )
            .unwrap()
    };
}

fn microphone_as_stream() -> (FuturesReceiver<Result<Bytes, RecvError>>, u32) {
    let (sync_tx, sync_rx) = crossbeam::channel::unbounded();
    let (mut async_tx, async_rx) = mpsc::channel(1);

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("no default input device");
    let config = device
        .default_input_config()
        .expect("no default input config");
    let sample_rate = config.sample_rate().0;

    thread::spawn(move || {
        let stream = match config.sample_format() {
            SampleFormat::F32 => create_stream!(device, config, sync_tx, f32),
            SampleFormat::I16 => create_stream!(device, config, sync_tx, i16),
            SampleFormat::U16 => create_stream!(device, config, sync_tx, u16),
            sample_format => {
                panic!("Unsupported sample format: {sample_format:?}");
            }
        };

        stream.play().unwrap();

        loop {
            thread::park();
        }
    });

    tokio::spawn(async move {
        loop {
            let data = sync_rx.recv();
            async_tx.send(data).await.unwrap();
        }
    });

    (async_rx, sample_rate)
}

/// Convert little-endian PCM16 bytes to `Vec<i16>` for rodio.
fn linear16_bytes_to_samples(bytes: &[u8]) -> Vec<i16> {
    bytes
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    println!("🎤 Starting Voice Agent microphone session...");
    println!("   Speak into your microphone. Press Ctrl+C to stop.\n");

    let (mut mic_stream, mic_sample_rate) = microphone_as_stream();
    println!("📊 Mic sample rate: {mic_sample_rate} Hz");

    let dg = Deepgram::new(&api_key)?;
    let (mut handle, mut events) = dg.agent().start().await?;

    println!("Connected. dg-request-id: {:?}", handle.request_id());

    // Output audio: linear16 at 24kHz so rodio knows what to play.
    let settings = SettingsMessage::new(
        AudioConfig::new(
            Some(AudioInput::new(
                AudioInputEncoding::Linear16,
                mic_sample_rate,
            )),
            Some(
                AudioOutput::new()
                    .with_encoding(AudioOutputEncoding::Linear16)
                    .with_sample_rate(AGENT_OUTPUT_SAMPLE_RATE),
            ),
        ),
        AgentConfig::inline(
            InlineAgentConfig::from_parts(
                AgentListenSettings::new(AgentListenProvider::DeepgramV2(
                    DeepgramListenV2Provider::new("flux-general-en"),
                )),
                ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
                    OpenAiModel::Gpt4oMini,
                ))),
                SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                    DeepgramSpeakModel::Aura2ThaliaEn,
                ))),
            )
            .with_greeting("Hi! How can I help today?"),
        ),
    );
    handle.send_settings(settings).await?;

    // Set up rodio playback for inbound audio.
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("failed to open default audio output");
    let sink = Sink::try_new(&stream_handle).expect("failed to create rodio sink");

    // Run mic-forwarding concurrently with the event loop.
    loop {
        tokio::select! {
            // Forward mic audio chunks to the agent.
            chunk = mic_stream.next() => {
                match chunk {
                    Some(Ok(audio)) => {
                        if let Err(err) = handle.send_data(audio.to_vec()).await {
                            eprintln!("send_data failed: {err}");
                            break;
                        }
                    }
                    Some(Err(err)) => {
                        eprintln!("mic stream error: {err}");
                        break;
                    }
                    None => {
                        println!("mic stream ended");
                        break;
                    }
                }
            }
            // Receive agent events and audio.
            event = events.next() => {
                match event {
                    Some(Ok(AgentEvent::Json(response))) => match response {
                        AgentResponse::Welcome(w) => {
                            println!("Welcome request_id: {}", w.request_id);
                        }
                        AgentResponse::SettingsApplied(_) => {
                            println!("Settings applied");
                        }
                        AgentResponse::ConversationText(c) => {
                            println!("Conversation ({:?}): {}", c.role, c.content);
                        }
                        AgentResponse::UserStartedSpeaking(_) => {
                            // Optional: clear the playback sink so the agent
                            // doesn't keep talking over the user.
                            sink.clear();
                            sink.play();
                        }
                        AgentResponse::Error(e) => {
                            eprintln!("Error [{}]: {}", e.code, e.description);
                            break;
                        }
                        AgentResponse::Warning(w) => {
                            println!("Warning [{}]: {}", w.code, w.description);
                        }
                        _ => {}
                    },
                    Some(Ok(AgentEvent::Audio(bytes))) => {
                        let samples = linear16_bytes_to_samples(&bytes);
                        if !samples.is_empty() {
                            sink.append(SamplesBuffer::new(
                                1,
                                AGENT_OUTPUT_SAMPLE_RATE,
                                samples,
                            ));
                        }
                    }
                    Some(Ok(_)) => {} // AgentEvent #[non_exhaustive]
                    Some(Err(err)) => {
                        eprintln!("Stream error: {err}");
                        break;
                    }
                    None => {
                        println!("Server closed connection.");
                        break;
                    }
                }
            }
        }
    }

    sink.sleep_until_end();
    handle.close().await?;
    Ok(())
}
