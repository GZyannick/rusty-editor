use crate::{theme::color_highligther::ColorHighligter, viewport::Viewport};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

// highlight the rust code with tree_sitter and tree_sitter_rust
pub fn highlight(viewport: &Viewport, code: &String) -> anyhow::Result<Vec<ColorHighligter>> {
    let mut colors: Vec<ColorHighligter> = vec![];
    let mut parser = Parser::new();

    let query = match &viewport.buffer.query_language {
        Some((query, language)) => {
            parser.set_language(language)?;
            query
        }
        None => return Ok(colors),
    };
    let tree = parser.parse(code, None).expect("tree_sitter couldnt parse");
    let mut query_cursor = QueryCursor::new();
    let mut query_matches = query_cursor.matches(query, tree.root_node(), code.as_bytes());
    while let Some(m) = query_matches.next() {
        for cap in m.captures {
            let node = cap.node;
            let punctuation = &query.capture_names()[cap.index as usize];

            colors.push(ColorHighligter::new_from_capture(
                node.start_byte(),
                node.end_byte(),
                punctuation,
            ))
        }
    }
    Ok(colors)
}

#[cfg(test)]
mod test_highlighting {
    use super::*;

    #[test]
    fn test_highlight() {
        // Création d'un Viewport par défaut
        let viewport = Viewport::default();

        // Code simple pour le test
        let code = r#"
            fn main() {
                let x = 42;
                println!("{}", x);
            }
        "#
        .to_string();

        // Appel de la fonction highlight
        let result = highlight(&viewport, &code);

        // Vérification des résultats
        assert!(result.is_ok());
        let colors = result.unwrap();

        // Vérification qu'il y a des éléments dans le vecteur de surbrillance
        assert!(!colors.is_empty());

        // Optionnel: tester des couleurs spécifiques ou des positions
        for color in colors {
            // On peut ajouter des vérifications supplémentaires ici selon le comportement attendu
            assert!(color.start > 0);
            assert!(color.end >= color.start);
        }
    }
}
