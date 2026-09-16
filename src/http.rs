use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::AppError, hydra::Hydra};

#[derive(Clone)]
pub struct S { pub hydra: Hydra }

#[derive(Deserialize)]
pub struct In { agent: String, text: String, supersedes: Option<String> }

#[derive(Serialize)]
pub struct Out { id: String, agent: String, text: String }

#[derive(Serialize)]
struct D { id: String, text: String, ts: String }

pub async fn healthz() -> StatusCode { StatusCode::OK }

pub async fn create(State(s): State<S>, Json(b): Json<In>) -> Result<Json<Out>, AppError> {
    let id = Uuid::new_v4().to_string();
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
    let old = b.supersedes.as_deref().filter(|x| !x.is_empty());
    s.hydra.create(&b.agent, &id, &b.text, &ts, old).await?;
    Ok(Json(Out { id, agent: b.agent, text: b.text }))
}

pub async fn list(State(s): State<S>, Path(agent): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let decisions: Vec<D> = s.hydra.list(&agent).await?.into_iter()
        .map(|(id, text, ts)| D { id, text, ts }).collect();
    Ok(Json(serde_json::json!({ "agent": agent, "decisions": decisions })))
}

pub async fn lineage(State(s): State<S>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let chain: Vec<D> = s.hydra.lineage(&id).await?.into_iter()
        .map(|(id, text, ts)| D { id, text, ts }).collect();
    Ok(Json(serde_json::json!({ "id": id, "chain": chain })))
}