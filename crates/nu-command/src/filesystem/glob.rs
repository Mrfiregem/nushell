use nu_engine::command_prelude::*;
use nu_protocol::{FromValue, ListStream};

use nu_glob2::{Glob as NuGlob, WalkOptions};

#[derive(Clone)]
pub struct Glob;

impl Command for Glob {
    fn name(&self) -> &str {
        "glob"
    }

    fn signature(&self) -> Signature {
        Signature::build("glob")
            .input_output_types(vec![(Type::Nothing, Type::List(Box::new(Type::String)))])
            .required("glob", SyntaxShape::OneOf(vec![SyntaxShape::String, SyntaxShape::GlobPattern]), "The glob expression.")
            .named(
                "depth",
                SyntaxShape::Int,
                "directory depth to search",
                Some('d'),
            )
            .switch(
                "no-dir",
                "Whether to filter out directories from the returned paths",
                Some('D'),
            )
            .switch(
                "no-file",
                "Whether to filter out files from the returned paths",
                Some('F'),
            )
            .switch(
                "no-symlink",
                "Whether to filter out symlinks from the returned paths",
                Some('S'),
            )
            .switch(
                "follow-symlinks",
                "Whether to follow symbolic links to their targets",
                Some('l'),
            )
            .named(
                "exclude",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Patterns to exclude from the search: `glob` will not walk the inside of directories matching the excluded patterns.",
                Some('e'),
            )
            .category(Category::FileSystem)
    }

    fn description(&self) -> &str {
        "Creates a list of files and/or folders based on the glob pattern provided."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["pattern", "files", "folders", "list", "ls"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Search for *.rs files",
                example: "glob *.rs",
                result: None,
            },
            Example {
                description: "Search for *.rs and *.toml files recursively up to 2 folders deep",
                example: "glob **/*.{rs,toml} --depth 2",
                result: None,
            },
            Example {
                description: "Search for files and folders that begin with uppercase C or lowercase c",
                example: r#"glob "[Cc]*""#,
                result: None,
            },
            Example {
                description: "Search for files and folders like abc or xyz substituting a character for ?",
                example: r#"glob "{a?c,x?z}""#,
                result: None,
            },
            Example {
                description: "A case-insensitive search for files and folders that begin with c",
                example: r#"glob "(?i)c*""#,
                result: None,
            },
            Example {
                description: "Search for files for folders that do not begin with c, C, b, M, or s",
                example: r#"glob "[!cCbMs]*""#,
                result: None,
            },
            Example {
                description: "Search for files or folders with 3 a's in a row in the name",
                example: "glob <a*:3>",
                result: None,
            },
            Example {
                description: "Search for files or folders with only a, b, c, or d in the file name between 1 and 10 times",
                example: "glob <[a-d]:1,10>",
                result: None,
            },
            Example {
                description: "Search for folders that begin with an uppercase ASCII letter, ignoring files and symlinks",
                example: r#"glob "[A-Z]*" --no-file --no-symlink"#,
                result: None,
            },
            Example {
                description: "Search for files named tsconfig.json that are not in node_modules directories",
                example: r#"glob **/tsconfig.json --exclude [**/node_modules/**]"#,
                result: None,
            },
            Example {
                description: "Search for all files that are not in the target nor .git directories",
                example: r#"glob **/* --exclude [**/target/** **/.git/** */]"#,
                result: None,
            },
            Example {
                description: "Search for files following symbolic links to their targets",
                example: r#"glob "**/*.txt" --follow-symlinks"#,
                result: None,
            },
        ]
    }

    fn extra_description(&self) -> &str {
        r#"For more glob pattern help, please refer to https://docs.rs/crate/wax/latest"#
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        new_glob(engine_state, stack, call)
    }
}

fn compile_exclusions(globs: Vec<Value>) -> Result<Vec<nu_glob2::CompiledGlob>, ShellError> {
    let span = globs
        .first()
        .map(|val| val.span())
        .unwrap_or_else(Span::unknown);

    globs
        .into_iter()
        .map(|val| {
            NuGlob::from_value(val).and_then(move |glob| {
                glob.compile(WalkOptions::default())
                    .map_err(|err| err.into_shell_error(span))
            })
        })
        .collect::<Result<Vec<_>, ShellError>>()
}

fn build_walk_options(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<WalkOptions, ShellError> {
    let exclusion_patterns = match call.get_flag::<Vec<Value>>(engine_state, stack, "exclude")? {
        None => Vec::new(),
        Some(list) => compile_exclusions(list)?,
    };
    eprintln!("list = {:#?}", exclusion_patterns);

    let options = WalkOptions::build()
        .max_depth(call.get_flag(engine_state, stack, "depth")?)
        .exclude_files(call.has_flag(engine_state, stack, "no-file")?)
        .exclude_directories(call.has_flag(engine_state, stack, "no-dir")?)
        .exclude_symlinks(call.has_flag(engine_state, stack, "no-symlink")?)
        .exclude_patterns(exclusion_patterns);
    Ok(options)
}

fn new_glob(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<PipelineData, ShellError> {
    let span = call.head;
    let input_value: Value = call.req(engine_state, stack, 0)?;

    let options = build_walk_options(engine_state, stack, call)?;
    let glob = NuGlob::from_value(input_value)?
        .compile(options)
        .map_err(|e| e.into_shell_error(span))?;

    Ok(PipelineData::from(ListStream::new(
        glob.walk().map(move |result| match result {
            Ok(path) => Value::string(path.to_string_lossy(), span),
            Err(err) => Value::error(err.into_shell_error(span), span),
        }),
        span,
        engine_state.signals().clone(),
    )))
}

#[cfg(windows)]
#[cfg(test)]
mod windows_tests {
    use super::*;

    #[test]
    fn glob_pattern_with_drive_letter() {
        let pattern = "D:/*.mp4".to_string();
        let result = patch_windows_glob_pattern(pattern, Span::test_data()).unwrap();
        assert!(WaxGlob::new(&result).is_ok());

        let pattern = "Z:/**/*.md".to_string();
        let result = patch_windows_glob_pattern(pattern, Span::test_data()).unwrap();
        assert!(WaxGlob::new(&result).is_ok());

        let pattern = "C:/nested/**/escaped/path/<[_a-zA-Z\\-]>.md".to_string();
        let result = patch_windows_glob_pattern(pattern, Span::test_data()).unwrap();
        assert!(dbg!(WaxGlob::new(&result)).is_ok());
    }

    #[test]
    fn glob_pattern_without_drive_letter() {
        let pattern = "/usr/bin/*.sh".to_string();
        let result = patch_windows_glob_pattern(pattern.clone(), Span::test_data()).unwrap();
        assert_eq!(result, pattern);
        assert!(WaxGlob::new(&result).is_ok());

        let pattern = "a".to_string();
        let result = patch_windows_glob_pattern(pattern.clone(), Span::test_data()).unwrap();
        assert_eq!(result, pattern);
        assert!(WaxGlob::new(&result).is_ok());
    }

    #[test]
    fn invalid_path_format() {
        let invalid = "C:lol".to_string();
        let result = patch_windows_glob_pattern(invalid, Span::test_data());
        assert!(result.is_err());
    }

    #[test]
    fn unpatched_patterns() {
        let unpatched = "C:/Users/*.txt".to_string();
        assert!(WaxGlob::new(&unpatched).is_err());

        let patched = patch_windows_glob_pattern(unpatched, Span::test_data()).unwrap();
        assert!(WaxGlob::new(&patched).is_ok());
    }
}
