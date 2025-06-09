use nu_protocol::{ShellError, Span, Value};
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
    Symlink
}

#[derive(Default, Debug)]
pub struct WalkOptions {
    max_depth: Option<usize>,
    no_dirs: bool,
    no_files: bool,
    no_symlinks: bool,
    follow_symlinks: bool,
    exclusions: Vec<Glob>
}

#[derive(Debug)]
pub struct Glob {
    pattern_string: String,
    span: Option<Span>,
    pattern: Arc<parser::Pattern>,
}

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

    pub fn exclude_patterns(mut self, patterns: Vec<Glob>) -> Self {
        self.exclusions = patterns;
        self
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

impl nu_protocol::FromValue for Glob {
    fn from_value(value: Value) -> Result<Self, ShellError> {
        if let Value::Glob {
            val, internal_span, ..
        }
        | Value::String { val, internal_span } = value
        {
            Ok(Glob::new(val, Some(internal_span)))
        } else {
            Err(ShellError::InvalidGlobPattern {
                msg: format!("Expected glob/string; got {}", value.get_type()),
                span: value.span(),
            })
        }
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

    pub fn absolute_prefix(&self) -> Option<std::path::PathBuf> {
        self.program.absolute_prefix.clone()
    }

    /// Check if a given path would match the glob pattern
    pub fn matches(&self, path: &std::path::Path) -> bool {
        matcher::path_matches(path, &self.program) == matcher::MatchResult::none()
    }

    pub fn walk(
        &self,
    ) -> impl Iterator<Item = Result<std::path::PathBuf, error::GlobError>> + Send {
        let relative_to = self
            .absolute_prefix()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        globber::glob(relative_to, self.inner_program())
    }
}

impl std::fmt::Display for CompiledGlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_pattern_string())
    }
}
