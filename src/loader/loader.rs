use crate::loader::document::Document;
use crate::output::elasticsearch_output::{ElasticsearchOutput, SearchEngine};
use clap::arg_enum;
use glob::glob;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

arg_enum! {
    pub enum SearchEngineType {
        Elasticsearch
    }
}

fn create_search_engine(
    config_file: &str,
    search_engine: &SearchEngineType,
) -> Box<dyn SearchEngine> {
    return match search_engine {
        SearchEngineType::Elasticsearch => Box::new(ElasticsearchOutput::new(config_file)),
    };
}

fn load_file(filepath: &str, search_engine: &mut Box<dyn SearchEngine>) {
    println!("Reading {}", filepath);
    let mut rt = tokio::runtime::Runtime::new().expect("Fail initializing runtime");
    for line in BufReader::new(File::open(filepath).unwrap()).lines() {
        let d = parse_document(line.unwrap().as_str());
        search_engine.add_document(d);
        let task = search_engine.flush();
        rt.block_on(task).expect("Error?");
    }
    let task = search_engine.close();
    rt.block_on(task).expect("erro?");
}

fn parse_document(_line: &str) -> Document {
    Document::new(_line)
}

pub fn load(
    input_dir: &str,
    config_file: &str,
    search_engine: &SearchEngineType,
) -> Result<(), String> {
    // create output instance search_engine_type
    let mut search_engine = create_search_engine(config_file, &search_engine);
    let path = Path::new(input_dir).join(Path::new("**/wiki_*"));
    // read files from input_dir
    for filepath in glob(path.to_str().unwrap()).unwrap().filter_map(Result::ok) {
        // read JSONs from file
        load_file(filepath.to_str().unwrap(), &mut search_engine);
    }
    Ok(())
}
