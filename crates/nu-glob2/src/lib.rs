use std::borrow::Cow;
use std::fmt::Display;
use nu_protocol;

pub mod compiler;
pub mod globber;
pub mod matcher;
pub mod parser;

#[derive(Debug)]
pub struct Glob {
    pattern_string: String,
    pattern: parser::Pattern,
}

impl Glob {
    /// Create a new Glob from a string
    pub fn new(pattern_string: &str) -> Self {
        Glob {
            pattern_string: String::from(pattern_string),
            pattern: parser::parse(pattern_string),
        }
    }
    /// Return the initial glob pattern string
    pub fn get_pattern_string(&self) -> &str {
        self.pattern_string.as_str()
    }
    
    /// Compile the glob to use for matching
    pub fn compile(&self) -> Result<compiler::Program, nu_protocol::ShellError> {
        compiler::compile(&self.pattern)
    }
}

impl Display for Glob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
