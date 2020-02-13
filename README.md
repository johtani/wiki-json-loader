# Wikipedia json data Loader

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT))

Load JSON files that made by [Wikiextractor](http://medialab.di.unipi.it/wiki/Wikipedia_Extractor) to search engines.
Now, the tool suppot only JSON files that are created wikiextractor default settings.


## Support search engines

* Elasticsearch >=7.x

## Usage

Show help with the following command::
```
$ ./bin/wiki-json-loader -h
```

#### Options

```
$ ./wiki-json-loader -c <SEARCH_ENGINE_CONFIG> -s <SEARCH_ENGINE_TYPE> <INPUT_DIR>
```

