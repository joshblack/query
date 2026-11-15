use std::process::ExitCode;

fn main() -> ExitCode {
    match query_cli::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            ExitCode::FAILURE
        }
    }
}
