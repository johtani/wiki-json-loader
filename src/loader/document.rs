use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
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
    pub fn to_hashmap(&self) -> HashMap<String, Value> {
        let mut data = HashMap::new();
        data.insert(String::from("id"), serde_json::to_value(&self.id).unwrap());
        data.insert(
            String::from("revision_id"),
            serde_json::to_value(&self.revision_id).unwrap(),
        );
        data.insert(
            String::from("title"),
            serde_json::to_value(&self.title).unwrap(),
        );
        data.insert(
            String::from("timestamp"),
            serde_json::to_value(&self.timestamp).unwrap(),
        );
        data.insert(
            String::from("contents"),
            serde_json::to_value(&self.contents).unwrap(),
        );
        data.insert(
            String::from("headings"),
            serde_json::to_value(&self.headings).unwrap(),
        );
        data.insert(
            String::from("categories"),
            serde_json::to_value(&self.categories).unwrap(),
        );
        // FIXME
        //data.insert(String::from("images"), Value::from(self.images));
        //data.insert(String::from("links"), Value::from(self.links));
        return data;
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
