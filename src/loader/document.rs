pub struct Document {
    title: String,
    url: String,
    id: String,
}

impl Document {
    pub fn new(line: &str) -> Self {
        // json parse?
        Document {
            title: "".to_string(),
            url: "".to_string(),
            id: "".to_string()
        }
    }
}