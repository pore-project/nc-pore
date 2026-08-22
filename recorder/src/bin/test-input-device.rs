use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    let target_index = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| {
            eprintln!("Verwendung: test-input-device <index>");
            std::process::exit(2);
        });

    let host = cpal::default_host();
    let mut devices = host.input_devices().unwrap_or_else(|error| {
        eprintln!("Eingabegeräte konnten nicht aufgelistet werden: {error}");
        std::process::exit(1);
    });

    let device = devices.nth(target_index).unwrap_or_else(|| {
        eprintln!("Kein CPAL-Eingabegerät mit Index {target_index}");
        std::process::exit(1);
    });

    let name = device
        .description()
        .map(|description| description.to_string())
        .unwrap_or_else(|_| "<unbekannt>".to_string());

    println!("CPAL Host: {:?}", host.id());
    println!("Test-Eingabegerät [{target_index}]: {name}");

    let config = device
        .supported_input_configs()
        .unwrap_or_else(|error| {
            eprintln!("Konfigurationen konnten nicht gelesen werden: {error}");
            std::process::exit(1);
        })
        .find(|configuration| {
            configuration.channels() == 2
                && configuration.min_sample_rate().0 <= 48_000
                && configuration.max_sample_rate().0 >= 48_000
                && matches!(configuration.sample_format(), cpal::SampleFormat::I32)
        })
        .map(|configuration| configuration.with_sample_rate(cpal::SampleRate(48_000)))
        .unwrap_or_else(|| {
            eprintln!("Keine passende 2-Kanal-I32-Konfiguration bei 48 kHz gefunden.");
            std::process::exit(1);
        });

    println!(
        "Konfiguration: {} Kanal, {} Hz, {:?}, Buffer {:?}",
        config.channels, config.sample_rate.0, config.sample_format(), config.buffer_size
    );

    let samples = Arc::new(Mutex::new(Vec::<i32>::new()));
    let captured = Arc::clone(&samples);
    let stream = device
        .build_input_stream(
            &config.config(),
            move |data: &[i32], _| {
                if let Ok(mut samples) = captured.lock() {
                    samples.extend_from_slice(data);
                }
            },
            move |error| eprintln!("Input-Stream-Fehler: {error}"),
            None,
        )
        .unwrap_or_else(|error| {
            eprintln!("Input-Stream konnte nicht gestartet werden: {error}");
            std::process::exit(1);
        });

    stream.play().unwrap_or_else(|error| {
        eprintln!("Input-Stream konnte nicht gestartet werden: {error}");
        std::process::exit(1);
    });

    std::thread::sleep(Duration::from_secs(3));
    drop(stream);

    let samples = samples.lock().unwrap();
    let min = samples.iter().copied().min().unwrap_or(0);
    let max = samples.iter().copied().max().unwrap_or(0);
    let nonzero = samples.iter().filter(|sample| **sample != 0).count();
    let rms = if samples.is_empty() {
        0.0
    } else {
        let mean_square = samples
            .iter()
            .map(|sample| {
                let normalized = *sample as f64 / i32::MAX as f64;
                normalized * normalized
            })
            .sum::<f64>()
            / samples.len() as f64;
        mean_square.sqrt()
    };

    println!("Empfangene Samples: {}", samples.len());
    println!("Minimum: {min}");
    println!("Maximum: {max}");
    println!("Nicht-Null-Samples: {} ({:.2} %)", nonzero, if samples.is_empty() { 0.0 } else { nonzero as f64 * 100.0 / samples.len() as f64 });
    println!("RMS: {rms:.6}");
}
