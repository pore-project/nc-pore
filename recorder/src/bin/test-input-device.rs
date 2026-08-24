use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    let host = cpal::default_host();
    println!("CPAL Host: {:?}", host.id());

    let devices = host.input_devices().unwrap_or_else(|error| {
        eprintln!("Eingabegeräte konnten nicht aufgelistet werden: {error}");
        std::process::exit(1);
    });

    let mut found = false;

    for device in devices {
        let name = match device.description() {
            Ok(description) => description.to_string(),
            Err(_) => continue,
        };

        if name != "RODECaster Pro, USB Audio" {
            continue;
        }

        found = true;
        println!("\n===== RODECaster Pro =====");
        println!("CPAL-Gerät: {name}");

        let config = match device
            .supported_input_configs()
            .ok()
            .and_then(|mut configs| {
                configs
                    .find(|configuration| {
                        configuration.channels() == 2
                            && configuration.min_sample_rate() <= 48_000
                            && configuration.max_sample_rate() >= 48_000
                            && matches!(configuration.sample_format(), cpal::SampleFormat::I32)
                    })
                    .map(|configuration| configuration.with_sample_rate(48_000))
            }) {
            Some(config) => config,
            None => {
                println!("Keine passende 2-Kanal-I32-Konfiguration bei 48 kHz.");
                continue;
            }
        };

        println!(
            "Konfiguration: {} Kanal, {} Hz, {:?}, Buffer {:?}",
            config.channels(),
            config.sample_rate(),
            config.sample_format(),
            config.buffer_size()
        );

        let samples = Arc::new(Mutex::new(Vec::<i32>::new()));
        let captured = Arc::clone(&samples);

        let stream = match device.build_input_stream(
            config.config(),
            move |data: &[i32], _| {
                if let Ok(mut samples) = captured.lock() {
                    samples.extend_from_slice(data);
                }
            },
            move |error| eprintln!("Input-Stream-Fehler: {error}"),
            None,
        ) {
            Ok(stream) => stream,
            Err(error) => {
                println!("Input-Stream konnte nicht erstellt werden: {error}");
                continue;
            }
        };

        if let Err(error) = stream.play() {
            println!("Input-Stream konnte nicht gestartet werden: {error}");
            continue;
        }

        println!("Aufnahme läuft 3 Sekunden – bitte ins Mikrofon sprechen.");
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
        println!(
            "Nicht-Null-Samples: {} ({:.2} %)",
            nonzero,
            if samples.is_empty() {
                0.0
            } else {
                nonzero as f64 * 100.0 / samples.len() as f64
            }
        );
        println!("RMS: {rms:.6}");

        if nonzero > 0 && rms > 0.0 {
            println!("ERGEBNIS: ECHTES AUDIO ERKANNT");
            return;
        }

        println!("ERGEBNIS: NUR NULL/KEIN SIGNAL");
    }

    if !found {
        eprintln!("Kein RODECaster Pro, USB Audio in CPAL gefunden.");
    } else {
        eprintln!("Kein verwendbarer RODECaster-CPAL-Endpunkt gefunden.");
    }
    std::process::exit(1);
}
