use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Document {
    pub name: String,
}

impl Document {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}
