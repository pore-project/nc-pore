use pore_runtime::{handle_submit, read_request, write_response};
use std::io::{self, BufReader, BufWriter};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());

    let (request, payload) = match read_request(&mut input) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("runtime protocol error: {error:?}");
            std::process::exit(3);
        }
    };

    let response = handle_submit(&request, &payload);

    if let Err(error) = write_response(&mut output, &response) {
        eprintln!("runtime response error: {error:?}");
        std::process::exit(4);
    }
}
