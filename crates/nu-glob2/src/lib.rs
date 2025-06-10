use nu_protocol::{FromValue, ShellError, Span, Value};
use std::sync::Arc;

mod compiler;
mod globber;
mod matcher;
mod parser;

pub mod error;

pub(crate) type GlobResult<T> = Result<T, error::GlobError>;

pub enum FilterType {
    File,
    Directory,
    Symlink,
}

#[derive(Default, Debug, Clone)]
pub struct WalkOptions {
    max_depth: Option<usize>,
    no_dirs: bool,
    no_files: bool,
    no_symlinks: bool,
    follow_symlinks: bool,
    exclusions: Vec<CompiledGlob>,
}

#[derive(Debug, Clone)]
pub struct Glob {
    pattern_string: String,
    span: Option<Span>,
    pattern: Arc<parser::Pattern>,
}

#[derive(Debug, Clone)]
pub struct CompiledGlob {
    inner_glob: Glob,
    walk_options: WalkOptions,
    program: Arc<compiler::Program>,
}

impl WalkOptions {
    pub fn build() -> Self {
        Self::default()
    }

    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn exclude_files(mut self, option: bool) -> Self {
        self.no_files = option;
        self
    }

    pub fn exclude_directories(mut self, option: bool) -> Self {
        self.no_dirs = option;
        self
    }

    pub fn exclude_symlinks(mut self, option: bool) -> Self {
        self.no_symlinks = option;
        self
    }

    pub fn follow_symlinks(mut self, option: bool) -> Self {
        self.follow_symlinks = option;
        self
    }

    pub fn exclude_patterns(mut self, patterns: impl Into<Vec<CompiledGlob>>) -> Self {
        self.exclusions = patterns.into();
        self
    }

    pub fn would_exclude_type(&self, path: &std::path::Path) -> bool {
        if !self.exclusions.is_empty() {
            return self.exclusions.iter().any(|glob| {
                let matches = glob.matches(path);
                eprintln!("path {} matches? {}", path.display(), matches);
                matches
            })
        }
        if path.is_file() {
            self.no_files
        } else if path.is_dir() {
            self.no_dirs
        } else if path.is_symlink() {
            self.no_symlinks
        } else {
            false
        }
    }
}

impl Glob {
    /// Create a new Glob from a string
    pub fn new(pattern_string: impl Into<String>, span: Option<Span>) -> Self {
        let string = pattern_string.into();
        Glob {
            pattern: Arc::new(parser::parse(&string)),
            span,
            pattern_string: string,
        }
    }

    /// Get internal span
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    /// Return the initial glob pattern string
    pub fn get_pattern_string(&self) -> &str {
        self.pattern_string.as_str()
    }

    /// Return the inner glob Pattern
    pub fn get_pattern(&self) -> Arc<parser::Pattern> {
        self.pattern.clone()
    }

    /// Compile the glob to use for matching
    pub fn compile(self, walk_options: WalkOptions) -> GlobResult<CompiledGlob> {
        Ok(CompiledGlob {
            program: Arc::new(compiler::compile(&self.get_pattern())?),
            walk_options,
            inner_glob: self,
        })
    }
}

impl std::fmt::Display for Glob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_pattern_string())
    }
}

impl CompiledGlob {
    /// Convert a CompiledGlob object back into a Glob
    pub fn into_glob(self) -> Glob {
        self.inner_glob
    }

    pub fn inner_program(&self) -> Arc<compiler::Program> {
        self.program.clone()
    }

    /// Get the initial glob pattern used to create the Glob
    pub fn get_pattern_string(&self) -> &str {
        self.inner_glob.get_pattern_string()
    }

    /// Return the WalkOption struct used to compile this Glob
    pub fn get_walk_options(&self) -> &WalkOptions {
        &self.walk_options
    }

    pub fn absolute_prefix(&self) -> Option<std::path::PathBuf> {
        self.program.absolute_prefix.clone()
    }

    /// Check if a given path would match the glob pattern
    pub fn matches(&self, path: &std::path::Path) -> bool {
        matcher::path_matches(path, &self.program).valid_as_complete_match
    }

    /// Iterate over the filesystem to return paths matching the glob
    pub fn walk(
        &self,
    ) -> impl Iterator<Item = Result<std::path::PathBuf, error::GlobError>> + Send {
        let walk_options = self.get_walk_options().clone();
        let relative_to = self
            .absolute_prefix()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        globber::glob(relative_to, self.inner_program(), self.get_walk_options()).filter(
            move |res| match res {
                Ok(path) => !walk_options.would_exclude_type(path),
                Err(_) => true,
            },
        )
    }
}

impl std::fmt::Display for CompiledGlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_pattern_string())
    }
}

impl FromValue for Glob {
    fn from_value(value: Value) -> Result<Self, ShellError> {
        match value {
            Value::String { val, internal_span }
            | Value::Glob {
                val, internal_span, ..
            } => Ok(Glob::new(
                nu_path::expand_tilde(val).to_string_lossy(),
                Some(internal_span),
            )),
            _ => Err(ShellError::IncorrectValue {
                msg: "Incorrect glob pattern supplied to glob. Please use string only.".to_string(),
                val_span: value.span(),
                call_span: value.span(),
            }),
        }
    }
}
