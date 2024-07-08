use std::{fmt, ops::Deref};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RedactedString(pub String);

impl fmt::Debug for RedactedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl Deref for RedactedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
