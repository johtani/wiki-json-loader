use crate::loader::document::Document;
use crate::output::elasticsearch_output::{ElasticsearchOutput, SearchEngine};
use clap::arg_enum;
use flamer::flame;
use glob::glob;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use log::info;

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
#[flame]
fn load_file(filepath: &str, search_engine: &mut Box<dyn SearchEngine>) -> Result<String, String> {
    info!("Reading {}", filepath);
    for line in BufReader::new(File::open(filepath).unwrap()).lines() {
        let d = parse_document(line.unwrap().as_str());
        search_engine.add_document(d);
    }
    search_engine.close();
    Ok(format!("Finish: {}", filepath).to_string())
}

fn parse_document(_line: &str) -> Document {
    Document::new(_line)
}

#[flame]
pub fn load(
    input_dir: &str,
    config_file: &str,
    search_engine: &SearchEngineType,
) -> Result<(), String> {
    let path = Path::new(input_dir).join(Path::new("**/sample_*.json"));
    // read files from input_dir
    let files: Vec<_> = glob(path.to_str().unwrap())
        .unwrap()
        .filter_map(|x| x.ok())
        .collect();
    files
        .par_iter()
        .map(|filepath| {
            // read JSONs from file
            // create output instance search_engine_type
            let mut search_engine = create_search_engine(config_file, &search_engine);
            load_file(filepath.to_str().unwrap(), &mut search_engine)
        })
        .filter_map(|x| x.ok())
        .collect::<String>();
    Ok(())
}
