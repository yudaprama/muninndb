use serde::{Deserialize, Serialize};

// ── Engram ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engram {
    pub id: String,
    pub concept: String,
    pub content: String,
    pub confidence: f64,
    pub relevance: f64,
    pub stability: f64,
    pub access_count: i64,
    pub tags: Option<Vec<String>>,
    pub state: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_access: Option<i64>,
    pub memory_type: Option<i32>,
    pub type_label: Option<String>,
    pub summary: Option<String>,
    pub key_points: Option<Vec<String>>,
    pub embed_dim: Option<i32>,
}

// ── Write ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineEntity {
    pub name: String,
    pub entity_type: String,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineRelationship {
    pub target_id: String,
    pub relation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    pub vault: String,
    pub concept: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associations: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<InlineEntity>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Vec<InlineRelationship>>,
}

impl WriteRequest {
    pub fn new(vault: &str, concept: &str, content: &str) -> Self {
        Self {
            vault: vault.to_string(),
            concept: concept.to_string(),
            content: content.to_string(),
            tags: None,
            confidence: None,
            stability: None,
            embedding: None,
            associations: None,
            memory_type: None,
            type_label: None,
            summary: None,
            entities: None,
            relationships: None,
        }
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn stability(mut self, stability: f64) -> Self {
        self.stability = Some(stability);
        self
    }

    pub fn summary(mut self, summary: &str) -> Self {
        self.summary = Some(summary.to_string());
        self
    }

    pub fn memory_type(mut self, memory_type: i32) -> Self {
        self.memory_type = Some(memory_type);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResponse {
    pub id: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

// ── Batch Write ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchWriteResponse {
    pub results: Vec<BatchWriteResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchWriteResult {
    pub index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Activate ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateRequest {
    pub vault: String,
    pub context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hops: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_why: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weights: Option<Weights>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<Filter>>,
}

impl ActivateRequest {
    pub fn new(vault: &str, context: Vec<String>) -> Self {
        Self {
            vault: vault.to_string(),
            context,
            max_results: None,
            threshold: None,
            max_hops: None,
            include_why: None,
            brief_mode: None,
            weights: None,
            filters: None,
        }
    }

    pub fn max_results(mut self, n: i32) -> Self {
        self.max_results = Some(n);
        self
    }

    pub fn threshold(mut self, t: f64) -> Self {
        self.threshold = Some(t);
        self
    }

    pub fn include_why(mut self, b: bool) -> Self {
        self.include_why = Some(b);
        self
    }

    pub fn brief_mode(mut self, mode: &str) -> Self {
        self.brief_mode = Some(mode.to_string());
        self
    }

    pub fn filters(mut self, filters: Vec<Filter>) -> Self {
        self.filters = Some(filters);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_relevance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decay_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hebbian_boost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_frequency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateResponse {
    pub query_id: String,
    pub total_found: i32,
    pub activations: Vec<ActivationItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<Vec<BriefSentence>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationItem {
    pub id: String,
    pub concept: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub score: f64,
    pub confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hop_path: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dormant: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefSentence {
    pub engram_id: String,
    pub text: String,
    pub score: f64,
}

// ── Link ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRequest {
    pub vault: String,
    pub source_id: String,
    pub target_id: String,
    pub rel_type: i32,
    pub weight: f64,
}

// ── Push (SSE) ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Push {
    pub subscription_id: String,
    pub trigger: String,
    pub push_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engram_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
}

// ── Stats ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub engram_count: i64,
    pub vault_count: i32,
    pub storage_bytes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coherence: Option<std::collections::HashMap<String, CoherenceResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceResult {
    pub score: f64,
    pub orphan_ratio: f64,
    pub contradiction_density: f64,
    pub duplication_pressure: f64,
    pub temporal_variance: f64,
    pub total_engrams: i64,
}

// ── Evolve ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolveResponse {
    pub id: String,
}

// ── Consolidate ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidateResponse {
    pub id: String,
    pub archived: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

// ── Decide ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecideResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

// ── Restore ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResponse {
    pub id: String,
    pub concept: String,
    pub restored: bool,
    pub state: String,
}

// ── Traverse ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseResponse {
    pub nodes: Vec<TraversalNode>,
    pub edges: Vec<TraversalEdge>,
    pub total_reachable: i32,
    pub query_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalNode {
    pub id: String,
    pub concept: String,
    pub hop_dist: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalEdge {
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub weight: f32,
}

// ── Explain ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResponse {
    pub engram_id: String,
    pub concept: String,
    pub final_score: f64,
    pub components: ExplainComponents,
    pub fts_matches: Vec<String>,
    pub assoc_path: Vec<String>,
    pub would_return: bool,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainComponents {
    pub full_text_relevance: f64,
    pub semantic_similarity: f64,
    pub decay_factor: f64,
    pub hebbian_boost: f64,
    pub access_frequency: f64,
    pub confidence: f64,
}

// ── Set State ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStateResponse {
    pub id: String,
    pub state: String,
    pub updated: bool,
}

// ── List Deleted ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDeletedResponse {
    pub deleted: Vec<DeletedEngram>,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedEngram {
    pub id: String,
    pub concept: String,
    pub deleted_at: i64,
    pub recoverable_until: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

// ── Retry Enrich ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryEnrichResponse {
    pub engram_id: String,
    pub plugins_queued: Vec<String>,
    pub already_complete: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ── Contradictions ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionsResponse {
    pub contradictions: Vec<ContradictionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionItem {
    pub id_a: String,
    pub concept_a: String,
    pub id_b: String,
    pub concept_b: String,
    pub detected_at: i64,
}

// ── Guide ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideResponse {
    pub guide: String,
}

// ── List Engrams ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEngramsResponse {
    pub engrams: Vec<EngramItem>,
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramItem {
    pub id: String,
    pub concept: String,
    pub content: String,
    pub confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub vault: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed_dim: Option<i32>,
}

// ── Associations ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationItem {
    pub target_id: String,
    pub rel_type: i32,
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub co_activation_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_at: Option<i64>,
}

// ── Session ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub entries: Vec<SessionEntry>,
    pub total: i32,
    pub offset: i32,
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: String,
    pub concept: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub created_at: i64,
}

// ── Health ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_writable: Option<bool>,
}

// ── Vaults ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultsResponse {
    pub vaults: Vec<String>,
}

// ── Internal helpers ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BatchWritePayload {
    pub engrams: Vec<WriteRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvolvePayload {
    pub new_content: String,
    pub reason: String,
    pub vault: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConsolidatePayload {
    pub vault: String,
    pub ids: Vec<String>,
    pub merged_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DecidePayload {
    pub vault: String,
    pub decision: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SetStatePayload {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TraversePayload {
    pub vault: String,
    pub start_id: String,
    pub max_hops: i32,
    pub max_nodes: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_entities: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExplainPayload {
    pub vault: String,
    pub engram_id: String,
    pub query: Vec<String>,
}
