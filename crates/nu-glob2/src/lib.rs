use nu_protocol::{ShellError, Value};
use std::fmt::Display;

pub mod compiler;
pub mod globber;
pub mod matcher;
pub mod parser;

pub type GlobResult<T> = Result<T, ShellError>;

#[derive(Debug)]
pub struct Glob {
    pattern_string: String,
    span: Option<nu_protocol::Span>,
    pattern: std::sync::Arc<parser::Pattern>,
}

impl Glob {
    /// Create a new Glob from a string
    pub fn new(pattern_string: impl Into<String>, span: Option<nu_protocol::Span>) -> Self {
        let string = pattern_string.into();
        Glob {
            pattern: std::sync::Arc::new(parser::parse(&string)),
            span,
            pattern_string: string,
        }
    }

    /// Get internal span
    pub fn span(&self) -> Option<nu_protocol::Span> {
        self.span
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
                msg: "Expected glob, string".to_string(),
                span: value.span(),
            })
        }
    }
}
