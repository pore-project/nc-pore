use nc_pore_infrastructure::FileProductionSessionRepository;
use pore_runtime::production::{
    OPERATION_PRODUCTION_COMMAND, ProductionCommandRequest, ProductionCommandResponse,
    handle_production_command,
};
use pore_runtime::{
    RecordingCommandRequest, RecordingCommandResponse, SubmitFinalizedArtifactRequest,
    handle_recording_command, handle_submit, write_response,
};
use std::io::{self, BufReader, BufWriter, Read, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());

    let frame = match read_frame(&mut input) {
        Ok(frame) => frame,
        Err(error) => {
            eprintln!("runtime protocol error: {error}");
            std::process::exit(3);
        }
    };

    let operation = match serde_json::from_slice::<serde_json::Value>(&frame) {
        Ok(value) => value
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        Err(error) => {
            eprintln!("runtime request JSON error: {error}");
            std::process::exit(3);
        }
    };

    match operation.as_deref() {
        Some(OPERATION_PRODUCTION_COMMAND) => {
            let request: ProductionCommandRequest = match serde_json::from_slice(&frame) {
                Ok(request) => request,
                Err(error) => {
                    eprintln!("production command JSON error: {error}");
                    std::process::exit(3);
                }
            };
            let root = std::env::var("PORE_SESSION_STORE")
                .unwrap_or_else(|_| "./var/sessions".to_owned());
            let mut repository = match FileProductionSessionRepository::new(root) {
                Ok(repository) => repository,
                Err(error) => {
                    eprintln!("runtime repository error: {error}");
                    std::process::exit(5);
                }
            };
            let response: ProductionCommandResponse =
                handle_production_command(&request, &mut repository);
            if let Err(error) = write_json_frame(&mut output, &response) {
                eprintln!("runtime response error: {error}");
                std::process::exit(4);
            }
        }
        Some(pore_runtime::OPERATION_RECORDING_COMMAND) => {
            let request: RecordingCommandRequest = match serde_json::from_slice(&frame) {
                Ok(request) => request,
                Err(error) => {
                    eprintln!("recording command JSON error: {error}");
                    std::process::exit(3);
                }
            };
            let root = std::env::var("PORE_SESSION_STORE")
                .unwrap_or_else(|_| "./var/sessions".to_owned());
            let mut repository = match FileProductionSessionRepository::new(root) {
                Ok(repository) => repository,
                Err(error) => {
                    eprintln!("runtime repository error: {error}");
                    std::process::exit(5);
                }
            };
            let response: RecordingCommandResponse =
                handle_recording_command(&request, &mut repository);
            if let Err(error) = write_json_frame(&mut output, &response) {
                eprintln!("runtime response error: {error}");
                std::process::exit(4);
            }
        }
        Some(pore_runtime::OPERATION_SUBMIT_FINALIZED_ARTIFACT) => {
            let request: SubmitFinalizedArtifactRequest = match serde_json::from_slice(&frame) {
                Ok(request) => request,
                Err(error) => {
                    eprintln!("artifact request JSON error: {error}");
                    std::process::exit(3);
                }
            };
            let payload_len = match usize::try_from(request.payload_length) {
                Ok(length) => length,
                Err(_) => {
                    eprintln!("artifact payload length is too large");
                    std::process::exit(3);
                }
            };
            let mut payload = vec![0_u8; payload_len];
            if let Err(error) = input.read_exact(&mut payload) {
                eprintln!("artifact payload read error: {error}");
                std::process::exit(3);
            }
            let response = handle_submit(&request, &payload);
            if let Err(error) = write_response(&mut output, &response) {
                eprintln!("runtime response error: {error:?}");
                std::process::exit(4);
            }
        }
        _ => {
            eprintln!("unsupported runtime operation");
            std::process::exit(3);
        }
    }
}

fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, io::Error> {
    let mut length = [0_u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid JSON frame length",
        ));
    }
    let mut frame = vec![0_u8; length];
    reader.read_exact(&mut frame)?;
    Ok(frame)
}

fn write_json_frame<W: Write, T: serde::Serialize>(
    writer: &mut W,
    value: &T,
) -> Result<(), io::Error> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let length = u32::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "response too large"))?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}
