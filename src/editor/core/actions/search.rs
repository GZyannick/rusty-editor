use super::action::Action;
use crate::editor::{core::mode::Mode, Editor};

impl Action {
    pub fn search(&self, editor: &mut Editor) -> anyhow::Result<()> {
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
                    editor.buffer_actions.push(Action::GotoPos(*cursor))
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
                    editor.buffer_actions.push(Action::GotoPos(*cursor));
                }
            }
            _ => {}
        }

        Ok(())
    }
}
