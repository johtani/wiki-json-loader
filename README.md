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

For Elasticsearch, sample index settings/mappings in [./sample/elasticsearch](sample/elasticsearch) directory.
For Azure Cognitive Search, sample index settings/mappings in [./sample/azure_cognitive_search](sample/azure_cognitive_search) directory.

The command will create an index with with schema json if the index doesn't exist.

### Load command
Show help with the following command::

```
$ ./bin/wiki-json-loader -h
```

#### Options

```
$ ./wiki-json-loader -c <SEARCH_ENGINE_CONFIG> -s <SEARCH_ENGINE_TYPE> <INPUT_DIR>
```

