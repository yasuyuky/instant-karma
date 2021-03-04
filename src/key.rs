use crate::statics::CONFIG;
use std::fmt;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    key: String,
}

impl Key {
    pub fn new() -> Self {
        let s = &Uuid::new_v4().to_string()[0..CONFIG.length];
        Self { key: s.to_owned() }
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self { Self { key: s.to_owned() } }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.key) }
}
