use nu_glob2::*;
use std::ops::Deref;
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

fn run_cmd() -> Result<(), Box<str>> {
    let conv_err = |e: Box<dyn std::error::Error>| -> Box<str> { format!("{e}").into() };
    let mut args = std::env::args_os().skip(1);

    let pattern_string = match args.next() {
        Some(pat) => pat,
        None => return Err("missing pattern".into()),
    };
    let glob = Glob::new(&*pattern_string.to_string_lossy(), None);

    match args.next().map(|s| s.into_encoded_bytes()).as_deref() {
        Some(b"parse") => {
            println!("{:#?}", glob.get_pattern().deref());
        }
        Some(b"compile") => {
            let program = glob.compile().map_err(conv_err)?;
            print!("{}", program);
        }
        Some(b"matches") => {
            let path: PathBuf = args
                .next()
                .ok_or_else(|| "no path given to match on")?
                .into();
            let program = glob.compile().map_err(|e| format!("{e}"))?;
            if program.matches(&path) {
                println!("{} does match the path \"{}\"", program, path.display());
            } else {
                println!("{} does not match the path \"{}\"", program, path.display());
            }
        }
        Some(b"glob") => {
            let program = glob.compile().map_err(|e| format!("{e}").into())?;
            let current_dir = std::env::current_dir().map_err(|e| format!("{e}").into())?;
            let mut stdout = std::io::stdout();
            let mut failed = false;
            for result in walk_glob(current_dir, program.inner_program()) {
                match result {
                    Ok(path) => {
                        stdout
                            .write_all(path.as_os_str().as_encoded_bytes())
                            .map_err(conv_err)
                            .map_err(|e| format!("{e}").into())?;
                        stdout.write_all(b"\n").map_err(|e| format!("{e}").into())?;
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
        _ => return Err("invalid command".into()),
    }

    Ok(())
}
