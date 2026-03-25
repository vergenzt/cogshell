use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use anyhow::Result;

use num_traits::FromPrimitive;
use regex::{Regex, escape};

use crate::config::{Config, MarkerKind};
use crate::source::{SourceLocation, SourceMarker, SourceSpan};
use crate::utils::common_prefix_of_chars;

mod file;
mod block;
mod error;
