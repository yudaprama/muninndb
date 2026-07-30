use std::time::Duration;

use futures::Stream;
use reqwest::Client as HttpClient;

use crate::errors::MuninnError;
use crate::sse::{self, SseEvent};
use crate::types::*;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_BACKOFF: Duration = Duration::from_millis(500);

/// MuninnDB REST API client.
#[derive(Debug, Clone)]
pub struct MuninnClient {
    base_url: String,
    token: String,
    http: HttpClient,
    max_retries: u32,
    retry_backoff: Duration,
}

impl MuninnClient {
    /// Create a new client with default settings.
    pub fn new(base_url: &str, token: &str) -> Self {
        Self::with_options(base_url, token, DEFAULT_TIMEOUT, DEFAULT_MAX_RETRIES, DEFAULT_RETRY_BACKOFF)
    }

    /// Create a new client with custom settings.
    pub fn with_options(
        base_url: &str,
        token: &str,
        timeout: Duration,
        max_retries: u32,
        retry_backoff: Duration,
    ) -> Self {
        let http = HttpClient::builder()
            .timeout(timeout)
            .build()
            .expect("failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            http,
            max_retries,
            retry_backoff,
        }
    }

    // ── Write ──────────────────────────────────────────────────────

    /// Write an engram with convenience defaults (confidence=0.9, stability=0.5).
    pub async fn write(&self, vault: &str, concept: &str, content: &str, tags: Vec<String>) -> Result<WriteResponse, MuninnError> {
        let req = WriteRequest::new(vault, concept, content)
            .tags(tags)
            .confidence(0.9)
            .stability(0.5);
        self.write_with_options(&req).await
    }

    /// Write an engram with full control over all fields.
    pub async fn write_with_options(&self, req: &WriteRequest) -> Result<WriteResponse, MuninnError> {
        let path = format!("/api/engrams?vault={}", urlencoding(&req.vault));
        self.request_post(&path, req).await
    }

    /// Write multiple engrams in a single batch call. Maximum 50 per batch.
    pub async fn write_batch(&self, vault: &str, engrams: Vec<WriteRequest>) -> Result<BatchWriteResponse, MuninnError> {
        if engrams.is_empty() {
            return Err(MuninnError::Validation("engrams list must not be empty".to_string()));
        }
        if engrams.len() > 50 {
            return Err(MuninnError::Validation("batch size exceeds maximum of 50".to_string()));
        }
        let path = format!("/api/engrams/batch?vault={}", urlencoding(vault));
        self.request_post(&path, &BatchWritePayload { engrams }).await
    }

    // ── Read ───────────────────────────────────────────────────────

    /// Read an engram by ID.
    pub async fn read(&self, id: &str, vault: &str) -> Result<Engram, MuninnError> {
        let path = format!("/api/engrams/{}?vault={}", id, urlencoding(vault));
        self.request_get(&path).await
    }

    // ── Activate ───────────────────────────────────────────────────

    /// Activate memory based on context.
    pub async fn activate(&self, req: &ActivateRequest) -> Result<ActivateResponse, MuninnError> {
        let path = format!("/api/activate?vault={}", urlencoding(&req.vault));
        self.request_post(&path, req).await
    }

    // ── Link ───────────────────────────────────────────────────────

    /// Link two engrams.
    pub async fn link(&self, req: &LinkRequest) -> Result<(), MuninnError> {
        let path = format!("/api/link?vault={}", urlencoding(&req.vault));
        self.request_post_raw(&path, req).await?;
        Ok(())
    }

    // ── Forget ─────────────────────────────────────────────────────

    /// Forget (soft-delete) an engram.
    pub async fn forget(&self, id: &str, vault: &str) -> Result<(), MuninnError> {
        let path = format!("/api/engrams/{}?vault={}", id, urlencoding(vault));
        self.request_delete(&path).await
    }

    // ── Stats ──────────────────────────────────────────────────────

    /// Get database statistics. Pass an empty vault for global stats.
    pub async fn stats(&self, vault: Option<&str>) -> Result<StatsResponse, MuninnError> {
        let path = match vault {
            Some(v) => format!("/api/stats?vault={}", urlencoding(v)),
            None => "/api/stats".to_string(),
        };
        self.request_get(&path).await
    }

    // ── Subscribe (SSE) ───────────────────────────────────────────

    /// Subscribe to vault events via Server-Sent Events.
    pub async fn subscribe(&self, vault: &str) -> Result<impl Stream<Item = Result<SseEvent, MuninnError>>, MuninnError> {
        let path = format!(
            "/api/subscribe?vault={}&push_on_write=true",
            urlencoding(vault)
        );
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(MuninnError::RequestFailed)?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(MuninnError::from_status(status, &body));
        }

        Ok(sse::parse_sse_stream(resp))
    }

    // ── Health ─────────────────────────────────────────────────────

    /// Check if the server is healthy.
    pub async fn health(&self) -> Result<HealthResponse, MuninnError> {
        self.request_get("/api/health").await
    }

    // ── Evolve ─────────────────────────────────────────────────────

    /// Evolve an engram's content, creating a new version.
    pub async fn evolve(&self, id: &str, vault: &str, new_content: &str, reason: &str) -> Result<EvolveResponse, MuninnError> {
        let path = format!("/api/engrams/{}/evolve?vault={}", id, urlencoding(vault));
        self.request_post(
            &path,
            &EvolvePayload {
                new_content: new_content.to_string(),
                reason: reason.to_string(),
                vault: vault.to_string(),
            },
        )
        .await
    }

    // ── Consolidate ────────────────────────────────────────────────

    /// Merge multiple engrams into one.
    pub async fn consolidate(
        &self,
        vault: &str,
        ids: Vec<String>,
        merged_content: &str,
    ) -> Result<ConsolidateResponse, MuninnError> {
        let path = format!("/api/consolidate?vault={}", urlencoding(vault));
        self.request_post(
            &path,
            &ConsolidatePayload {
                vault: vault.to_string(),
                ids,
                merged_content: merged_content.to_string(),
            },
        )
        .await
    }

    // ── Decide ─────────────────────────────────────────────────────

    /// Record a decision as an engram.
    pub async fn decide(
        &self,
        vault: &str,
        decision: &str,
        rationale: &str,
        alternatives: Option<Vec<String>>,
        evidence_ids: Option<Vec<String>>,
    ) -> Result<DecideResponse, MuninnError> {
        let path = format!("/api/decide?vault={}", urlencoding(vault));
        self.request_post(
            &path,
            &DecidePayload {
                vault: vault.to_string(),
                decision: decision.to_string(),
                rationale: rationale.to_string(),
                alternatives,
                evidence_ids,
            },
        )
        .await
    }

    // ── Restore ────────────────────────────────────────────────────

    /// Restore a soft-deleted engram.
    pub async fn restore(&self, id: &str, vault: &str) -> Result<RestoreResponse, MuninnError> {
        let path = format!("/api/engrams/{}/restore?vault={}", id, urlencoding(vault));
        self.request_post_raw_empty(&path).await
    }

    // ── Traverse ───────────────────────────────────────────────────

    /// Traverse the association graph from a starting engram.
    pub async fn traverse(
        &self,
        vault: &str,
        start_id: &str,
        max_hops: i32,
        max_nodes: i32,
        rel_types: Option<Vec<String>>,
        follow_entities: Option<bool>,
    ) -> Result<TraverseResponse, MuninnError> {
        let path = format!("/api/traverse?vault={}", urlencoding(vault));
        self.request_post(
            &path,
            &TraversePayload {
                vault: vault.to_string(),
                start_id: start_id.to_string(),
                max_hops,
                max_nodes,
                rel_types,
                follow_entities,
            },
        )
        .await
    }

    // ── Explain ────────────────────────────────────────────────────

    /// Explain why an engram would or wouldn't be returned for a query.
    pub async fn explain(
        &self,
        vault: &str,
        engram_id: &str,
        query: Vec<String>,
    ) -> Result<ExplainResponse, MuninnError> {
        let path = format!("/api/explain?vault={}", urlencoding(vault));
        self.request_post(
            &path,
            &ExplainPayload {
                vault: vault.to_string(),
                engram_id: engram_id.to_string(),
                query,
            },
        )
        .await
    }

    // ── Set State ──────────────────────────────────────────────────

    /// Set the state of an engram.
    pub async fn set_state(
        &self,
        id: &str,
        vault: &str,
        state: &str,
        reason: Option<&str>,
    ) -> Result<SetStateResponse, MuninnError> {
        let path = format!("/api/engrams/{}/state?vault={}", id, urlencoding(vault));
        self.request_put(
            &path,
            &SetStatePayload {
                state: state.to_string(),
                reason: reason.map(|s| s.to_string()),
            },
        )
        .await
    }

    // ── List Deleted ───────────────────────────────────────────────

    /// List soft-deleted engrams that can be restored.
    pub async fn list_deleted(&self, vault: &str, limit: Option<i32>) -> Result<ListDeletedResponse, MuninnError> {
        let mut path = format!("/api/deleted?vault={}", urlencoding(vault));
        if let Some(l) = limit {
            path = format!("{}&limit={}", path, l);
        }
        self.request_get(&path).await
    }

    // ── Retry Enrich ───────────────────────────────────────────────

    /// Retry enrichment plugins for an engram.
    pub async fn retry_enrich(&self, id: &str, vault: &str) -> Result<RetryEnrichResponse, MuninnError> {
        let path = format!("/api/engrams/{}/retry-enrich?vault={}", id, urlencoding(vault));
        self.request_post_raw_empty(&path).await
    }

    // ── Contradictions ─────────────────────────────────────────────

    /// List detected contradictions in a vault.
    pub async fn contradictions(&self, vault: &str) -> Result<ContradictionsResponse, MuninnError> {
        let path = format!("/api/contradictions?vault={}", urlencoding(vault));
        self.request_get(&path).await
    }

    // ── Guide ──────────────────────────────────────────────────────

    /// Return a natural-language guide of a vault's contents.
    pub async fn guide(&self, vault: &str) -> Result<String, MuninnError> {
        let path = format!("/api/guide?vault={}", urlencoding(vault));
        let resp: GuideResponse = self.request_get(&path).await?;
        Ok(resp.guide)
    }

    // ── List Engrams ───────────────────────────────────────────────

    /// List engrams with pagination.
    pub async fn list_engrams(
        &self,
        vault: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<ListEngramsResponse, MuninnError> {
        let mut path = format!("/api/engrams?vault={}", urlencoding(vault));
        if let Some(l) = limit {
            path = format!("{}&limit={}", path, l);
        }
        if let Some(o) = offset {
            path = format!("{}&offset={}", path, o);
        }
        self.request_get(&path).await
    }

    // ── Get Links ──────────────────────────────────────────────────

    /// Get associations/links for an engram.
    pub async fn get_links(&self, id: &str, vault: &str) -> Result<Vec<AssociationItem>, MuninnError> {
        let path = format!("/api/engrams/{}/links?vault={}", id, urlencoding(vault));
        self.request_get(&path).await
    }

    // ── List Vaults ────────────────────────────────────────────────

    /// List all available vaults.
    pub async fn list_vaults(&self) -> Result<Vec<String>, MuninnError> {
        let resp: VaultsResponse = self.request_get("/api/vaults").await?;
        Ok(resp.vaults)
    }

    // ── Session ────────────────────────────────────────────────────

    /// Get session activity for a vault.
    pub async fn session(
        &self,
        vault: &str,
        since: Option<&str>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<SessionResponse, MuninnError> {
        let mut path = format!("/api/session?vault={}", urlencoding(vault));
        if let Some(s) = since {
            path = format!("{}&since={}", path, urlencoding(s));
        }
        if let Some(l) = limit {
            path = format!("{}&limit={}", path, l);
        }
        if let Some(o) = offset {
            path = format!("{}&offset={}", path, o);
        }
        self.request_get(&path).await
    }

    // ── Internal helpers ───────────────────────────────────────────

    async fn request_get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, MuninnError> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<MuninnError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/json")
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let body = r.text().await.map_err(MuninnError::RequestFailed)?;
                    return serde_json::from_str(&body).map_err(|e| {
                        MuninnError::Validation(format!("failed to decode response: {}", e))
                    });
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    let err = MuninnError::from_status(status, &body);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let err = MuninnError::RequestFailed(e);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| MuninnError::MaxRetriesExceeded("exhausted".to_string())))
    }

    async fn request_post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, MuninnError> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<MuninnError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .json(body)
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let body = r.text().await.map_err(MuninnError::RequestFailed)?;
                    return serde_json::from_str(&body).map_err(|e| {
                        MuninnError::Validation(format!("failed to decode response: {}", e))
                    });
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    let err = MuninnError::from_status(status, &body);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let err = MuninnError::RequestFailed(e);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| MuninnError::MaxRetriesExceeded("exhausted".to_string())))
    }

    async fn request_post_raw<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<(), MuninnError> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<MuninnError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Content-Type", "application/json")
                .json(body)
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    return Ok(());
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    let err = MuninnError::from_status(status, &body);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let err = MuninnError::RequestFailed(e);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| MuninnError::MaxRetriesExceeded("exhausted".to_string())))
    }

    async fn request_post_raw_empty<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, MuninnError> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<MuninnError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/json")
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let body = r.text().await.map_err(MuninnError::RequestFailed)?;
                    return serde_json::from_str(&body).map_err(|e| {
                        MuninnError::Validation(format!("failed to decode response: {}", e))
                    });
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    let err = MuninnError::from_status(status, &body);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let err = MuninnError::RequestFailed(e);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| MuninnError::MaxRetriesExceeded("exhausted".to_string())))
    }

    async fn request_put<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, MuninnError> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<MuninnError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .put(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .json(body)
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let body = r.text().await.map_err(MuninnError::RequestFailed)?;
                    return serde_json::from_str(&body).map_err(|e| {
                        MuninnError::Validation(format!("failed to decode response: {}", e))
                    });
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    let err = MuninnError::from_status(status, &body);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let err = MuninnError::RequestFailed(e);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| MuninnError::MaxRetriesExceeded("exhausted".to_string())))
    }

    async fn request_delete(&self, path: &str) -> Result<(), MuninnError> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<MuninnError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .delete(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    return Ok(());
                }
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    let err = MuninnError::from_status(status, &body);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let err = MuninnError::RequestFailed(e);
                    if err.is_retryable() && attempt < self.max_retries {
                        last_err = Some(err);
                        self.backoff(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| MuninnError::MaxRetriesExceeded("exhausted".to_string())))
    }

    async fn backoff(&self, attempt: u32) {
        let delay = self.retry_backoff * 2u32.pow(attempt.min(10));
        tokio::time::sleep(delay).await;
    }
}

/// Percent-encode a string for use in query parameters.
fn urlencoding(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('+', "%2B")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('?', "%3F")
        .replace('#', "%23")
        .replace('/', "%2F")
}
