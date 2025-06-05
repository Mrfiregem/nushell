use thiserror::Error;
use miette::Diagnostic;

#[derive(Error, Debug, Diagnostic)]
pub enum GlobError {
    #[error("cannot parse '{input}' as a glob pattern")]
    #[diagnostic(code(nu_glob2::parser::unparsable_input))]
    UnparseableInput {
        input: String,
    },
    #[error("{0} must be <= {}", u16::MAX)]
    #[diagnostic(code(nu_glob2::compiler::counter_overflow))]
    CounterOverflow(u16),
    #[error("encountered rust IO error: {0}")]
    #[diagnostic(code(nu_glob2::globber::io_error))]
    Io(#[from] std::io::Error)
}