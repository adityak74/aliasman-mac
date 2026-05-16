use arrow_array::{Float32Array, RecordBatch, FixedSizeListArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::{error::Error as LanceError, DistanceType, Db};
use std::sync::Arc;
use thiserror::Error;

use crate::model::AliasRecord;
use crate::store::AliasStore;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Embedding failed: {0}")]
    Embedding(#[from] EmbedError),

    #[error("LanceDB error: {0}")]
    Lance(#[from] LanceError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Schema error: {0}")]
    Schema(String),
}

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("No Ollama response")]
    NoResponse,

    #[error("Ollama not available: {0}")]
    NotAvailable(String),
}

pub const DEFAULT_EMBEDDING_MODEL: &str = "embeddinggemma";
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
pub const DEFAULT_SEARCH_LIMIT: usize = 5;

// --- Embedding Provider ---

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, input: &str) -> Result<Vec<f32>, EmbedError>;
    fn model_name(&self) -> &str;
    fn dimensions(&self) -> usize;
}

pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
    dimensions: usize,
    client: reqwest::Client,
}

impl OllamaEmbeddingProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, dimensions: usize) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            dimensions,
            client: reqwest::Client::new(),
        }
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_OLLAMA_URL, DEFAULT_EMBEDDING_MODEL, 768)
    }

    async fn embed_async(&self, input: &str) -> Result<Vec<f32>, EmbedError> {
        let url = format!("{}/api/embed", self.base_url);
        let payload = serde_json::json!({
            "model": self.model,
            "input": input,
        });

        let resp = self.client.post(&url).json(&payload).send().await?;
        if !resp.status().is_success() {
            return Err(EmbedError::NotAvailable(format!("Ollama returned {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;
        let embeddings = body
            .get("embeddings")
            .ok_or(EmbedError::NoResponse)?
            .as_array()
            .ok_or(EmbedError::NoResponse)?;

        let mut result = Vec::with_capacity(self.dimensions);
        if let Some(first) = embeddings.get(0) {
            if let Some(arr) = first.as_array() {
                for val in arr {
                    result.push(val.as_f64().unwrap_or(0.0) as f32);
                }
            }
        }

        Ok(result)
    }
}

impl EmbeddingProvider for OllamaEmbeddingProvider {
    fn embed(&self, input: &str) -> Result<Vec<f32>, EmbedError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| EmbedError::NotAvailable(e.to_string()))?;
        rt.block_on(self.embed_async(input))
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

// --- Mock provider for tests ---

#[cfg(test)]
pub struct MockEmbeddingProvider {
    pub dimensions: usize,
    pub model: String,
    pub calls: std::sync::Mutex<Vec<String>>,
    pub fail: std::sync::Mutex<bool>,
}

#[cfg(test)]
impl MockEmbeddingProvider {
    pub fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            model: "mock-model".to_string(),
            calls: std::sync::Mutex::new(Vec::new()),
            fail: std::sync::Mutex::new(false),
        }
    }

    pub fn set_fail(&self, fail: bool) {
        *self.fail.lock().unwrap() = fail;
    }
}

#[cfg(test)]
impl EmbeddingProvider for MockEmbeddingProvider {
    fn embed(&self, input: &str) -> Result<Vec<f32>, EmbedError> {
        if *self.fail.lock().unwrap() {
            return Err(EmbedError::NotAvailable("mock failure".to_string()));
        }
        self.calls.lock().unwrap().push(input.to_string());
        let hash = djb2_hash(input);
        let mut vec = Vec::with_capacity(self.dimensions);
        for i in 0..self.dimensions {
            vec.push(((hash.wrapping_mul((i + 1) as u64)) % 1000) as f32 / 1000.0);
        }
        Ok(vec)
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

fn djb2_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

// --- Search Record ---

#[derive(Debug, Clone)]
pub struct SearchRecord {
    pub name: String,
    pub command: String,
    pub search_text: String,
    pub tags: Vec<String>,
    pub source: String,
    pub shell: String,
    pub updated_at: u64,
    pub vector: Vec<f32>,
}

impl SearchRecord {
    pub fn from_alias(record: &AliasRecord, vector: Vec<f32>) -> Self {
        let search_text = format!(
            "{}\n{}\n{}\n{}",
            record.name,
            record.command,
            record.description.as_deref().unwrap_or(""),
            record.tags.join(" ")
        );
        Self {
            name: record.name.clone(),
            command: record.command.clone(),
            search_text,
            tags: record.tags.clone(),
            source: format!("{:?}", record.source).to_lowercase(),
            shell: format!("{:?}", record.shell).to_lowercase(),
            updated_at: record.updated_at,
            vector,
        }
    }
}

// --- Index Metadata ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexMetadata {
    pub embedding_provider: String,
    pub embedding_model: String,
    pub vector_dimensions: usize,
    pub alias_count: usize,
}

// --- LanceDB helpers ---

const TABLE_NAME: &str = "aliases";

async fn connect_db(db_path: &str) -> Result<Db, LanceError> {
    lancedb::connect(db_path).execute().await
}

fn make_schema(dimensions: usize) -> Schema {
    Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("command", DataType::Utf8, false),
        Field::new("search_text", DataType::Utf8, false),
        Field::new("tags", DataType::Utf8, false),
        Field::new("source", DataType::Utf8, false),
        Field::new("shell", DataType::Utf8, false),
        Field::new("updated_at", DataType::UInt64, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Box::new(Field::new("item", DataType::Float32, true)),
                dimensions as i32,
            ),
            false,
        ),
    ])
}

async fn get_or_create_table(db: &Db, dimensions: usize) -> Result<lancedb::Table, LanceError> {
    if db.table(TABLE_NAME).exists() {
        db.open_table(TABLE_NAME).execute().await
    } else {
        let schema = make_schema(dimensions);
        let empty_batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![],
        ).expect("empty batch should match schema");

        let batches = vec![Ok(empty_batch)];
        let iter = batches.into_iter();

        db.create_table(TABLE_NAME, iter)
            .execute()
            .await
    }
}

// --- Reindex ---

pub async fn reindex_aliases<P: EmbeddingProvider>(
    db_path: &str,
    store: &AliasStore,
    provider: &P,
) -> Result<IndexMetadata, SearchError> {
    let db = connect_db(db_path).await?;
    let table = get_or_create_table(&db, provider.dimensions()).await?;

    if store.aliases.is_empty() {
        return Ok(IndexMetadata {
            embedding_provider: "ollama".to_string(),
            embedding_model: provider.model_name().to_string(),
            vector_dimensions: provider.dimensions(),
            alias_count: 0,
        });
    }

    let records: Vec<SearchRecord> = store
        .aliases
        .iter()
        .filter_map(|alias| {
            let vector = provider.embed(&alias.command).ok()?;
            Some(SearchRecord::from_alias(alias, vector))
        })
        .collect();

    if records.is_empty() {
        return Ok(IndexMetadata {
            embedding_provider: "ollama".to_string(),
            embedding_model: provider.model_name().to_string(),
            vector_dimensions: provider.dimensions(),
            alias_count: 0,
        });
    }

    let dims = provider.dimensions();
    let schema = make_schema(dims);

    let name_arr: arrow_array::StringArray = records.iter().map(|r| Some(&r.name)).collect();
    let cmd_arr: arrow_array::StringArray = records.iter().map(|r| Some(&r.command)).collect();
    let stxt_arr: arrow_array::StringArray = records.iter().map(|r| Some(&r.search_text)).collect();
    let tags_arr: arrow_array::StringArray = records.iter().map(|r| Some(&r.tags.join(" "))).collect();
    let src_arr: arrow_array::StringArray = records.iter().map(|r| Some(&r.source)).collect();
    let shell_arr: arrow_array::StringArray = records.iter().map(|r| Some(&r.shell)).collect();
    let updated_arr: arrow_array::UInt64Array = records.iter().map(|r| Some(r.updated_at)).collect();

    let dim = dims as i32;
    let total_vectors = records.len();
    let all_floats: Float32Array = records.iter().flat_map(|r| r.vector.iter().copied()).collect();

    let list_data = arrow_array::ArrayData::builder(DataType::FixedSizeList(
        Box::new(Field::new("item", DataType::Float32, true)),
        dim,
    ))
    .len(total_vectors)
    .add_child_data(all_floats.to_data())
    .build()
    .expect("vector array data should build");

    let vector_arr = FixedSizeListArray::from(list_data);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(name_arr),
            Arc::new(cmd_arr),
            Arc::new(stxt_arr),
            Arc::new(tags_arr),
            Arc::new(src_arr),
            Arc::new(shell_arr),
            Arc::new(updated_arr),
            Arc::new(vector_arr),
        ],
    ).map_err(|e| SearchError::Schema(e.to_string()))?;

    let _ = table.delete("1=1").execute().await;
    table.add(vec![batch]).execute().await?;

    Ok(IndexMetadata {
        embedding_provider: "ollama".to_string(),
        embedding_model: provider.model_name().to_string(),
        vector_dimensions: provider.dimensions(),
        alias_count: records.len(),
    })
}

// --- Semantic Search ---

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub alias_name: String,
    pub command: String,
    pub score: f32,
    pub reason: String,
}

pub async fn search_aliases<P: EmbeddingProvider>(
    db_path: &str,
    query: &str,
    provider: &P,
    limit: usize,
) -> Result<Vec<SearchResult>, SearchError> {
    let db = connect_db(db_path).await?;
    if !db.table(TABLE_NAME).exists() {
        return Ok(Vec::new());
    }

    let table = db.open_table(TABLE_NAME).execute().await?;
    let query_vector = provider.embed(query)?;

    let dims = provider.dimensions() as i32;
    let vec_data = arrow_array::ArrayData::builder(DataType::FixedSizeList(
        Box::new(Field::new("item", DataType::Float32, true)),
        dims,
    ))
    .len(1)
    .add_child_data(Float32Array::from(query_vector).to_data())
    .build()
    .expect("query vector data should build");

    let vec_array = FixedSizeListArray::from(vec_data);

    let results = table
        .query()
        .select(vec!["name", "command", "search_text", "tags"])
        .nearest_to(vec_array)
        .distance_type(DistanceType::Cosine)
        .limit(limit)
        .execute()
        .await?;

    let mut search_results = Vec::new();
    for batch in results {
        let name_col = batch
            .column_by_name("name")
            .and_then(|c| c.as_any().downcast_ref::<arrow_array::StringArray>());
        let cmd_col = batch
            .column_by_name("command")
            .and_then(|c| c.as_any().downcast_ref::<arrow_array::StringArray>());
        let dist_col = batch
            .column_by_name("_distance")
            .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float64Array>());

        for i in 0..batch.num_rows() {
            let name = name_col.map(|c| c.value(i).to_string()).unwrap_or_default();
            let command = cmd_col.map(|c| c.value(i).to_string()).unwrap_or_default();
            let distance = dist_col.map(|c| c.value(i) as f32).unwrap_or(0.0);
            let similarity = 1.0 - distance;

            search_results.push(SearchResult {
                alias_name: name,
                command,
                score: similarity,
                reason: format!("semantic similarity {:.2}", similarity),
            });
        }
    }

    Ok(search_results)
}

// --- Lexical Fallback ---

pub fn lexical_search(store: &AliasStore, query: &str, limit: usize) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<SearchResult> = Vec::new();

    for alias in &store.aliases {
        let name_lower = alias.name.to_lowercase();
        let cmd_lower = alias.command.to_lowercase();
        let desc_lower = alias.description.as_deref().unwrap_or("").to_lowercase();
        let tags_lower = alias.tags.join(" ").to_lowercase();

        let mut score: f32 = 0.0;
        let mut match_reasons = Vec::new();

        if name_lower == query_lower || query_words.iter().any(|w| name_lower.contains(w)) {
            score += 0.8;
            match_reasons.push("name match");
        }

        if cmd_lower == query_lower || query_words.iter().any(|w| cmd_lower.contains(w)) {
            score += 0.6;
            match_reasons.push("command match");
        }

        if !desc_lower.is_empty() && (desc_lower == query_lower || query_words.iter().any(|w| desc_lower.contains(w))) {
            score += 0.3;
            match_reasons.push("description match");
        }

        if query_words.iter().any(|w| tags_lower.contains(w)) {
            score += 0.2;
            match_reasons.push("tag match");
        }

        if score > 0.0 {
            scored.push(SearchResult {
                alias_name: alias.name.clone(),
                command: alias.command.clone(),
                score,
                reason: match_reasons.join(", "),
            });
        }
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored
}

// --- Mutation Refresh ---

pub async fn refresh_alias_index_after_mutation<P: EmbeddingProvider>(
    db_path: &str,
    store: &AliasStore,
    provider: &P,
) {
    if let Err(e) = reindex_aliases(db_path, store, provider).await {
        eprintln!("Warning: index refresh failed: {}", e);
    }
}

// --- Default Index Path ---

pub fn default_index_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/"))
        .join(".config")
        .join("aliasman")
        .join("index")
}
