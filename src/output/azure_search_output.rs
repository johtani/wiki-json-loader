use crate::loader::document::Document;
use crate::output::elasticsearch_output::load_schema;
use crate::output::elasticsearch_output::SearchEngine;
use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Error;

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {
    value: Vec<DocResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocResponse {
    key: String,
    status: bool,
    error_message: Option<String>,
    status_code: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct AzureSearchConfig {
    service_name: String,
    buffer_size: usize,
    index_name: String,
    schema_file: String,
    //TODO need secure store
    api_key: String,
    #[serde(default = "Vec::new")]
    drop_fields: Vec<String>,
    #[serde(default = "Vec::new")]
    copy_fields: Vec<String>,
}

pub struct AzureSearchOutput {
    client: Client,
    buffer: Vec<AzureDocument>,
    config: AzureSearchConfig,
}

struct AzureDocument {
    data: HashMap<String, Value>,
}

impl AzureDocument {
    fn new(_document: &Document, _config: &AzureSearchConfig) -> Self {
        // copy field value to AzureDocument hashmap
        let mut data = _document.to_hashmap();
        data.insert(String::from("@search.action"), Value::from("upload"));
        // impl copy_fields
        for copy_field in &_config.copy_fields {
            match parse_copy_field(copy_field) {
                Some((source, target)) => {
                    if data.contains_key(&source) {
                        data.insert(target, data.get(&source).unwrap().clone());
                    } else {
                        warn!("there is no {} field in document.", source);
                    }
                }
                None => warn!("copy_fields setting parse error. setting is {}", copy_field),
            };
        }

        AzureDocument { data }
    }

    fn get_id(&self) -> String {
        return self.data.get("id").unwrap().to_string();
    }

    fn to_json_string(&self) -> String {
        return serde_json::to_string(&self.data).unwrap();
    }
}

fn parse_copy_field(setting: &str) -> Option<(String, String)> {
    let splitted_setting: Vec<&str> = setting.split("=>").collect();
    if splitted_setting.len() == 2 {
        return Some((
            splitted_setting[0].to_string(),
            splitted_setting[1].to_string(),
        ));
    } else {
        return None;
    }
}

fn load_config(config_file: &str) -> AzureSearchConfig {
    let f = File::open(config_file)
        .expect(format!("config file is not found. {}", config_file).as_str());
    let config: AzureSearchConfig =
        serde_yaml::from_reader(f).expect(format!("Parse Error").as_str());
    return config;
}

impl SearchEngine for AzureSearchOutput {
    fn new(_config_file: &str) -> Self
    where
        Self: Sized,
    {
        let config = load_config(_config_file);
        let buffer = vec![];
        let client = reqwest::Client::new();
        AzureSearchOutput {
            client,
            buffer,
            config,
        }
    }

    fn add_document(&mut self, mut _document: Document) {
        // TODO Is it smart??
        for field in &self.config.drop_fields {
            let field = field.as_str();
            match field {
                "images" => &_document.images.clear(),
                "links" => &_document.links.clear(),
                &_ => &(),
            };
        }
        let azure_doc = AzureDocument::new(&_document, &self.config);
        self.buffer.push(azure_doc);
    }

    fn initialize(&self) {
        if self.exist_index() {
            info!(
                "{} index already exists. skip initialization phase.",
                &self.config.index_name
            );
        } else {
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

impl AzureSearchOutput {
    fn get_api_version() -> String {
        String::from("?api-version=2019-05-06")
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            "api-key",
            HeaderValue::from_str(&self.config.api_key).unwrap(),
        );
        return headers;
    }

    fn get_service_url(&self) -> String {
        format!(
            "https://{}.search.windows.net/indexes/{}",
            &self.config.service_name, &self.config.index_name
        )
    }

    async fn call_indices_create(&self) -> Result<(), Error> {
        let schema_json = load_schema(&self.config.schema_file);
        let result = &self
            .client
            .put(
                format!(
                    "{}{}",
                    &self.get_service_url(),
                    AzureSearchOutput::get_api_version()
                )
                .as_str(),
            )
            .headers(self.get_headers())
            .json(&schema_json)
            .send()
            .await;
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    info!("{} index was created.", &self.config.index_name);
                } else {
                    warn!(
                        "Create index request has failed. Status Code is {:?}.",
                        response.status()
                    );
                    warn!("{:?}", response);
                    // FIXME if ...
                    panic!("create index failed")
                }
                return Ok(());
            }
            Err(_) => panic!("create index failed"),
        }
    }

    async fn call_indices_exists(&self) -> Result<bool, Error> {
        //        GET  https://{{host}}/indexes/hogehoge?api-version=2019-05-06
        //        Content-Type: application/json
        //        api-key: {{api-key}}
        let result = &self
            .client
            .get(
                format!(
                    "{}{}",
                    &self.get_service_url(),
                    AzureSearchOutput::get_api_version()
                )
                .as_str(),
            )
            .headers(self.get_headers())
            .send()
            .await;
        match result {
            Ok(response) => match response.status() {
                StatusCode::NOT_FOUND => return Ok(false),
                StatusCode::OK => return Ok(true),
                _ => {
                    warn!(
                        "Indices exists request has failed. Status Code is {:?}.",
                        response.status()
                    );
                    warn!("{:?}", response);
                    panic!("Indices exists request failed")
                }
            },
            Err(err) => {
                error!("{:?}", err);
                panic!("Indices exists request failed...")
            }
        }
    }

    async fn proceed_chunk(
        &self,
        chunk: &[AzureDocument],
    ) -> Result<(), Box<dyn std::error::Error>> {
        //FIXME copy fields...
        // need other settings like field copy mapping...

        let mut docs: Vec<String> = vec![];
        let mut doc_id = String::new();
        let mut chunk_size = 0;
        for d in chunk {
            if doc_id.is_empty() {
                doc_id.push_str(d.get_id().as_str());
            }

            // read json as hashmap
            //serde_json::to_value(d).unwrap().
            let json_string = d.to_json_string();

            docs.push(json_string);
            chunk_size += 1;
        }
        let root_json = format!("{{ \"value\": [{}]}}", docs.join(", "));
        debug!("root_json is {}", &root_json);

        info!("Sending {} documents... {}", chunk_size, doc_id);
        let response = self
            .client
            .post(
                format!(
                    "{}/docs/index{}",
                    &self.get_service_url(),
                    AzureSearchOutput::get_api_version()
                )
                .as_str(),
            )
            .headers(self.get_headers())
            .body(root_json)
            .send()
            .await?;
        if response.status().is_success() {
            info!("response : {}", response.status());
            debug!("{:?}", response);
            let result = response.json::<UploadResponse>().await;
            match result {
                Ok(upload_response) => {
                    for doc_response in upload_response.value {
                        if !doc_response.status {
                            warn!(
                                "error id:[{}], status_code:[{}], reason:[{}]",
                                doc_response.key,
                                doc_response.status_code,
                                doc_response.error_message.unwrap()
                            );
                        }
                    }
                }
                Err(err) => {
                    warn!("Error parse json from response body. {:?}", err);
                    panic!("bulk indexing failed.");
                }
            }
        } else {
            warn!(
                "Bulk request has failed. Status Code is {:?}. First doc id is [{}]",
                response.status(),
                doc_id
            );
            warn!("response - {:?}", response);
            let response_body = response.text().await?;
            warn!("res_body - {:?}", response_body);
            panic!("bulk indexing failed")
        }
        info!("Finished bulk request. {}", doc_id);
        Ok(())
    }
}
