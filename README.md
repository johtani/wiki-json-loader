# Wikipedia json data Loader

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Load JSON files that made by [wiki-extractor-rs](https://github.com/johtani/wiki-extractor-rs) to search engines.
Now, the tool support only JSON files that are created wiki-extractor-rs default settings.

## Requirements

* Rust nightly : elasticsearch-rs support only nightly rustc

## Support search engines

* Elasticsearch >=7.x
* Azure Cognitive Search

## Usage

Sample schema and setting yaml in sample directory.

### Prepare

For Elasticsearch, sample index settings/mappings in [./sample/elasticsearch_wiki_extractor_rs](sample/elasticsearch) directory.

### Load command
Show help with the following command::
```
$ ./bin/wiki-json-loader -h
```

#### Options

```
$ ./wiki-json-loader -c <SEARCH_ENGINE_CONFIG> -s <SEARCH_ENGINE_TYPE> <INPUT_DIR>
```

