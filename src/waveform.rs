use rodio::Decoder;
use rodio::source::Source;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

pub const WAVEFORM_RESOLUTION: usize = 240;

#[derive(Debug, Clone)]
pub enum WaveformState {
    Loading,
    Ready(Vec<f32>),
    Failed,
}

pub struct WaveformService {
    tx: Sender<PathBuf>,
    rx: Receiver<(PathBuf, Option<Vec<f32>>)>,
    cache: HashMap<PathBuf, WaveformState>,
}

impl WaveformService {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<PathBuf>();
        let (res_tx, res_rx) = channel::<(PathBuf, Option<Vec<f32>>)>();

        thread::spawn(move || {
            while let Ok(path) = req_rx.recv() {
                let samples = compute_waveform(&path);
                let _ = res_tx.send((path, samples));
            }
        });

        Self {
            tx: req_tx,
            rx: res_rx,
            cache: HashMap::new(),
        }
    }

    pub fn request(&mut self, path: &Path) {
        if self.cache.contains_key(path) {
            return;
        }
        self.cache
            .insert(path.to_path_buf(), WaveformState::Loading);
        let _ = self.tx.send(path.to_path_buf());
    }

    pub fn poll(&mut self) {
        while let Ok((path, samples)) = self.rx.try_recv() {
            let state = match samples {
                Some(s) if !s.is_empty() => WaveformState::Ready(s),
                _ => WaveformState::Failed,
            };
            self.cache.insert(path, state);
        }
    }

    pub fn get(&self, path: &Path) -> Option<&WaveformState> {
        self.cache.get(path)
    }
}

fn compute_waveform(path: &Path) -> Option<Vec<f32>> {
    let file = File::open(path).ok()?;
    let decoder = Decoder::new(BufReader::new(file)).ok()?;

    let channels = decoder.channels() as usize;
    let sample_rate = decoder.sample_rate() as usize;
    let total_duration = decoder.total_duration();

    let estimated_samples = total_duration
        .map(|d| d.as_secs_f64() * sample_rate as f64 * channels as f64)
        .unwrap_or(0.0) as usize;

    let mut buckets = vec![0.0_f32; WAVEFORM_RESOLUTION];

    if estimated_samples > 0 {
        let bucket_size = (estimated_samples / WAVEFORM_RESOLUTION).max(1);
        for (i, sample) in decoder.convert_samples::<f32>().enumerate() {
            let bucket = (i / bucket_size).min(WAVEFORM_RESOLUTION - 1);
            let abs = sample.abs();
            if abs > buckets[bucket] {
                buckets[bucket] = abs;
            }
        }
    } else {
        let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();
        if samples.is_empty() {
            return None;
        }
        let bucket_size = (samples.len() / WAVEFORM_RESOLUTION).max(1);
        for (i, s) in samples.iter().enumerate() {
            let bucket = (i / bucket_size).min(WAVEFORM_RESOLUTION - 1);
            buckets[bucket] = buckets[bucket].max(s.abs());
        }
    }

    Some(buckets)
}
