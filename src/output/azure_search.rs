use crate::loader::document::Document;
use std::fs::File;
use crate::output::elasticsearch_output::SearchEngine;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct AzureSearchConfig {

}

pub struct AzureSearchOutput {

}


fn load_config(config_file: &str) -> AzureSearchConfig {
    let f = File::open(config_file)
        .expect(format!("config file is not found. {}", config_file).as_str());
    let config: EsConfig = serde_yaml::from_reader(f).expect(format!("Parse Error").as_str());
    return config;
}

impl SearchEngine for AzureSearchOutput {
    fn new(config_file: &str) -> Self where
        Self: Sized {
        unimplemented!()
    }

    fn add_document(&mut self, document: Document) {
        unimplemented!()
    }

    async fn flush(&mut self) -> Result<(), Box<Error>> {
        unimplemented!()
    }

    async fn close(&mut self) -> Result<(), Box<Error>> {
        unimplemented!()
    }
}