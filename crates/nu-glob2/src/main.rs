use nu_glob2::*;
use std::{io::Write, path::PathBuf};

fn main() {
    const USAGE: &str = "Usage: glob_experiment <pattern> <parse|compile|matches|glob> [path]";

    match run_cmd() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("error: {}", err);
            eprintln!("\n{}", USAGE);
            std::process::exit(1);
        }
    }
}

fn run_cmd() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);

    let pattern_string = match args.next() {
        Some(pat) => pat,
        None => return Err("missing pattern".into()),
    };
    let glob = Glob::new(pattern_string.to_string_lossy());

    match args.next().map(|s| s.into_encoded_bytes()).as_deref() {
        Some(b"parse") => {
            println!("{:#?}", glob.get_pattern());
        }
        Some(b"compile") => {
            let compiled_glob = glob.compile().map_err(convert_error)?;
            print!("{}", compiled_glob.get_program());
        }
        Some(b"matches") => {
            let path: PathBuf = args.next().ok_or("no path given to match on")?.into();
            let program = glob.compile().map_err(convert_error)?;
            if program.matches(&path) {
                println!("{} does match the path \"{}\"", program, path.display());
            } else {
                println!("{} does not match the path \"{}\"", program, path.display());
            }
        }
        Some(b"glob") => {
            let program = glob.compile().map_err(convert_error)?;
            let mut stdout = std::io::stdout();
            let mut failed = false;
            for result in program.walk() {
                match result {
                    Ok(path) => {
                        stdout
                            .write_all(path.as_os_str().as_encoded_bytes())
                            .map_err(convert_error)?;
                        stdout.write_all(b"\n").map_err(convert_error)?;
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
        Some(cmd) => return Err(format!("invalid command: {}", String::from_utf8_lossy(cmd))),
        None => return Err("no command given".into()),
    }

    Ok(())
}

fn convert_error<E>(error: E) -> String
where
    E: std::error::Error,
{
    error.to_string()
}
