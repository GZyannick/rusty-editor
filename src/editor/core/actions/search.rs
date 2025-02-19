use std::io::Write;

use super::action::Action;
use crate::editor::{core::mode::Mode, Editor};

impl Action {
    pub fn search<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            // allow us to clear search string
            Action::ClearToNormalMode => {
                let current_viewport = editor.viewports.c_mut_viewport();
                current_viewport.clear_search();
                editor.search = String::new();
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }

            // research correspondng value in file when editor.search got updated
            Action::FindSearchValue => {
                let current_viewport = editor.viewports.c_mut_viewport();
                current_viewport.find_occurence(&editor.search);

                if let Some(cursor) = current_viewport.search_pos.first() {
                    let goto_cursor = (cursor.0, cursor.1);
                    editor.buffer_actions.push(Action::GotoPos(goto_cursor))
                }
            }

            Action::IterNextSearch => {
                // iter through the list of search
                let current_viewport = editor.viewports.c_mut_viewport();
                match current_viewport.search_index < current_viewport.search_pos.len() {
                    true => current_viewport.search_index += 1,
                    false => current_viewport.search_index = 0,
                }

                if let Some(cursor) = current_viewport
                    .search_pos
                    .get(current_viewport.search_index)
                {
                    let goto_cursor = (cursor.0, cursor.1);
                    editor.buffer_actions.push(Action::GotoPos(goto_cursor));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_search {
    use tempfile::NamedTempFile;

    use crate::{
        buff::Buffer,
        editor::{
            core::{actions::action::Action, mode::Mode},
            Editor,
        },
    };
    use std::io::{Cursor, Seek, Write};

    fn setup_temp_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().expect("Failed to create a temporary_file");
        let content =
            "This is a test file with multiple line.\nHere is a keyword we will search.\nAnother line with keyword.";
        temp_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        // return the cursor at 0:0
        temp_file
            .seek(std::io::SeekFrom::Start(0))
            .expect("Failed to seek temp file");
        temp_file
    }

    fn create_mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::default()
    }
    #[test]
    fn test_clear_to_normal_mode() {
        let mut editor = create_mock_editor();
        editor.mode = Mode::Search;
        editor.search = "Some search".to_string();
        Action::ClearToNormalMode.execute(&mut editor).unwrap();
        assert!(editor.search.is_empty(), "Search string should be Empty");
        assert!(
            matches!(editor.mode, Mode::Normal),
            "Search string should be Empty"
        );
    }

    #[test]
    fn test_find_search_value() {
        let tmp_file = setup_temp_file();
        let path = tmp_file.path().to_str().unwrap().to_string();
        let mut editor = create_mock_editor();
        let old_cursor = editor.cursor;
        editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(path));

        editor.search = "line".to_string();
        editor.mode = Mode::Search;

        Action::FindSearchValue.execute(&mut editor).unwrap();
        assert!(
            !editor.viewports.c_mut_viewport().search_pos.is_empty(),
            "search_pos should not be empty"
        );
        assert!(
            editor.viewports.c_mut_viewport().search_pos.len() == 2,
            "search_pos should have 2 occurences"
        );

        assert!(
            editor.cursor != old_cursor,
            "search_pos should have 2 occurences"
        );
    }

    #[test]
    fn test_iter_next_search() {
        let tmp_file = setup_temp_file();
        let path = tmp_file.path().to_str().unwrap().to_string();
        let mut editor = create_mock_editor();
        editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(path));

        editor.search = "line".to_string();
        editor.mode = Mode::Search;

        Action::FindSearchValue.execute(&mut editor).unwrap();
        assert!(
            !editor.viewports.c_mut_viewport().search_pos.is_empty(),
            "search_pos should not be empty"
        );
        assert!(
            editor.viewports.c_mut_viewport().search_index == 0,
            "search_index should be 0"
        );

        let old_cursor = editor.cursor;
        Action::IterNextSearch.execute(&mut editor).unwrap();
        assert!(
            editor.viewports.c_mut_viewport().search_index == 1,
            "search_index should be 1"
        );
        assert!(
            editor.cursor != old_cursor,
            "cursor should be different after iterating"
        );
    }
}
