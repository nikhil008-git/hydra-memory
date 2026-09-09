use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
#[derive(Deserialize)]
pub struct In {
    agent : String,
    text : String, // text smthg like Use Hydra for memory 
    supersedes : Option<String>,
}

#[derive(Serialize)]
pub struct Out {
    id : String,
    agent : String,
    text : String,
}

pub async fn create( 
    Json(b) : Json<In> 
) -> Result<Json<Out>, AppError> {
    let out = Out{ 
        id : "123".to_string(),
        agent : b.agent,
        text : b.text,
    };
    Ok(Json(out))
}