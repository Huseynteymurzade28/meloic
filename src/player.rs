use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub fn play_file(path: &PathBuf) {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle).unwrap();

    let file = File::open(path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    println!("▶️ Çalıyor: {}", path.display());
    sink.append(source);
    sink.sleep_until_end();
}
