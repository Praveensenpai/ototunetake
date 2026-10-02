use crate::error::{AppError, Result};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub enum AudioReader {
    File(BufReader<File>),
    Memory(Cursor<Vec<u8>>),
}

impl Read for AudioReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::File(r) => r.read(buf),
            Self::Memory(c) => c.read(buf),
        }
    }
}

impl Seek for AudioReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::File(r) => r.seek(pos),
            Self::Memory(c) => c.seek(pos),
        }
    }
}

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    playback_start: Option<Instant>,
    paused_duration: Duration,
    pause_start: Option<Instant>,
    is_paused: Arc<AtomicBool>,
    volume: f32,
    cached_wav: Option<(PathBuf, Vec<u8>)>,
}

impl AudioEngine {
    pub fn try_new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AppError::Audio(format!("Failed to initialize audio output: {e}")))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: None,
            playback_start: None,
            paused_duration: Duration::ZERO,
            pause_start: None,
            is_paused: Arc::new(AtomicBool::new(false)),
            volume: 1.0,
            cached_wav: None,
        })
    }

    pub fn play_file(&mut self, path: &Path) -> Result<()> {
        self.stop();

        let source = self.create_source(path)?;
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AppError::Audio(format!("Failed to create audio sink: {e}")))?;

        sink.set_volume(self.volume);
        sink.append(source);

        self.sink = Some(sink);
        self.playback_start = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.pause_start = None;
        self.is_paused.store(false, Ordering::SeqCst);

        Ok(())
    }

    fn create_source(&mut self, path: &Path) -> Result<Decoder<AudioReader>> {
        if let Some((cached_path, wav_bytes)) = &self.cached_wav {
            if cached_path == path {
                let reader = AudioReader::Memory(Cursor::new(wav_bytes.clone()));
                if let Ok(source) = Decoder::new(reader) {
                    return Ok(source);
                }
            }
        }

        let file = File::open(path)?;
        let direct_reader = AudioReader::File(BufReader::new(file));
        if let Ok(source) = Decoder::new(direct_reader) {
            return Ok(source);
        }

        let transcoded_bytes = Self::transcode_with_ffmpeg(path)?;
        self.cached_wav = Some((path.to_path_buf(), transcoded_bytes.clone()));
        let memory_reader = AudioReader::Memory(Cursor::new(transcoded_bytes));

        Decoder::new(memory_reader).map_err(|e| {
            AppError::Audio(format!(
                "Failed to decode audio file {}: {e}",
                path.display()
            ))
        })
    }

    fn transcode_with_ffmpeg(path: &Path) -> Result<Vec<u8>> {
        let output = Command::new("ffmpeg")
            .arg("-v")
            .arg("error")
            .arg("-i")
            .arg(path)
            .arg("-f")
            .arg("flac")
            .arg("-")
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Audio(format!(
                "ffmpeg failed to decode {}: {err}",
                path.display()
            )));
        }

        Ok(output.stdout)
    }

    pub fn toggle_pause(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                if let Some(start) = self.pause_start.take() {
                    self.paused_duration += start.elapsed();
                }
                self.is_paused.store(false, Ordering::SeqCst);
            } else {
                sink.pause();
                self.pause_start = Some(Instant::now());
                self.is_paused.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn is_done(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.empty()
        } else {
            true
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.playback_start = None;
        self.paused_duration = Duration::ZERO;
        self.pause_start = None;
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn elapsed(&self) -> Duration {
        let Some(start) = self.playback_start else {
            return Duration::ZERO;
        };

        if self.is_paused() {
            if let Some(p_start) = self.pause_start {
                let total = p_start.duration_since(start);
                total.saturating_sub(self.paused_duration)
            } else {
                Duration::ZERO
            }
        } else {
            let total = start.elapsed();
            total.saturating_sub(self.paused_duration)
        }
    }
}
