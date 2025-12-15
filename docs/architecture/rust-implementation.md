# AOF Memory System - Rust Implementation

## Core Traits and Structures

### Memory Backend Traits

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use anyhow::Result;

/// Memory entry stored in backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: SystemTime,
    pub ttl: Option<Duration>,
    pub entry_type: MemoryEntryType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryEntryType {
    Conversational,
    Semantic,
    Knowledge,
    System,
}

/// Core memory backend trait
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Store a memory entry
    async fn store(&self, entry: MemoryEntry) -> Result<()>;

    /// Retrieve a memory entry by key
    async fn retrieve(&self, key: &str) -> Result<Option<MemoryEntry>>;

    /// List entries with optional filtering
    async fn list(
        &self,
        filter: Option<MemoryFilter>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryEntry>>;

    /// Delete a memory entry
    async fn delete(&self, key: &str) -> Result<()>;

    /// Update TTL for an entry
    async fn update_ttl(&self, key: &str, ttl: Duration) -> Result<()>;

    /// Batch operations
    async fn batch_store(&self, entries: Vec<MemoryEntry>) -> Result<()>;
    async fn batch_delete(&self, keys: Vec<String>) -> Result<()>;

    /// Check if backend is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Get backend statistics
    async fn stats(&self) -> Result<BackendStats>;
}

#[derive(Debug, Clone)]
pub struct MemoryFilter {
    pub entry_type: Option<MemoryEntryType>,
    pub metadata_filters: HashMap<String, serde_json::Value>,
    pub time_range: Option<(SystemTime, SystemTime)>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub oldest_entry: Option<SystemTime>,
    pub newest_entry: Option<SystemTime>,
}

/// Conversational memory management
#[async_trait]
pub trait ConversationalMemory: Send + Sync {
    /// Add message to conversation
    async fn add_message(&self, conversation_id: &str, message: ChatMessage) -> Result<()>;

    /// Get conversation history
    async fn get_history(
        &self,
        conversation_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<ChatMessage>>;

    /// Get last N messages
    async fn get_recent(
        &self,
        conversation_id: &str,
        count: usize,
    ) -> Result<Vec<ChatMessage>>;

    /// Clear conversation
    async fn clear(&self, conversation_id: &str) -> Result<()>;

    /// Summarize conversation
    async fn summarize(
        &self,
        conversation_id: &str,
        model: &str,
    ) -> Result<String>;

    /// Get token count for conversation
    async fn token_count(&self, conversation_id: &str) -> Result<usize>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: SystemTime,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}
```

### Vector Store Traits

```rust
use ndarray::Array1;

/// Document for vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub chunks: Option<Vec<DocumentChunk>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub start_char: usize,
    pub end_char: usize,
}

/// Vector store operations
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Upsert documents (insert or update)
    async fn upsert(&self, docs: Vec<Document>) -> Result<()>;

    /// Search by embedding vector
    async fn search(
        &self,
        embedding: &[f32],
        top_k: usize,
        filter: Option<MetadataFilter>,
    ) -> Result<Vec<SearchResult>>;

    /// Search by document ID
    async fn search_by_id(
        &self,
        id: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Hybrid search (semantic + keyword)
    async fn hybrid_search(
        &self,
        embedding: &[f32],
        query: &str,
        top_k: usize,
        semantic_weight: f32,
        keyword_weight: f32,
    ) -> Result<Vec<SearchResult>>;

    /// Delete documents
    async fn delete(&self, ids: Vec<String>) -> Result<()>;

    /// Delete by filter
    async fn delete_by_filter(&self, filter: MetadataFilter) -> Result<usize>;

    /// Get document by ID
    async fn get(&self, id: &str) -> Result<Option<Document>>;

    /// List all documents
    async fn list(
        &self,
        filter: Option<MetadataFilter>,
        limit: Option<usize>,
    ) -> Result<Vec<Document>>;

    /// Create index (for optimization)
    async fn create_index(&self, config: IndexConfig) -> Result<()>;

    /// Get collection stats
    async fn stats(&self) -> Result<VectorStoreStats>;
}

#[derive(Debug, Clone)]
pub struct MetadataFilter {
    pub must: Vec<FilterCondition>,
    pub should: Vec<FilterCondition>,
    pub must_not: Vec<FilterCondition>,
}

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub highlights: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct IndexConfig {
    pub index_type: IndexType,
    pub metric: DistanceMetric,
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    HNSW,
    IVF,
    Flat,
    IVFPQ,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreStats {
    pub total_documents: usize,
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_size_bytes: u64,
}
```

### Embedding Provider Traits

```rust
/// Embedding provider for text vectorization
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Embed multiple texts (batch)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get embedding dimensions
    fn dimensions(&self) -> usize;

    /// Get model name
    fn model_name(&self) -> &str;

    /// Get max tokens per request
    fn max_tokens(&self) -> usize;

    /// Calculate estimated tokens for text
    fn estimate_tokens(&self, text: &str) -> usize;
}

/// Multi-provider embedding manager
pub struct EmbeddingManager {
    providers: HashMap<String, Box<dyn EmbeddingProvider>>,
    default_provider: String,
}

impl EmbeddingManager {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: String::new(),
        }
    }

    pub fn register(
        &mut self,
        name: String,
        provider: Box<dyn EmbeddingProvider>,
        is_default: bool,
    ) {
        if is_default || self.default_provider.is_empty() {
            self.default_provider = name.clone();
        }
        self.providers.insert(name, provider);
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_with_provider(&self.default_provider, text).await
    }

    pub async fn embed_with_provider(
        &self,
        provider: &str,
        text: &str,
    ) -> Result<Vec<f32>> {
        let provider = self.providers.get(provider)
            .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", provider))?;
        provider.embed(text).await
    }
}
```

### Chunking Strategies

```rust
/// Text chunking trait
#[async_trait]
pub trait ChunkingStrategy: Send + Sync {
    /// Chunk text into smaller pieces
    async fn chunk(&self, text: &str, metadata: Option<HashMap<String, serde_json::Value>>)
        -> Result<Vec<TextChunk>>;

    /// Get strategy name
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub content: String,
    pub start_char: usize,
    pub end_char: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Fixed-size chunking
pub struct FixedSizeChunking {
    chunk_size: usize,
    overlap: usize,
}

#[async_trait]
impl ChunkingStrategy for FixedSizeChunking {
    async fn chunk(&self, text: &str, metadata: Option<HashMap<String, serde_json::Value>>)
        -> Result<Vec<TextChunk>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;

        while start < chars.len() {
            let end = (start + self.chunk_size).min(chars.len());
            let chunk_text: String = chars[start..end].iter().collect();

            chunks.push(TextChunk {
                content: chunk_text,
                start_char: start,
                end_char: end,
                metadata: metadata.clone().unwrap_or_default(),
            });

            start += self.chunk_size - self.overlap;
            if start >= chars.len() {
                break;
            }
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "fixed-size"
    }
}

/// Semantic chunking (uses embeddings)
pub struct SemanticChunking {
    embedding_provider: Box<dyn EmbeddingProvider>,
    threshold: f32,
    min_chunk_size: usize,
    max_chunk_size: usize,
}

#[async_trait]
impl ChunkingStrategy for SemanticChunking {
    async fn chunk(&self, text: &str, metadata: Option<HashMap<String, serde_json::Value>>)
        -> Result<Vec<TextChunk>> {
        // Split by sentences
        let sentences = self.split_sentences(text);

        // Embed all sentences
        let embeddings = self.embedding_provider.embed_batch(&sentences).await?;

        // Group by semantic similarity
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut start_char = 0;

        for (i, sentence) in sentences.iter().enumerate() {
            if current_chunk.is_empty() {
                current_chunk = sentence.clone();
                start_char = text.find(sentence).unwrap_or(0);
                continue;
            }

            // Check similarity with previous sentence
            let similarity = if i > 0 {
                self.cosine_similarity(&embeddings[i-1], &embeddings[i])
            } else {
                1.0
            };

            // If similarity below threshold or max size reached, start new chunk
            if similarity < self.threshold || current_chunk.len() > self.max_chunk_size {
                let end_char = start_char + current_chunk.len();
                chunks.push(TextChunk {
                    content: current_chunk.clone(),
                    start_char,
                    end_char,
                    metadata: metadata.clone().unwrap_or_default(),
                });

                current_chunk = sentence.clone();
                start_char = text.find(sentence).unwrap_or(end_char);
            } else {
                current_chunk.push(' ');
                current_chunk.push_str(sentence);
            }
        }

        // Add final chunk
        if !current_chunk.is_empty() && current_chunk.len() >= self.min_chunk_size {
            chunks.push(TextChunk {
                content: current_chunk,
                start_char,
                end_char: text.len(),
                metadata: metadata.unwrap_or_default(),
            });
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "semantic"
    }
}

impl SemanticChunking {
    fn split_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting (can be improved with NLP libraries)
        text.split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b)
    }
}

/// Markdown-aware chunking
pub struct MarkdownChunking {
    max_chunk_size: usize,
    respect_headers: bool,
    respect_code_blocks: bool,
}

#[async_trait]
impl ChunkingStrategy for MarkdownChunking {
    async fn chunk(&self, text: &str, metadata: Option<HashMap<String, serde_json::Value>>)
        -> Result<Vec<TextChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        let mut current_chunk = String::new();
        let mut start_line = 0;
        let mut in_code_block = false;

        for (i, line) in lines.iter().enumerate() {
            // Check for code blocks
            if line.starts_with("```") {
                in_code_block = !in_code_block;
            }

            // Check for headers
            let is_header = line.starts_with('#') && !in_code_block;

            // Start new chunk on header if configured
            if is_header && self.respect_headers && !current_chunk.is_empty() {
                chunks.push(TextChunk {
                    content: current_chunk.clone(),
                    start_char: start_line,
                    end_char: i,
                    metadata: metadata.clone().unwrap_or_default(),
                });
                current_chunk = String::new();
                start_line = i;
            }

            // Don't split code blocks
            if in_code_block && self.respect_code_blocks {
                current_chunk.push_str(line);
                current_chunk.push('\n');
                continue;
            }

            // Check size limits
            if current_chunk.len() + line.len() > self.max_chunk_size {
                chunks.push(TextChunk {
                    content: current_chunk.clone(),
                    start_char: start_line,
                    end_char: i,
                    metadata: metadata.clone().unwrap_or_default(),
                });
                current_chunk = String::new();
                start_line = i;
            }

            current_chunk.push_str(line);
            current_chunk.push('\n');
        }

        // Add final chunk
        if !current_chunk.is_empty() {
            chunks.push(TextChunk {
                content: current_chunk,
                start_char: start_line,
                end_char: lines.len(),
                metadata: metadata.unwrap_or_default(),
            });
        }

        Ok(chunks)
    }

    fn name(&self) -> &str {
        "markdown"
    }
}
```

### RAG Pipeline

```rust
/// RAG (Retrieval-Augmented Generation) pipeline
pub struct RAGPipeline {
    vector_store: Box<dyn VectorStore>,
    embedding_provider: Box<dyn EmbeddingProvider>,
    reranker: Option<Box<dyn Reranker>>,
    config: RAGConfig,
}

#[derive(Debug, Clone)]
pub struct RAGConfig {
    pub top_k: usize,
    pub threshold: f32,
    pub rerank_top_k: Option<usize>,
    pub hybrid_search: bool,
    pub semantic_weight: f32,
    pub keyword_weight: f32,
    pub max_context_tokens: usize,
}

impl RAGPipeline {
    pub fn new(
        vector_store: Box<dyn VectorStore>,
        embedding_provider: Box<dyn EmbeddingProvider>,
        config: RAGConfig,
    ) -> Self {
        Self {
            vector_store,
            embedding_provider,
            reranker: None,
            config,
        }
    }

    pub fn with_reranker(mut self, reranker: Box<dyn Reranker>) -> Self {
        self.reranker = Some(reranker);
        self
    }

    /// Retrieve relevant context for a query
    pub async fn retrieve(&self, query: &str, filter: Option<MetadataFilter>)
        -> Result<Vec<SearchResult>> {
        // Embed query
        let query_embedding = self.embedding_provider.embed(query).await?;

        // Search vector store
        let mut results = if self.config.hybrid_search {
            self.vector_store.hybrid_search(
                &query_embedding,
                query,
                self.config.top_k,
                self.config.semantic_weight,
                self.config.keyword_weight,
            ).await?
        } else {
            self.vector_store.search(
                &query_embedding,
                self.config.top_k,
                filter,
            ).await?
        };

        // Filter by threshold
        results.retain(|r| r.score >= self.config.threshold);

        // Rerank if configured
        if let Some(reranker) = &self.reranker {
            let rerank_top_k = self.config.rerank_top_k.unwrap_or(5);
            results = reranker.rerank(query, results, rerank_top_k).await?;
        }

        Ok(results)
    }

    /// Build context string from retrieval results
    pub fn build_context(&self, results: Vec<SearchResult>) -> Result<String> {
        let mut context = String::new();
        let mut token_count = 0;

        for (i, result) in results.iter().enumerate() {
            // Estimate tokens (rough: 1 token ≈ 4 chars)
            let result_tokens = result.content.len() / 4;

            if token_count + result_tokens > self.config.max_context_tokens {
                break;
            }

            context.push_str(&format!("\n--- Document {} ---\n", i + 1));
            context.push_str(&result.content);
            context.push('\n');

            token_count += result_tokens;
        }

        Ok(context)
    }

    /// Full RAG: retrieve and build context
    pub async fn augment(&self, query: &str, filter: Option<MetadataFilter>)
        -> Result<String> {
        let results = self.retrieve(query, filter).await?;
        self.build_context(results)
    }
}

/// Reranking trait for improving retrieval results
#[async_trait]
pub trait Reranker: Send + Sync {
    async fn rerank(
        &self,
        query: &str,
        results: Vec<SearchResult>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>>;
}
```

### Context Manager

```rust
/// Context manager for agent execution
pub struct ContextManager {
    memory: Box<dyn MemoryBackend>,
    rag_pipeline: Option<RAGPipeline>,
    static_contexts: HashMap<String, String>,
}

impl ContextManager {
    pub fn new(memory: Box<dyn MemoryBackend>) -> Self {
        Self {
            memory,
            rag_pipeline: None,
            static_contexts: HashMap::new(),
        }
    }

    pub fn with_rag(mut self, rag: RAGPipeline) -> Self {
        self.rag_pipeline = Some(rag);
        self
    }

    pub fn add_static_context(&mut self, name: String, content: String) {
        self.static_contexts.insert(name, content);
    }

    /// Build complete context for agent
    pub async fn build_context(
        &self,
        conversation_id: &str,
        query: &str,
        config: ContextConfig,
    ) -> Result<AgentContext> {
        let mut context = AgentContext::default();

        // Add static contexts
        if config.include_static {
            for (name, content) in &self.static_contexts {
                context.add_section(name, content);
            }
        }

        // Add conversational memory
        if config.include_conversational {
            let messages = self.get_conversation_history(
                conversation_id,
                config.max_messages,
            ).await?;
            context.set_conversation(messages);
        }

        // Add RAG context
        if config.include_rag {
            if let Some(rag) = &self.rag_pipeline {
                let rag_context = rag.augment(query, config.rag_filter).await?;
                context.add_section("knowledge_base", &rag_context);
            }
        }

        // Add dynamic contexts (fetched at runtime)
        if config.include_dynamic {
            for fetcher in &config.dynamic_fetchers {
                let content = fetcher.fetch().await?;
                context.add_section(&fetcher.name(), &content);
            }
        }

        Ok(context)
    }

    async fn get_conversation_history(
        &self,
        conversation_id: &str,
        max_messages: Option<usize>,
    ) -> Result<Vec<ChatMessage>> {
        // Implementation depends on memory backend
        // This is a simplified version
        let filter = MemoryFilter {
            entry_type: Some(MemoryEntryType::Conversational),
            metadata_filters: {
                let mut map = HashMap::new();
                map.insert(
                    "conversation_id".to_string(),
                    serde_json::json!(conversation_id),
                );
                map
            },
            time_range: None,
            tags: None,
        };

        let entries = self.memory.list(Some(filter), max_messages).await?;

        // Convert entries to messages
        let messages: Vec<ChatMessage> = entries
            .into_iter()
            .filter_map(|entry| serde_json::from_str(&entry.content).ok())
            .collect();

        Ok(messages)
    }
}

#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub include_static: bool,
    pub include_conversational: bool,
    pub include_rag: bool,
    pub include_dynamic: bool,
    pub max_messages: Option<usize>,
    pub rag_filter: Option<MetadataFilter>,
    pub dynamic_fetchers: Vec<Box<dyn DynamicContextFetcher>>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            include_static: true,
            include_conversational: true,
            include_rag: false,
            include_dynamic: false,
            max_messages: Some(50),
            rag_filter: None,
            dynamic_fetchers: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct AgentContext {
    pub sections: HashMap<String, String>,
    pub conversation: Vec<ChatMessage>,
}

impl AgentContext {
    pub fn add_section(&mut self, name: &str, content: &str) {
        self.sections.insert(name.to_string(), content.to_string());
    }

    pub fn set_conversation(&mut self, messages: Vec<ChatMessage>) {
        self.conversation = messages;
    }

    pub fn to_prompt(&self) -> String {
        let mut prompt = String::new();

        // Add sections
        for (name, content) in &self.sections {
            prompt.push_str(&format!("\n## {}\n{}\n", name, content));
        }

        // Add conversation
        if !self.conversation.is_empty() {
            prompt.push_str("\n## Conversation History\n");
            for msg in &self.conversation {
                prompt.push_str(&format!("{:?}: {}\n", msg.role, msg.content));
            }
        }

        prompt
    }
}

/// Dynamic context fetcher trait
#[async_trait]
pub trait DynamicContextFetcher: Send + Sync {
    async fn fetch(&self) -> Result<String>;
    fn name(&self) -> String;
}
```

This Rust implementation provides:
1. Complete trait definitions for memory backends, vector stores, and embedding providers
2. Multiple chunking strategies (fixed, semantic, markdown)
3. Full RAG pipeline with reranking support
4. Context management system
5. Type-safe, async-first design
6. Extensible architecture for adding new backends

The traits are designed to be implemented for specific backends (Redis, PostgreSQL, Qdrant, etc.) while maintaining a consistent interface.
