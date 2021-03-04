use crate::statics::CONFIG;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    key: String,
}

impl Key {
    pub fn new() -> Self {
        Self {
            key: format!("{:l$}", Uuid::new_v4(), l = CONFIG.length),
        }
    }
}
