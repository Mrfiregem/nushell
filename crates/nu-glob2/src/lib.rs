use std::ffi::OsString;
use std::fmt::Display;

pub mod compiler;
pub mod globber;
pub mod matcher;
pub mod parser;

pub type GlobResult<T> = Result<T, nu_protocol::ShellError>;

#[derive(Debug)]
pub struct Glob {
    pattern_string: String,
    pattern: std::sync::Arc<parser::Pattern>,
}

impl Glob {
    /// Create a new Glob from a string
    pub fn new(pattern_string: &str) -> Self {
        Glob {
            pattern_string: String::from(pattern_string),
            pattern: std::sync::Arc::new(parser::parse(pattern_string)),
        }
    }
    /// Return the initial glob pattern string
    pub fn get_pattern_string(&self) -> &str {
        self.pattern_string.as_str()
    }

    /// Return the inner glob Pattern
    pub fn get_pattern(&self) -> std::sync::Arc<parser::Pattern> {
        self.pattern.clone()
    }

    /// Compile the glob to use for matching
    pub fn compile(&self) -> GlobResult<compiler::Program> {
        compiler::compile(&self.pattern)
    }
}

impl Display for Glob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<OsString> for Glob {
    type Error = nu_protocol::ShellError;

    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        let pattern_string = value
            .to_str()
            .ok_or_else(|| Self::Error::InvalidGlobPattern {
                msg: "Unable to convert string to Glob".to_string(),
                span: nu_protocol::Span::unknown(),
            })?;
        Ok(Glob::new(pattern_string))
    }
}
