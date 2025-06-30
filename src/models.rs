use serde::Deserialize;

/// A code fragment within a snippet
#[derive(Deserialize, Debug, PartialEq)]
pub struct Fragment {
    pub id: u64,
    pub file_name: String,
    pub code: String,
    pub language: String,
    pub position: u64,
}

/// A complete code snippet with metadata and fragments
#[derive(Deserialize, Debug, PartialEq)]
pub struct Snippet {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub fragments: Vec<Fragment>,
    pub updated_at: String,
    pub share_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_fragment_deserialization() {
        let json = r#"{
            "id": 1,
            "file_name": "test.rs",
            "code": "fn main() {}",
            "language": "rust",
            "position": 0
        }"#;

        let fragment: Fragment = serde_json::from_str(json).unwrap();

        assert_eq!(fragment.id, 1);
        assert_eq!(fragment.file_name, "test.rs");
        assert_eq!(fragment.code, "fn main() {}");
        assert_eq!(fragment.language, "rust");
        assert_eq!(fragment.position, 0);
    }

    #[test]
    fn test_snippet_deserialization() {
        let json = r#"{
            "id": 42,
            "title": "Test Snippet",
            "description": "A test snippet",
            "categories": ["rust", "test"],
            "fragments": [
                {
                    "id": 1,
                    "file_name": "main.rs",
                    "code": "fn main() { println!(\"Hello!\"); }",
                    "language": "rust",
                    "position": 0
                }
            ],
            "updated_at": "2023-01-01T00:00:00Z",
            "share_count": 5
        }"#;

        let snippet: Snippet = serde_json::from_str(json).unwrap();

        assert_eq!(snippet.id, 42);
        assert_eq!(snippet.title, "Test Snippet");
        assert_eq!(snippet.description, "A test snippet");
        assert_eq!(snippet.categories, vec!["rust", "test"]);
        assert_eq!(snippet.fragments.len(), 1);
        assert_eq!(snippet.updated_at, "2023-01-01T00:00:00Z");
        assert_eq!(snippet.share_count, 5);

        let fragment = &snippet.fragments[0];
        assert_eq!(fragment.file_name, "main.rs");
        assert_eq!(fragment.language, "rust");
    }

    #[test]
    fn test_snippet_with_empty_fragments() {
        let json = r#"{
            "id": 1,
            "title": "Empty Snippet",
            "description": "",
            "categories": [],
            "fragments": [],
            "updated_at": "2023-01-01T00:00:00Z",
            "share_count": 0
        }"#;

        let snippet: Snippet = serde_json::from_str(json).unwrap();

        assert_eq!(snippet.fragments.len(), 0);
        assert_eq!(snippet.categories.len(), 0);
        assert!(snippet.description.is_empty());
    }
}
