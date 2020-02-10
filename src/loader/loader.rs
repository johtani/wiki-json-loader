#[macro_use]

use crate::output::elasticsearch_output::{SearchEngine, ElasticsearchOutput};
use crate::loader::document::Document;
use std::path::Path;
use glob::glob;
use clap::arg_enum;

arg_enum!{
    pub enum SearchEngineType {
        Elasticsearch
    }
}


fn create_search_engine(config_file: &str, search_engine: &SearchEngineType) -> Box<dyn SearchEngine> {
    return match search_engine {
        SearchEngineType::Elasticsearch => {
            Box::new(ElasticsearchOutput::new(config_file))
        }
    };
}

fn load_file(filepath: &str, search_engine: &Box<dyn SearchEngine>) {
    println!("{}", filepath);
    let line = "";
    let d = parse_document(line);
    search_engine.send(&d);
}


fn parse_document(_line: &str) -> Document {
    Document::new(_line)
}


pub fn load(input_dir: &str, config_file: &str, search_engine: &SearchEngineType) -> Result<(), String> {
    // create output instance search_engine_type
    let search_engine = create_search_engine(config_file, search_engine);
    let path = Path::new(input_dir).join(Path::new("**/wiki_*"));
    // read files from input_dir
    for filepath in glob(path.to_str().unwrap()).unwrap().filter_map(Result::ok) {
        // read JSONs from file
        load_file(filepath.to_str().unwrap(), &search_engine);
    }
    Ok(())
}

