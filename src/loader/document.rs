use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    text: String,
    title: String,
    url: String,
    pub id: String,
}

impl Document {
    pub fn new(line: &str) -> Self {
        serde_json::from_str(line).unwrap()
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{{\"id\":\"{}\",\"title\":\"{}\",\"url\":\"{}\",\"text\":\"{}\"}}",
            self.id, self.title, self.url, self.text
        )
    }
}
