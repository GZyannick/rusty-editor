use std::collections::HashMap;

use tree_sitter::Language;

#[derive(Debug, Clone)]
pub struct Languages {
    languages: HashMap<String, (Language, String)>,
}

impl Languages {
    pub fn new() -> Self {
        Self {
            languages: Self::init(),
        }
    }

    pub fn get(&self, path: &str) -> Option<&(Language, String)> {
        let name = Self::get_file_extension(path).unwrap_or_default();
        self.languages.get(&name)
    }

    fn get_file_extension(path: &str) -> Option<String> {
        std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str().map(|str| str.to_string()))
    }
    fn init() -> HashMap<String, (Language, String)> {
        let mut languages = HashMap::new();
        languages.insert(
            "rs".to_string(),
            (
                tree_sitter_rust::LANGUAGE.into(),
                tree_sitter_rust::HIGHLIGHTS_QUERY.to_string(),
            ),
        );
        languages.insert(
            "lua".to_string(),
            (
                tree_sitter_lua::LANGUAGE.into(),
                tree_sitter_lua::HIGHLIGHTS_QUERY.to_string(),
            ),
        );
        languages.insert(
            "html".to_string(),
            (
                tree_sitter_html::LANGUAGE.into(),
                tree_sitter_html::HIGHLIGHTS_QUERY.to_string(),
            ),
        );
        languages.insert(
            "css".to_string(),
            (
                tree_sitter_css::LANGUAGE.into(),
                tree_sitter_css::HIGHLIGHTS_QUERY.to_string(),
            ),
        );
        languages.insert(
            "js".to_string(),
            (
                tree_sitter_javascript::LANGUAGE.into(),
                tree_sitter_javascript::HIGHLIGHT_QUERY.to_string(),
            ),
        );
        languages.insert(
            "rb".to_string(),
            (
                tree_sitter_ruby::LANGUAGE.into(),
                tree_sitter_ruby::HIGHLIGHTS_QUERY.to_string(),
            ),
        );
        languages.insert(
            "py".to_string(),
            (
                tree_sitter_python::LANGUAGE.into(),
                tree_sitter_python::HIGHLIGHTS_QUERY.to_string(),
            ),
        );

        languages
    }
}

#[cfg(test)]
mod tests_languages {
    use super::*;

    #[test]
    fn test_get_file_extension() {
        let cases = vec![
            ("./src/editor/mod.rs", Some("rs".to_string())),
            ("./home/viewport/test.json", Some("json".to_string())),
            ("/absolute/path/to/file.txt", Some("txt".to_string())),
            ("no_extension_file", None),
            ("/another.path/file.tar.gz", Some("gz".to_string())), // Vérifie la dernière extension
            ("./somefile.TXT", Some("TXT".to_string())),           // Gère les majuscules
            ("./hiddenfile.", Some("".to_string())),               // Aucun caractère après le point
            ("./double..dotfile", Some("dotfile".to_string())),    // Cas spécial avec double point
        ];

        for (path, expected) in cases {
            assert_eq!(
                Languages::get_file_extension(path),
                expected,
                "Failed for path: {}",
                path
            );
        }
    }
}
