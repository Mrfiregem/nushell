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

#[derive(Debug, Clone)]
pub struct Glob {
    pattern_string: String,
    pattern: Arc<parser::Pattern>,
}

#[derive(Debug, Clone)]
pub struct CompiledGlob {
    pattern_string: String,
    program: Arc<compiler::Program>,
}

impl Glob {
    /// Create a new Glob from a string
    pub fn new(pattern_string: impl Into<String>) -> Self {
        let string = pattern_string.into();
        Glob {
            pattern: Arc::new(parser::parse(&string)),
            pattern_string: string,
        }
    }

    /// Return the initial glob pattern string
    pub fn get_pattern_string(&self) -> &str {
        self.pattern_string.as_str()
    }

    /// Return the inner glob Pattern
    pub fn get_pattern(&self) -> &parser::Pattern {
        self.pattern.as_ref()
    }

    fn into_pattern(self) -> Arc<parser::Pattern> {
        self.pattern
    }

    /// Compile the glob to use for matching
    pub fn compile(self) -> GlobResult<CompiledGlob> {
        Ok(CompiledGlob {
            pattern_string: self.get_pattern_string().to_string(),
            program: Arc::new(compiler::compile(&self.into_pattern())?),
        })
    }
}

impl std::fmt::Display for Glob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_pattern_string())
    }
}

impl CompiledGlob {
    pub fn get_program(&self) -> &compiler::Program {
        self.program.as_ref()
    }

    fn into_program(self) -> Arc<compiler::Program> {
        self.program
    }

    /// Get the initial glob pattern used to create the Glob
    pub fn get_pattern_string(&self) -> &str {
        self.pattern_string.as_str()
    }

    fn absolute_prefix(&self) -> Option<&std::path::Path> {
        match self.get_program().absolute_prefix {
            Some(ref p) => Some(p.as_path()),
            None => None,
        }
    }

    pub fn get_prefix(&self) -> &std::path::Path {
        self.absolute_prefix()
            .unwrap_or_else(|| std::path::Path::new(""))
    }

    /// Check if a given path would match the glob pattern
    pub fn matches(&self, path: &std::path::Path) -> bool {
        matcher::path_matches(path, &self.program).valid_as_complete_match
    }

    /// Iterate over the filesystem to return paths matching the glob
    pub fn walk(self) -> impl Iterator<Item = Result<std::path::PathBuf, error::GlobError>> + Send {
        let relative_to = self.get_prefix().to_path_buf();
        globber::glob(relative_to, self.into_program())
    }

    pub fn walk_and_filter(
        self,
        no_files: bool,
        no_dirs: bool,
        no_symlinks: bool,
    ) -> impl Iterator<Item = Result<std::path::PathBuf, error::GlobError>> + Send {
        self.walk().filter(move |res| match res {
            Ok(path) => {
                !((no_files && path.is_file())
                    || (no_dirs && path.is_dir())
                    || (no_symlinks && !path.is_symlink()))
            }
            Err(_) => true,
        })
    }
}

impl std::fmt::Display for CompiledGlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_pattern_string())
    }
}
