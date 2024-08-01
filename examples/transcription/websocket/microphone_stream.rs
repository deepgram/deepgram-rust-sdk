use std::env;
use std::thread;

use bytes::{BufMut, Bytes, BytesMut};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use crossbeam::channel::RecvError;
use deepgram::common::options::Encoding;
use deepgram::common::options::Language;
use deepgram::common::options::Model;
use deepgram::common::options::Options;
use deepgram::listen::websocket::Event;
use futures::channel::mpsc::{self, Receiver as FuturesReceiver};
use futures::SinkExt;
use futures_util::stream::StreamExt;

use deepgram::{Deepgram, DeepgramError};

fn microphone_as_stream() -> FuturesReceiver<Result<Bytes, RecvError>> {
    let (sync_tx, sync_rx) = crossbeam::channel::unbounded();
    let (mut async_tx, async_rx) = mpsc::channel(1);

    thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();

        let config = device.default_input_config().unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        let mut bytes = BytesMut::with_capacity(data.len() * 2);
                        for sample in data {
                            bytes.put_i16_le(sample.to_i16());
                        }
                        sync_tx.send(bytes.freeze()).unwrap();
                    },
                    |_| panic!(),
                )
                .unwrap(),
            cpal::SampleFormat::I16 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &_| {
                        let mut bytes = BytesMut::with_capacity(data.len() * 2);
                        for sample in data {
                            bytes.put_i16_le(*sample);
                        }
                        sync_tx.send(bytes.freeze()).unwrap();
                    },
                    |_| panic!(),
                )
                .unwrap(),
            cpal::SampleFormat::U16 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &_| {
                        let mut bytes = BytesMut::with_capacity(data.len() * 2);
                        for sample in data {
                            bytes.put_i16_le(sample.to_i16());
                        }
                        sync_tx.send(bytes.freeze()).unwrap();
                    },
                    |_| panic!(),
                )
                .unwrap(),
        };

        stream.play().unwrap();

        loop {
            thread::park();
        }
    });

    tokio::spawn(async move {
        loop {
            match sync_rx.recv() {
                Ok(data) => {
                    if let Err(e) = async_tx.send(Ok(data)).await {
                        eprintln!("Failed to send data: {:?}", e);
                        break; // Exit the loop if the channel is disconnected
                    }
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {:?}", e);
                    if let Err(send_err) = async_tx.send(Err(e)).await {
                        eprintln!("Failed to send error: {:?}", send_err);
                    }
                    break; // Exit the loop if the receiving end is closed
                }
            }
        }
    });

    async_rx
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg = Deepgram::new(env::var("DEEPGRAM_API_KEY").unwrap());

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<Event>(100);

    // Event handling task
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                Event::Open => println!("Connection opened"),
                Event::Close => println!("Connection closed"),
                Event::Error(e) => eprintln!("Error occurred: {:?}", e),
                Event::Result(result) => println!("got: {:?}", result),
            }
        }
    });

    let options = Options::builder()
        .model(Model::Nova2)
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let (connection, mut response_stream) = dg
        .transcription()
        .stream_request_with_options(options)
        .keep_alive()
        .stream(microphone_as_stream())
        .encoding(Encoding::Linear16)
        // TODO Specific to my machine, not general enough example.
        .sample_rate(44100)
        // TODO Specific to my machine, not general enough example.
        .channels(2)
        .start(event_tx.clone())
        .await?;

    let mut count = 0;

    while let Some(response) = response_stream.next().await {
        // Close the stream after 5 messages are received
        if count == 5 {
            // Call finalize after processing the stream
            connection.finalize(event_tx.clone()).await?;

            // Call  after processing the stream
            connection.finish(event_tx.clone()).await?;
        }
        count += 1;
        match response {
            Ok(result) => println!("Transcription result: {:?}", result),
            Err(e) => eprintln!("Transcription error: {:?}", e),
        }
    }

    Ok(())
}
