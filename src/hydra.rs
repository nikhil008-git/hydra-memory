use reqwest::Client;

#[derive(Clone)] // Axum's application state needs to be clonable when we eventually do: axum::extract::State(hydra)
pub struct Hydra {
    http: Client,
    base: String,
    token: String,
    ns: String,
    graph: String,
    cell: String,
}

impl Hydra {
    pub fn from_env() -> Self {
        Self {
            http: Client::new(),

            base: std::env::var("HYDRA_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:18443".into()),

            token: std::env::var("HYDRA_TOKEN")
                .expect("HYDRA_TOKEN"),

            ns: std::env::var("HYDRA_NAMESPACE")
                .unwrap_or_else(|_| "local".into()),

            graph: std::env::var("HYDRA_GRAPH")
                .unwrap_or_else(|_| "default".into()),

            cell: std::env::var("HYDRA_CELL")
                .unwrap_or_else(|_| "cell-0".into()),
        }
    }
}