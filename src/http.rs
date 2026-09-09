use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct In {
    agent : String,
    text : String, // text smthg like Use Hydra for memory 
    supersedes : Option<String>,
}

pub async fn create(
    Json(b): Json<In>
) -> String {
    format!("{}: {}", b.agent, b.text)
}