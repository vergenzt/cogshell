use std::iter::repeat_with;

use crate::deref_field;

#[derive(Debug, Clone)]
pub struct OutputTerminator(String);

impl OutputTerminator {
    pub fn new() -> Self {
        Self(repeat_with(fastrand::alphanumeric).take(20).collect())
    }
}

deref_field! { impl *OutputTerminator = .0: String }
