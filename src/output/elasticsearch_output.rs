use crate::loader::document::Document;
use async_trait::async_trait;
use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::{BulkParts, Elasticsearch};
use log::{debug, info, warn};
use serde_json::{json, Value};
use std::fs::File;
use url::Url;

#[async_trait]
pub trait SearchEngine {
    fn new(config_file: &str) -> Self
    where
        Self: Sized;
    fn add_document(&mut self, document: Document);
    fn close(&mut self);
}

#[derive(Debug, Serialize, Deserialize)]
struct EsConfig {
    url: String,
    buffer_size: usize,
    index_name: String,
}

pub struct ElasticsearchOutput {
    client: Elasticsearch,
    buffer: Vec<Document>,
    config: EsConfig,
}

fn load_config(config_file: &str) -> EsConfig {
    let f = File::open(config_file)
        .expect(format!("config file is not found. {}", config_file).as_str());
    let config: EsConfig = serde_yaml::from_reader(f).expect(format!("Parse Error").as_str());
    return config;
}

#[async_trait]
impl SearchEngine for ElasticsearchOutput {
    fn new(_config_file: &str) -> Self {
        // read config
        let config = load_config(_config_file);
        debug!("url: {}", config.url);
        debug!("buffer_size: {}", config.buffer_size);
        // TODO Elastic Cloud?
        let url = Url::parse(config.url.as_str()).unwrap();
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .disable_proxy()
            .build()
            .unwrap();
        let client = Elasticsearch::new(transport);
        let buffer = vec![];
        ElasticsearchOutput {
            client,
            buffer,
            config,
        }
    }

    fn add_document(&mut self, _document: Document) {
        self.buffer.push(_document);
    }

    fn close(&mut self) {
        let chunk_size = if self.buffer.len() <= self.config.buffer_size {
            self.buffer.len()
        } else {
            self.config.buffer_size
        };
        let mut _rt = tokio::runtime::Runtime::new().expect("Fail initializing runtime");
        let mut tasks = vec![];
        for chunk in self.buffer.chunks(chunk_size) {
            let task = self.proceed_chunk(chunk);
            tasks.push(task);
        }

        for task in tasks {
            _rt.block_on(task).expect("Error on task...");
        }
        self.buffer.clear();
    }
}

impl ElasticsearchOutput {
    pub async fn proceed_chunk(&self, chunk: &[Document]) -> Result<(), Box<dyn std::error::Error>> {
        let mut body: Vec<JsonBody<_>> = Vec::new();
        let mut doc_id = String::new();
        for d in chunk {
            if doc_id.is_empty() {
                doc_id.push_str(d.id.as_str());
            }
            body.push(json!({"index": {"_id": d.id}}).into());
            body.push(JsonBody::from(serde_json::to_value(d).unwrap()));
        }
        info!("Sending {} documents... {}", chunk.len(), doc_id);
        let bulk_response = self
            .client
            .bulk(BulkParts::Index(self.config.index_name.as_str()))
            .body(body)
            .send()
            .await?;
        if !bulk_response.status_code().is_success() {
            warn!("Bulk request has failed. Status Code is {:?}. First doc id is [{}]", bulk_response.status_code(), doc_id);
            panic!("bulk indexing failed")
        } else {
            info!("response : {}", bulk_response.status_code());
            let response_body = bulk_response.json::<Value>().await?;
            let successful = response_body["errors"].as_bool().unwrap() == false;
            if successful == false {
                warn!("Bulk Request has some errors. {:?}, {}", successful, doc_id);
                let items = response_body["items"].as_array().unwrap();
                for item in items {
                    let error = item["error"].as_object();
                    match error {
                        None => {},
                        Some(obj) => {
                            warn!("error type:[{}], reason:[{}]", obj.get("type").unwrap(), obj.get("reason").unwrap());
                        },
                    }
                }
            }
        }
        info!("Finished bulk request. {}", doc_id);
        Ok(())
    }
}
