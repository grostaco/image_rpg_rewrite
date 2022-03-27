use nom::Needed;
use thiserror::Error;

macro_rules! impl_from {
    ($from: path, $to: tt, $variant: ident) => {
        impl From<$from> for $to {
            fn from(error: $from) -> Self {
                $to::$variant(error)
            }
        }
    };
    ($from: path, $to: tt, $variant: ident, $( $lt:lifetime ),+ ) => {
        impl<$($lt), +> From<$from> for $to<$($lt), +> {
            fn from(error: $from) -> Self {
                $to::$variant(error)
            }
        }
    };
}

#[derive(Debug, Error)]

pub enum DirectiveError {
    #[error("Unknown directive {0}")]
    UnknownDirective(String),
    #[error("Missing argument {1} at position {0}")]
    MissingArgument(usize, &'static str),
    #[error("{0}")]
    Custom(&'static str),
}

#[derive(Debug)]
pub enum EngineError {
    Directive(DirectiveError),
    Nom(String),
    Incomplete(Needed),
}

impl_from!(DirectiveError, EngineError, Directive);
impl_from!(String, EngineError, Nom);
impl_from!(Needed, EngineError, Incomplete);
