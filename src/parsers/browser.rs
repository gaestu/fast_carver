use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BrowserHistoryRecord {
    pub run_id: String,
    pub browser: String,
    pub profile: String,
    pub url: String,
    pub title: Option<String>,
    pub visit_time: Option<chrono::NaiveDateTime>,
    pub visit_source: Option<String>,
    pub source_file: std::path::PathBuf,
}
