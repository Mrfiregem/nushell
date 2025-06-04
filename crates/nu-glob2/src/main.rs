use nu_glob2::*;
use std::num::NonZeroI32;
use std::ops::Deref;
use std::{io::Write, path::PathBuf};
use nu_protocol::ShellError;

fn die(exit_code: i32) -> nu_protocol::ShellError {
    const USAGE: &str = "Usage: glob_experiment <pattern> <parse|compile|matches|glob> [path]";
    if exit_code == 0 {
        println!("{}", USAGE);
    } else {
        eprintln!("{}", USAGE);
    }
    nu_protocol::ShellError::NonZeroExitCode {
        exit_code: NonZeroI32::new(exit_code).expect("unreachable"),
        span: nu_protocol::Span::unknown(),
    }
}

fn main() {
    match run_cmd() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(err.exit_code().unwrap_or(1));
        }
    }
}

fn run_cmd() -> Result<(), ShellError> {
    let conv_err =
        |e| nu_protocol::shell_error::io::IoError::new_internal(e, "", nu_protocol::location!());
    let mut args = std::env::args_os().skip(1);

    let pattern_string = match args.next() {
        Some(pat) => pat,
        None => return Err(die(1)),
    };
    let glob = Glob::new(&*pattern_string.to_string_lossy(), None);

    match args.next().map(|s| s.into_encoded_bytes()).as_deref() {
        Some(b"parse") => {
            println!("{:#?}", glob.get_pattern().deref());
        }
        Some(b"compile") => {
            let program = glob.compile()?;
            print!("{}", program);
        }
        Some(b"matches") => {
            let path: PathBuf = args.next().ok_or_else(|| die(1))?.into();
            let program = glob.compile()?;
            if program.matches(&path) {
                println!("{} does match the path \"{}\"", program, path.display());
            } else {
                println!("{} does not match the path \"{}\"", program, path.display());
            }
        }
        Some(b"glob") => {
            let program = glob.compile()?;
            let current_dir = std::env::current_dir().map_err(conv_err)?;
            let mut stdout = std::io::stdout();
            let mut failed = false;
            for result in globber::glob(current_dir, program) {
                match result {
                    Ok(path) => {
                        stdout
                            .write_all(path.as_os_str().as_encoded_bytes())
                            .map_err(conv_err)?;
                        stdout.write_all(b"\n").map_err(conv_err)?;
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        failed = true;
                    }
                }
            }
            if failed {
                std::process::exit(1);
            }
        }
        _ => return Err(die(1)),
    }

    Ok(())
}
