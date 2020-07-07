use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    revision_id: String,
    title: String,
    timestamp: String,
    contents: Vec<String>,
    headings: Vec<String>,
    categories: Vec<String>,
    pub images: Vec<Image>,
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ImageType {
    Image,
    File,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub target: String,
    pub target_type: ImageType,
    pub text: Text,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    LinkText {
        text: String,
        #[serde(flatten)]
        link: Link,
    },
    Text {
        text: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Link {
    ExternalLink { link_target: String },
    Link { link_target: String },
}

impl Document {
    pub fn new(line: &str) -> Self {
        serde_json::from_str(line).unwrap()
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
