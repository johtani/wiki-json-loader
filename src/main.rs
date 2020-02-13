#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg};
use wiki_json_loader::loader::loader::{load, SearchEngineType};

fn main() {
    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("The directory where JSON files made wikiestractor containing.")
                .value_name("INPUT_DIR")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CONFIG")
                .help("The config yaml file for search engine.")
                .value_name("CONFIG")
                .short("c")
                .long("config")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("SEARCH_ENGINE_TYPE")
                .help("")
                .help("Search engine type what is sent wiki data")
                .short("s")
                .long("search_engine")
                .value_name("SEARCH_ENGINE_TYPE")
                .required(true)
                .takes_value(true),
        );
    let matches = app.get_matches();
    let config_file = matches.value_of("CONFIG").unwrap();
    let input_dir = matches.value_of("INPUT_DIR").unwrap();
    let search_engine_type =
        value_t!(matches, "SEARCH_ENGINE_TYPE", SearchEngineType).unwrap_or_else(|e| e.exit());

    match load(input_dir, config_file, &search_engine_type) {
        Ok(()) => println!("{}", "done"),
        Err(msg) => println!("{}", msg),
    }
}
