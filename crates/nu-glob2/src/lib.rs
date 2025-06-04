use nu_protocol::{ShellError, Span, Value};
use std::fmt::Display;

mod compiler;
mod globber;
mod matcher;
mod parser;

pub(crate) type GlobResult<T> = Result<T, ShellError>;

#[derive(Debug)]
pub struct Glob {
    pattern_string: String,
    span: Option<Span>,
    pattern: std::sync::Arc<parser::Pattern>,
}

pub struct CompiledGlob {
    inner_glob: Glob,
    program: compiler::Program,
}

impl Glob {
    /// Create a new Glob from a string
    pub fn new(pattern_string: impl Into<String>, span: Option<Span>) -> Self {
        let string = pattern_string.into();
        Glob {
            pattern: std::sync::Arc::new(parser::parse(&string)),
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
    pub fn get_pattern(&self) -> std::sync::Arc<parser::Pattern> {
        self.pattern.clone()
    }

    /// Compile the glob to use for matching
    pub fn compile(self) -> GlobResult<CompiledGlob> {
        Ok(CompiledGlob {
            program: compiler::compile(&self.get_pattern())?,
            inner_glob: self,
        })
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
                msg: format!("Expected glob/string; got {}", value.get_type()),
                span: value.span(),
            })
        }
    }
}
