use miette::Diagnostic;
use nu_protocol::ShellError;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum GlobError {
    #[error("cannot parse '{input}' as a glob pattern")]
    #[diagnostic(code(nu_glob2::parser::unparsable_input))]
    UnparseableInput { input: String },
    #[error("{0} must be <= {}", u16::MAX)]
    #[diagnostic(code(nu_glob2::compiler::counter_overflow))]
    CounterOverflow(u16),
    #[error("encountered rust IO error while working on '{path}': {source}")]
    #[diagnostic(code(nu_glob2::globber::io_error))]
    Io {
        source: std::io::Error,
        path: std::path::PathBuf,
    },
}

impl GlobError {
    pub fn into_shell_error(self, span: nu_protocol::Span) -> ShellError {
        match self {
            GlobError::UnparseableInput { input } => {
                ShellError::InvalidGlobPattern { msg: input, span }
            }
            GlobError::CounterOverflow(_) => ShellError::OperatorOverflow {
                msg: "counter overflow when compiling glob".to_string(),
                span,
                help: None,
            },
            GlobError::Io { source, path } => {
                nu_protocol::shell_error::io::IoError::new(source, span, path).into()
            }
        }
    }
}
