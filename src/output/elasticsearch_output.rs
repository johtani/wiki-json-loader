use elasticsearch::Elasticsearch;
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use url::{Url};
use crate::loader::document::Document;

pub trait SearchEngine {
    fn new(config_file: &str) -> Self where Self: Sized;
    fn send(&self, document: &Document)-> Result<(), String>;
}

pub struct ElasticsearchOutput {
    client: Elasticsearch,
    buffer: Vec<Document>,
}


impl SearchEngine for ElasticsearchOutput{
    fn new(_config_file: &str) -> Self{
        // read config
        // load url
        let _url = "";
        // TODO Elastic Cloud?
        let url = Url::parse("https://example.com").unwrap();
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool).disable_proxy().build().unwrap();
        let client = Elasticsearch::new(transport);
        let buffer = vec![];
        // TODO decide buffer size
        ElasticsearchOutput {
          client,
          buffer,
        }
    }

    fn send(&self, _documents: &Document) -> Result<(), String> {
        unimplemented!()
    }

}
