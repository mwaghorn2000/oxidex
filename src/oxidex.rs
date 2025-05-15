use std::{collections::HashMap, fs, path::PathBuf};

use crate::document::DocumentEntry;

type Token = String;

pub enum OxidexError {
    AddDocumentError(String),
}

pub struct Oxidex {
    documents: HashMap<usize, DocumentEntry>,
    inverted_index: HashMap<Token, HashMap<usize, u32>>,
    next_idx: usize,
}

pub struct SearchResult {
    pub doc_id: usize,
    pub score: f32,
}

impl Oxidex {
    pub fn new() -> Self {
        Oxidex {
            documents: HashMap::new(),
            inverted_index: HashMap::new(),
            next_idx: 0,
        }
    }

    pub fn add_document(&mut self, path: PathBuf) -> Result<(), OxidexError> {
        let raw_bytes =
            fs::read(&path).map_err(|e| OxidexError::AddDocumentError(e.to_string()))?;
        // Todo:
        // Implement parsers for different file types. At the moment we just convert a byte vector into a String
        let parsed_content = String::from_utf8_lossy(&raw_bytes).into_owned();
        let tokens: Vec<Token> = parsed_content
            .split_ascii_whitespace()
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        for token in tokens.clone() {
            self.inverted_index
                .entry(token)
                .or_default()
                .entry(self.next_idx)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        let doc_entry = DocumentEntry::new(self.next_idx, path, tokens.len())
            .map_err(|e| OxidexError::AddDocumentError(e.to_string()))?;

        self.documents.insert(self.next_idx, doc_entry);
        self.next_idx += 1;
        Ok(())
    }

    /// Removes the document from Oxidex, by id.
    pub fn remove_id(&mut self, id: usize) -> bool {
        let existed = self.documents.remove(&id).is_some();

        for (_, doc_freq_map) in self.inverted_index.iter_mut() {
            doc_freq_map.remove(&id);
        }
        self.inverted_index
            .retain(|_, doc_freq_map| !doc_freq_map.is_empty());
        existed
    }

    pub fn get_doc(&self, doc_id: usize) -> Option<&DocumentEntry> {
        self.documents.get(&doc_id)
    }

    pub fn search(&self, query: Token) -> Vec<SearchResult> {
        // Need to build a query.
        // Using the formula sum(TF(t, d)* IDF(t)) for all t in Q, we can get the score
        // of a file based on the search query.
        let mut search_results: Vec<SearchResult> = Vec::new();

        if let Some(data) = self.inverted_index.get(&query) {
            for doc_id in data.keys() {
                search_results.push(SearchResult {
                    doc_id: *doc_id,
                    score: self.get_normalised_tf_idf(&query, *doc_id),
                });
            }
        }

        search_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        search_results
    }

    fn term_frequency(&self, token: &Token, id: usize) -> f32 {
        self.inverted_index
            .get(token)
            .and_then(|entry| entry.get(&id))
            .copied()
            .unwrap_or(0) as f32
    }

    fn inverse_document_frequency(&self, token: &Token) -> f32 {
        let df_t = self
            .inverted_index
            .get(token)
            .map(|inner| inner.len())
            .unwrap_or(0) as f32;

        let n = self.documents.len() as f32;

        (n / (df_t + 1.0)).log10()
    }

    fn get_tf_idf(&self, token: &Token, id: usize) -> f32 {
        self.term_frequency(token, id) * self.inverse_document_frequency(token)
    }

    fn get_normalised_tf_idf(&self, token: &Token, id: usize) -> f32 {
        let tf_idf = self.get_tf_idf(token, id);

        let len = self
            .documents
            .get(&id)
            .map(|doc| doc.token_count)
            .unwrap_or(1) as f32;

        tf_idf / len.sqrt()
    }
}

impl Default for Oxidex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> std::io::Result<Self> {
            let path = Path::new("tmp_test").join(name);
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
    
    #[test]
    fn test_with_raii_temp() -> std::io::Result<()> {
        let dir = TestDir::new("test_with_raii_temp")?;
        let file_path = dir.path().join("foo.txt");

        fs::write(&file_path, "data")?;
        let content = fs::read_to_string(&file_path)?;
        assert_eq!(content, "data");

        Ok(())
    }
}
