use thiserror::Error;

#[derive(Debug, Error)]

pub enum DirectiveError {
    #[error("Missing argument {1} at position {0}")]
    MissingArgument(usize, &'static str),
    #[error("{0}")]
    Custom(&'static str),
}

pub enum ParseError {
    
}