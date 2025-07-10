use std::env;
use std::thread;

use bytes::{BufMut, Bytes, BytesMut};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use crossbeam::channel::RecvError;
use deepgram::common::options::Encoding;
use futures::channel::mpsc::{self, Receiver as FuturesReceiver};
use futures::stream::StreamExt;
use futures::SinkExt;

use deepgram::{Deepgram, DeepgramError};

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

fn microphone_as_stream() -> FuturesReceiver<Result<Bytes, RecvError>> {
    let (sync_tx, sync_rx) = crossbeam::channel::unbounded();
    let (mut async_tx, async_rx) = mpsc::channel(1);

    thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();

        // let config = device.supported_input_configs().unwrap();
        // for config in config {
        //     dbg!(&config);
        // }

        let config = device.default_input_config().unwrap();

        // dbg!(&config);

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

    async_rx
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let mut results = dg_client
        .transcription()
        .stream_request()
        .keep_alive()
        .encoding(Encoding::Linear16)
        // TODO Specific to my machine, not general enough example.
        .sample_rate(44100)
        // TODO Specific to my machine, not general enough example.
        .channels(2)
        .stream(microphone_as_stream())
        .await?;

    println!("Deepgram Request ID: {}", results.request_id());
    while let Some(result) = results.next().await {
        println!("got: {result:?}");
    }

    Ok(())
}
