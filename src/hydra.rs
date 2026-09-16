use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde_json::{json, Value};
use crate::{cypher, error::AppError};

#[derive(Clone)]
pub struct Hydra {
    http: reqwest::Client,
    base: String,
    token: String,
    ns: String,
    graph: String,
    cell: String,
}

fn hydra_id(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    match h.finish() {
        0 => 1,
        n => n,
    }
}

impl Hydra {
    pub fn from_env() -> Self {
        Self {
            http: reqwest::Client::new(),
            base: std::env::var("HYDRA_URL").unwrap_or_else(|_| "http://127.0.0.1:18443".into()),
            token: std::env::var("HYDRA_TOKEN").expect("HYDRA_TOKEN"),
            ns: std::env::var("HYDRA_NAMESPACE").unwrap_or_else(|_| "local".into()),
            graph: std::env::var("HYDRA_GRAPH").unwrap_or_else(|_| "default".into()),
            cell: std::env::var("HYDRA_CELL").unwrap_or_else(|_| "cell-0".into()),
        }
    }

    async fn q(&self, query: &str, parameters: Value) -> Result<Value, AppError> {
        let url = format!("{}/v1/graphs/{}/query", self.base, self.graph);
        let res = self.http.post(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("X-Graph-Namespace", &self.ns)
            .json(&json!({"cell_id": self.cell, "query": query, "parameters": parameters}))
            .send().await
            .map_err(|e| AppError::BadGateway(e.to_string()))?;
        let status = res.status();
        let body: Value = res.json().await.map_err(|e| AppError::BadGateway(e.to_string()))?;
        if !status.is_success() {
            return Err(AppError::BadGateway(format!("{status}: {body}")));
        }
        Ok(body)
    }

    fn cells(body: &Value) -> Vec<(String, String, String)> {
        let cols: Vec<String> = body["columns"].as_array().unwrap_or(&vec![])
            .iter().filter_map(|c| c.as_str().map(String::from)).collect();
        let idx = |n: &str| cols.iter().position(|c| c == n);
        body["rows"].as_array().unwrap_or(&vec![]).iter().filter_map(|row| {
            let row = row.as_array()?;
            let s = |i: usize| row.get(i)?.get("value").and_then(|v| {
                v.as_str().map(String::from).or_else(|| v.as_u64().map(|n| n.to_string()))
            });
            Some((s(idx("id")?)?, s(idx("text")?).unwrap_or_default(), s(idx("ts")?).unwrap_or_default()))
        }).collect()
    }

    pub async fn create(&self, agent: &str, id: &str, text: &str, ts: &str, old: Option<&str>) -> Result<(), AppError> {
        let old_row = if let Some(old) = old {
            let rows = Self::cells(&self.q(cypher::GET, json!({"id": old})).await?);
            let row = rows.into_iter().next().ok_or_else(|| {
                AppError::BadRequest(format!("no decision {old}"))
            })?;
            Some(row)
        } else {
            None
        };

        self.q(
            cypher::CREATE,
            json!({
                "aid": hydra_id(agent),
                "did": hydra_id(id),
                "agent": agent,
                "id": id,
                "text": text,
                "ts": ts,
            }),
        )
        .await?;

        if let Some(old) = old {
            let (old_id, old_text, old_ts) = old_row.expect("checked above");
            self.q(
                cypher::SUPERSEDE,
                json!({
                    "nid": hydra_id(id),
                    "oid": hydra_id(&old_id),
                    "new": id,
                    "old": old,
                    "new_text": text,
                    "new_ts": ts,
                    "old_text": old_text,
                    "old_ts": old_ts,
                }),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn list(&self, agent: &str) -> Result<Vec<(String, String, String)>, AppError> {
        Ok(Self::cells(&self.q(cypher::LIST, json!({"agent": agent})).await?))
    }

    pub async fn lineage(&self, id: &str) -> Result<Vec<(String, String, String)>, AppError> {
        let mut chain = Self::cells(&self.q(cypher::GET, json!({"id": id})).await?);
        if chain.is_empty() {
            return Err(AppError::BadRequest(format!("no decision {id}")));
        }
        let mut cur = id.to_string();
        for _ in 0..32 {
            let hop = Self::cells(&self.q(cypher::HOP, json!({"id": cur})).await?);
            let Some(next) = hop.into_iter().next() else { break };
            cur = next.0.clone();
            chain.push(next);
        }
        Ok(chain) // newest first
    }
}