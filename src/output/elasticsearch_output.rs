use crate::loader::document::Document;
use async_trait::async_trait;
use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::http::StatusCode;
use elasticsearch::indices::{IndicesCreateParts, IndicesExistsParts};
use elasticsearch::{BulkParts, Elasticsearch};
use log::{debug, info, warn};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Error;
use url::Url;

#[async_trait]
pub trait SearchEngine {
    fn new(config_file: &str) -> Self
    where
        Self: Sized;
    fn add_document(&mut self, document: Document);
    fn initialize(&self);
    fn exist_index(&self) -> bool;
    fn close(&mut self);
}

#[derive(Debug, Serialize, Deserialize)]
struct EsConfig {
    url: String,
    buffer_size: usize,
    index_name: String,
    schema_file: String,
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

pub fn load_schema(schema_file: &str) -> Value {
    let f = File::open(schema_file)
        .expect(format!("schema file is not found. {}", schema_file).as_str());
    let schema: Value = serde_json::from_reader(f).expect("schema cannot read...");
    return schema;
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

    fn initialize(&self) {
        if self.exist_index() {
            //no-op if index already exists
            info!(
                "{} index already exists. skip initialization phase.",
                &self.config.index_name
            );
        } else {
            // load schema.json from file
            // -> if not found, panic!
            // create index with schema file
            info!("{} index is creating...", &self.config.index_name);
            let mut _rt = tokio::runtime::Runtime::new().expect("Fail initializing runtime");
            let task = self.call_indices_create();
            _rt.block_on(task).expect("Something wrong...")
        }
    }

    fn exist_index(&self) -> bool {
        let mut _rt = tokio::runtime::Runtime::new().expect("Fail initializing runtime");
        let task = self.call_indices_exists();
        _rt.block_on(task).expect("Something wrong...")
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
    async fn call_indices_create(&self) -> Result<(), Error> {
        let schema_json = load_schema(&self.config.schema_file);
        let response = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(&self.config.index_name))
            .body(schema_json)
            .send()
            .await;
        match response {
            Ok(response) => {
                if !response.status_code().is_success() {
                    warn!(
                        "Create index request has failed. Status Code is {:?}.",
                        response.status_code()
                    );
                    // FIXME if ...
                    panic!("create index failed")
                } else {
                    info!("{} index was created.", &self.config.index_name);
                }
                return Ok(());
            }
            Err(_) => panic!("create index failed"),
        }
    }

    async fn call_indices_exists(&self) -> Result<bool, Error> {
        let indices: [&str; 1] = [&self.config.index_name.as_str()];
        let result = self
            .client
            .indices()
            .exists(IndicesExistsParts::Index(&indices))
            .send()
            .await;
        match result {
            Ok(response) => match response.status_code() {
                StatusCode::NOT_FOUND => return Ok(false),
                StatusCode::OK => return Ok(true),
                _ => {
                    warn!(
                        "Indices exists request has failed. Status Code is {:?}.",
                        response.status_code()
                    );
                    panic!("Indices exists request failed")
                }
            },
            Err(_) => panic!("Indices exists request failed..."),
        }
    }

    pub async fn proceed_chunk(
        &self,
        chunk: &[Document],
    ) -> Result<(), Box<dyn std::error::Error>> {
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
            warn!(
                "Bulk request has failed. Status Code is {:?}. First doc id is [{}]",
                bulk_response.status_code(),
                doc_id
            );
            panic!("bulk indexing failed")
        } else {
            info!("response : {}", bulk_response.status_code());
            let response_body = bulk_response.json::<Value>().await?;
            let successful = response_body["errors"].as_bool().unwrap() == false;
            if successful == false {
                warn!("Bulk Request has some errors. {:?}, {}", successful, doc_id);
                let items = response_body["items"].as_array().unwrap();
                for item in items {
                    if let Some(index_obj) = item["index"].as_object() {
                        if index_obj.contains_key("error") {
                            if let Some(obj) = index_obj["error"].as_object() {
                                warn!(
                                    "error id:[{}], type:[{}], reason:[{}]",
                                    index_obj.get("_id").unwrap(),
                                    obj.get("type").unwrap(),
                                    obj.get("reason").unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }
        info!("Finished bulk request. {}", doc_id);
        Ok(())
    }
}
