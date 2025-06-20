use serde::Deserialize;

#[derive(Deserialize)]
pub struct Fragment {
    pub id: u64,
    pub file_name: String,
    pub code: String,
    pub language: String,
    pub position: u64,
}

#[derive(Deserialize)]
pub struct Snippet {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub fragments: Vec<Fragment>,
    pub updated_at: String,
    pub share_count: u64,
}