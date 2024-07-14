use std::env;
use deepgram::{speak::rest::options::Options, Deepgram, DeepgramError};
use futures::stream::StreamExt;
use rodio::{OutputStream, Sink, Source};
use bytes::BytesMut;
use std::time::{Duration, Instant};
use std::vec::IntoIter;

pub struct Linear16Source {
    samples: IntoIter<i16>,
    sample_rate: u32,
    channels: u16,
}

impl Linear16Source {
    pub fn new(data: Vec<u8>, sample_rate: u32, channels: u16) -> Self {
        // Convert the raw bytes to i16 samples
        let samples = data.chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>()
            .into_iter();

        Linear16Source {
            samples,
            sample_rate,
            channels,
        }
    }
}

impl Iterator for Linear16Source {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples.next()
    }
}

impl Source for Linear16Source {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len())
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let sample_rate = 16000;
    let channels = 1;

    let options = Options::builder()
        .model("aura-asteria-en")
        .encoding("linear16")
        .sample_rate(sample_rate)
        .container("none")
        .build();

    let text = "Hello, how can I help you today? This is a longer sentence to increase the time taken to process the audio, so that the streaming shows the full delta vs downloading the whole file.";

    // Record the start time
    let start_time = Instant::now();

    let mut audio_stream = dg_client
        .text_to_speech()
        .speak_to_stream(text, &options)
        .await?;

    // Set up audio output
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Buffer to accumulate initial audio data
    let mut buffer = BytesMut::new();
    // Define a threshold for the initial buffer (e.g., 32000 bytes for 2 seconds)
    let buffer_threshold = sample_rate as usize * 2 * channels as usize * 1; // 2 seconds of audio

    // Flag to indicate if timing information has been printed
    let mut timing_printed = false;

    println!("1st while loop");
    // Accumulate initial buffer
    while let Some(data) = audio_stream.next().await {
        // Print timing information if not already printed
        if !timing_printed {
            let elapsed_time = start_time.elapsed();
            println!("Time to first audio byte: {:.2?}", elapsed_time);
            timing_printed = true;
        }

        // Process and accumulate the audio data here
        println!("Received {} bytes of audio data", data.len());
        buffer.extend_from_slice(&data);

        // Check if buffer has reached the initial threshold
        if buffer.len() >= buffer_threshold {
            let source = Linear16Source::new(buffer.split().to_vec(), sample_rate, channels);
            sink.append(source);

            // Start playing the audio and break to start streaming in smaller chunks
            break;
        }
    }

    println!("2nd while loop");
    // Continue streaming the audio in smaller chunks
    while let Some(data) = audio_stream.next().await {
        // Process and accumulate the audio data here
        println!("Received {} bytes of audio data", data.len());
        buffer.extend_from_slice(&data);

        // Check if buffer has enough data to continue streaming
        if buffer.len() >= buffer_threshold {
            let source = Linear16Source::new(buffer.split().to_vec(), sample_rate, channels);
            sink.append(source);
        }
    }

    println!("play end of buffer");
    // Play any remaining buffered data
    if !buffer.is_empty() {
        let source = Linear16Source::new(buffer.to_vec(), sample_rate, channels);
        sink.append(source);
    }

    println!("Received all audio data");

    // Ensure all audio is played before exiting
    sink.sleep_until_end();

    Ok(())
}