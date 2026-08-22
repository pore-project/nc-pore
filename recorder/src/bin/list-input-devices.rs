use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|device| device.description().ok())
        .map(|description| description.to_string());

    println!("CPAL Host: {:?}", host.id());
    println!(
        "CPAL Default Input: {}",
        default_name.as_deref().unwrap_or("<kein Standard-Eingabegerät>")
    );
    println!();
    println!("Verfügbare CPAL-Eingabegeräte:");

    let devices = host.input_devices().unwrap_or_else(|error| {
        eprintln!("Eingabegeräte konnten nicht aufgelistet werden: {error}");
        std::process::exit(1);
    });

    let mut count = 0usize;
    for (index, device) in devices.enumerate() {
        count += 1;
        let name = device
            .description()
            .map(|description| description.to_string())
            .unwrap_or_else(|_| "<unbekannt>".to_string());
        let marker = if default_name.as_deref() == Some(name.as_str()) {
            " [DEFAULT]"
        } else {
            ""
        };

        println!("  {index}: {name}{marker}");

        match device.supported_input_configs() {
            Ok(configurations) => {
                for configuration in configurations {
                    println!(
                        "      channels={} rate={}..{} format={:?} buffer={:?}",
                        configuration.channels(),
                        configuration.min_sample_rate(),
                        configuration.max_sample_rate(),
                        configuration.sample_format(),
                        configuration.buffer_size(),
                    );
                }
            }
            Err(error) => println!("      Konfigurationen: <Fehler: {error}>"),
        }
    }

    println!();
    println!("Anzahl CPAL-Eingabegeräte: {count}");
}
