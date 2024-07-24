use audio::channel::LinearChannel;
use audio::Buf;
use bytes::BytesMut;
use deepgram::speak::options::{Container, Encoding, Model};
use deepgram::{speak::options::Options, Deepgram, DeepgramError};
use futures::stream::StreamExt;
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use std::env;
use std::time::Instant;

#[derive(Clone)]
pub struct Linear16AudioSource {
    sample_rate: u32,
    channels: u16,
    buffer: Vec<i16>,
}

impl Linear16AudioSource {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            buffer: Vec::new(),
        }
    }

    pub fn push_samples(&mut self, samples: &[i16]) {
        self.buffer.extend_from_slice(samples);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn take_buffer(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.buffer)
    }
}

impl Buf for Linear16AudioSource {
    type Sample = i16;

    type Channel<'this> = LinearChannel<'this, i16>
    where
        Self: 'this;

    type IterChannels<'this> = std::vec::IntoIter<LinearChannel<'this, i16>>
    where
        Self: 'this;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.buffer.len() / self.channels as usize)
    }

    fn channels(&self) -> usize {
        self.channels as usize
    }

    fn get_channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        if channel < self.channels as usize {
            Some(LinearChannel::new(&self.buffer[channel..]))
        } else {
            None
        }
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        (0..self.channels as usize)
            .map(|channel| LinearChannel::new(&self.buffer[channel..]))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

pub fn play_audio(sink: &Sink, sample_rate: u32, channels: u16, samples: Vec<i16>) {
    // Create a rodio source from the raw audio data
    let source = SamplesBuffer::new(channels, sample_rate, samples);

    // Play the audio
    sink.append(source);
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let sample_rate = 16000;
    let channels = 1;

    let options = Options::builder()
        .model(Model::AuraAsteriaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(sample_rate)
        .container(Container::Wav)
        .build();

    let text = "Hello, how can I help you today? This is a longer sentence to increase the time taken to process the audio, so that the streaming shows the full delta vs downloading the whole file.";

    // Record the start time
    let start_time = Instant::now();

    let audio_stream = dg_client
        .text_to_speech()
        .speak_to_stream(text, &options)
        .await?;

    // Set up audio output
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Create the audio source
    let mut source = Linear16AudioSource::new(sample_rate, channels);

    // Use the audio_stream for streaming audio and play it
    let mut stream = audio_stream;
    let mut buffer = BytesMut::new();
    let mut extra_byte: Option<u8> = None;

    // Define a threshold for the buffer (e.g., 32000 bytes for 1 second)
    let buffer_threshold = 0; // increase for slow networks

    // Flag to indicate if timing information has been printed
    let mut time_to_first_byte_printed = false;

    // Accumulate initial buffer
    while let Some(data) = stream.next().await {
        // Print timing information if not already printed
        if !time_to_first_byte_printed {
            let elapsed_time = start_time.elapsed();
            println!("Time to first audio byte: {:.2?}", elapsed_time);
            time_to_first_byte_printed = true;
        }

        // Process and accumulate the audio data here
        buffer.extend_from_slice(&data);

        // Prepend the extra byte if present
        if let Some(byte) = extra_byte.take() {
            let mut new_buffer = BytesMut::with_capacity(buffer.len() + 1);
            new_buffer.extend_from_slice(&[byte]);
            new_buffer.extend_from_slice(&buffer);
            buffer = new_buffer;
        }

        // Check if buffer has reached the initial threshold
        if buffer.len() >= buffer_threshold {
            // Convert buffer to i16 samples and push to source
            if buffer.len() % 2 != 0 {
                extra_byte = Some(buffer.split_off(buffer.len() - 1)[0]);
            }

            let samples: Vec<i16> = buffer
                .chunks_exact(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            source.push_samples(&samples);

            println!("Playing {} bytes of audio data", buffer.len());

            // Start playing the audio
            play_audio(&sink, sample_rate, channels, source.take_buffer());

            // Clear the buffer
            buffer.clear();
        }
    }

    // Play any remaining buffered data
    if !buffer.is_empty() {
        // Prepend the extra byte if present
        if let Some(byte) = extra_byte {
            let mut new_buffer = BytesMut::with_capacity(buffer.len() + 1);
            new_buffer.extend_from_slice(&[byte]);
            new_buffer.extend_from_slice(&buffer);
            buffer = new_buffer;
        }

        let samples: Vec<i16> = buffer
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        source.push_samples(&samples);

        // Play the remaining audio
        play_audio(&sink, sample_rate, channels, source.take_buffer());
    }

    println!("Received all audio data");

    // Ensure all audio is played before exiting
    sink.sleep_until_end();

    Ok(())
}
