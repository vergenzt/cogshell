use std::io::{self, BufRead, Write};
use std::ops::Range;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::fs;

use anyhow::{bail, Context, Result};
use md5::{Digest, Md5};

mod utils;
mod config;
mod source;
mod parse;
// mod execute;

// use config::{Config, LINE_SEPS};
// use parse::ParsedBlock;
