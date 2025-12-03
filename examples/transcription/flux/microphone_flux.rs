use std::env;
use std::io::Write;
use std::thread;

use bytes::{BufMut, Bytes, BytesMut};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use crossbeam::channel::RecvError;
use deepgram::common::options::{Encoding, Model, Options};
use futures::channel::mpsc::{self, Receiver as FuturesReceiver};
use futures::stream::StreamExt;
use futures::SinkExt;

use deepgram::{
    common::flux_response::{FluxResponse, TurnEvent},
    Deepgram, DeepgramError,
};

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
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();
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

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    // Configure Flux for more reliable turn detection
    // - eot_threshold: 0.75 (higher = more reliable, less false positives)
    // - eot_timeout_ms: 5000 (default, allows longer pauses before forcing turn end)
    // - eager_eot_threshold: None (disabled - remove if you want early LLM responses)
    let options = Options::builder()
        .model(Model::FluxGeneralEn)
        .eot_threshold(0.75)
        .eot_timeout_ms(5000)
        // Uncomment below if you want early response generation (increases LLM calls by 50-70%)
        // .eager_eot_threshold(0.7)
        .build();

    println!("🎤 Starting Flux microphone transcription...");
    println!("   Speak into your microphone. Press Ctrl+C to stop.\n");

    let (mic_stream, sample_rate) = microphone_as_stream();
    println!("📊 Using sample rate: {} Hz\n", sample_rate);

    let mut results = dg_client
        .transcription()
        .flux_request_with_options(options)
        .encoding(Encoding::Linear16)
        .sample_rate(sample_rate)
        .stream(mic_stream)
        .await?;

    println!("Flux Request ID: {}\n", results.request_id());

    while let Some(result) = results.next().await {
        match result? {
            FluxResponse::Connected {
                request_id,
                sequence_id,
            } => {
                println!("✓ Connected: {} (seq: {})\n", request_id, sequence_id);
            }
            FluxResponse::TurnInfo {
                event,
                turn_index,
                transcript,
                end_of_turn_confidence,
                words,
                ..
            } => match event {
                TurnEvent::StartOfTurn => {
                    println!("\n▶ [Turn {}] START", turn_index);
                }
                TurnEvent::EndOfTurn => {
                    println!(
                        "\n✓ [Turn {}] END (conf: {:.2}): {}",
                        turn_index, end_of_turn_confidence, transcript
                    );
                    println!("  Words: {}\n", words.len());
                }
                TurnEvent::EagerEndOfTurn => {
                    println!("\n⚡ [Turn {}] EAGER END: {}\n", turn_index, transcript);
                }
                TurnEvent::TurnResumed => {
                    println!("\n↻ [Turn {}] RESUMED: {}\n", turn_index, transcript);
                }
                TurnEvent::Update => {
                    if !transcript.is_empty() {
                        print!("\r[Turn {}] UPDATE: {}", turn_index, transcript);
                        std::io::stdout().flush().unwrap();
                    }
                }
                _ => {
                    println!("\n[Turn {}] Unknown event: {:?}", turn_index, event);
                }
            },
            FluxResponse::FatalError {
                code, description, ..
            } => {
                eprintln!("\n❌ Error {}: {}", code, description);
                break;
            }
            _ => {
                println!("Unknown response type");
            }
        }
    }

    Ok(())
}
