use std::path::PathBuf;
use nu_protocol::{ShellError, Span, Value};
use std::sync::Arc;

mod compiler;
mod globber;
mod matcher;
mod parser;

pub(crate) type GlobResult<T> = Result<T, ShellError>;

#[derive(Debug)]
pub struct Glob {
    pattern_string: String,
    span: Option<Span>,
    pattern: Arc<parser::Pattern>,
}

pub struct CompiledGlob {
    inner_glob: Glob,
    program: Arc<compiler::Program>,
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
    pub fn compile(self) -> GlobResult<CompiledGlob> {
        Ok(CompiledGlob {
            program: Arc::new(compiler::compile(&self.get_pattern())?),
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
        matcher::path_matches(path, &*self.program) == matcher::MatchResult::none()
    }
}

impl std::fmt::Display for CompiledGlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_pattern_string())
    }
}

impl IntoIterator for CompiledGlob {
    type Item = std::path::PathBuf;
    type IntoIter = GlobIterator;
}

pub struct GlobIterator {
    glob: CompiledGlob,
    index: usize,
}

impl Iterator for GlobIterator {
    type Item = PathBuf;
    fn next(&mut self) -> Option<Self::Item> {
        globber::glob(self.glob)
    }
}