-- ALL KeyModifiers ---
-- Shift Control Option Command Hyper Meta

return {
	keybinds = {
		normal = {
			{
				key = "x",
				action = "RemoveCharAt",
				description = "Deletes a character at a specific position.",
				modifiers = "",
			},
			{
				key = "<leader>bd",
				action = "DeleteViewport",
				description = "Delete current viewport",
				modifiers = "",
			},
			{
				key = "<leader>bo",
				action = "DeleteOtherViewport",
				description = "Delete other viewport",
				modifiers = "",
			},
			{
				key = "H",
				action = "PrevViewport",
				description = "Switch to the previous viewport",
				modifiers = "Shift",
			},
			{
				key = "L",
				action = "NextViewport",
				description = "Switch to the next viewport",
				modifiers = "Shift",
			},
			{ key = "v", action = "EnterMode Visual", description = "Switches to Visual mode.", modifiers = "" },
			{ key = "h", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "j", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "k", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "l", action = "MoveRight", description = "Move right by 1", modifiers = "" },

			{ key = "Left", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "Down", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "Up", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "Right", action = "MoveRight", description = "Move right by 1", modifiers = "" },

			{ key = "w", action = "MoveNext", description = "Move to the next different char", modifiers = "" },
			{ key = "b", action = "MovePrev", description = "Move to the prev different char", modifiers = "" },
			{
				key = "zz",
				action = "CenterLine",
				description = "allow us to center the line",
				modifiers = "",
			},

			{
				key = "<leader>-",
				action = "SwapViewportToExplorer",
				description = "swap between viewport and file explorer",
				modifiers = "",
			},
			{
				key = "<leader>e",
				action = "SwapViewportToPopupExplorer",
				description = "swap between viewport and file explorer popup",
				modifiers = "",
			},
			{ key = "u", action = "Undo", description = "Reverts the last performed action.", modifiers = "" },
			{ key = ":", action = "EnterMode Command", description = "Switches to Command mode.", modifiers = "" },
			{ key = "p", action = "Past", description = "Pastes previously copied text.", modifiers = "" },
			{
				key = "Esc",
				action = "ClearToNormalMode",
				description = "Returns to Normal mode and clears the current state.",
				modifiers = "",
			},

			-- Search Actions
			{ key = "/", action = "EnterMode Search", description = "Switches to Search mode.", modifiers = "" },
			{
				key = "n",
				action = "IterNextSearch",
				description = "Jumps to the next search occurrence.",
				modifiers = "",
			},

			-- Insert Actions
			{ key = "i", action = "EnterInsertMode", description = "Switches to Insert mode.", modifiers = "" },
			{ key = "a", action = "AppendInsertMode", description = "Switches to Insert mode.", modifiers = "" },

			-- Delete Actions
			-- { key = "x", action = "RemoveCharAt", description = "Deletes a character at a specific position.", modifiers = "" },
			{
				key = "dd",
				action = "DeleteLine",
				description = "DeleteLine on cursor",
				modifiers = "",
			},
			{
				key = "dw",
				action = "DeleteWord",
				description = "DeleteWord on cursor",
				modifiers = "",
			},
			-- Create line Actions
			{
				key = "o",
				action = "NewLineInsertionBelowCursor",
				description = "Inserts a new line below the cursor and enters Insert mode.",
				modifiers = "",
			},
			{
				key = "O",
				action = "NewLineInsertionAtCursor",
				description = "Inserts a new line at the cursor and enters Insert mode.",
				modifiers = "Shift",
			},

			-- Yank Actions
			{
				key = "yy",
				action = "YankLine",
				description = "copy the line on cursor",
				modifiers = "",
			},

			-- Movement Actions
			{ key = "Page Up", action = "PageUp", description = "Scrolls up by one page.", modifiers = "" },
			{ key = "Page Down", action = "PageDown", description = "Scrolls down by one page.", modifiers = "" },
			{ key = "b", action = "PageUp", description = "Scrolls up by one page.", modifiers = "Control" },
			{ key = "f", action = "PageDown", description = "Scrolls down by one page.", modifiers = "Control" },

			{
				key = "gg",
				action = "StartOfFile",
				description = "return at the start of the file",
				modifiers = "",
			},
			{
				key = "$",
				action = "EndOfLine",
				description = "Moves the cursor to the end of the file.",
				modifiers = "",
			},

			{
				key = "0",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
				modifiers = "",
			},
			{
				key = "Home",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
				modifiers = "",
			},
			{
				action = "EndOfFile",
				key = "G",
				description = "Moves the cursor to the end of the file.",
				modifiers = "Shift",
			},
		},
		visual = {
			{ key = "h", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "j", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "k", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "l", action = "MoveRight", description = "Move right by 1", modifiers = "" },

			{ key = "Left", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "Down", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "Up", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "Right", action = "MoveRight", description = "Move right by 1", modifiers = "" },

			{ key = "w", action = "MoveNext", description = "Move to the next different char", modifiers = "" },
			{ key = "b", action = "MovePrev", description = "Move to the prev different char", modifiers = "" },

			{ key = "Esc", action = "EnterMode Normal", description = "Switches to Normal mode.", modifiers = "" },
			{ key = ":", action = "EnterMode Command", description = "Switches to Command mode.", modifiers = "" },

			-- Actions
			{ key = "d", action = "DeleteBlock", description = "Deletes a selected block of text.", modifiers = "" },
			{ key = "y", action = "YankBlock", description = "Copies a selected block of text.", modifiers = "" },

			-- Movement Actions
			{ key = "Page Up", action = "PageUp", description = "Scrolls up by one page.", modifiers = "" },
			{ key = "Page Down", action = "PageDown", description = "Scrolls down by one page.", modifiers = "" },
			{
				key = "G",
				action = "EndOfFile",
				description = "Moves the cursor to the end of the file.",
				modifiers = "Shift",
			},
			{
				key = "gg",
				action = "StartOfFile",
				description = "return at the start of the file",
				modifiers = "",
			},
			{
				key = "$",
				action = "EndOfLine",
				description = "Moves the cursor to the end of the current line.",
				modifiers = "",
			},
			{
				key = "End",
				action = "EndOfLine",
				description = "Moves the cursor to the end of the current line.",
				modifiers = "",
			},
			{
				key = "0",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
				modifiers = "",
			},
			{
				key = "Home",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
				modifiers = "",
			},
		},

		insert = {

			{ key = "Left", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "Down", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "Up", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "Right", action = "MoveRight", description = "Move right by 1", modifiers = "" },

			{ key = "Esc", action = "EnterMode Normal", description = "Switches to Normal mode.", modifiers = "" },
			{
				key = "Delete",
				action = "RemoveChar",
				description = "Deletes the character before the cursor.",
				modifiers = "",
			},
			{
				key = "Return",
				action = "NewLine",
				description = "Inserts a new line below the current line.",
				modifiers = "",
			},
			{
				key = "Tab",
				action = "AddStr /space/space",
				description = "Adds a string of text at the cursor position.",
				modifiers = "",
			},
		},

		command = {
			{ key = "Return", action = "EnterMode Normal", description = "Switches to Normal mode.", modifiers = "" },
			{
				key = "Esc",
				action = "ClearToNormalMode",
				description = "Returns to Normal mode and clears the current state.",
				modifiers = "",
			},
			{
				key = "Delete",
				action = "RemoveCharFrom false",
				description = "Deletes a character based on context (e.g., search or command).",
				modifiers = "",
			},
			{
				key = "Return",
				action = "ExecuteCommand",
				description = "Executes the entered command.",
				modifiers = "",
			},
		},

		search = {
			{
				key = "Esc",
				action = "ClearToNormalMode",
				description = "Returns to Normal mode and clears the current state.",
				modifiers = "",
			},
			{
				key = "Delete",
				action = "RemoveCharFrom true",
				description = "Deletes a character based on context (e.g., search or command).",
				modifiers = "",
			},
			{ key = "Return", action = "EnterMode Normal", description = "Switches to Normal mode.", modifiers = "" },
		},

		file_explorer = {
			{ key = "h", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "j", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "k", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "l", action = "MoveRight", description = "Move right by 1", modifiers = "" },

			{ key = "Left", action = "MoveLeft", description = "Move left by 1", modifiers = "" },
			{ key = "Down", action = "MoveDown", description = "Move down by 1", modifiers = "" },
			{ key = "Up", action = "MoveUp", description = "Move up by 1", modifiers = "" },
			{ key = "Right", action = "MoveRight", description = "Move right by 1", modifiers = "" },
			{
				key = "<leader>-",
				action = "SwapViewportToExplorer",
				description = "swap between viewport and file explorer",
				modifiers = "",
			},
			{
				key = "<leader>e",
				action = "SwapViewportToPopupExplorer",
				description = "swap between viewport and file explorer popup",
				modifiers = "",
			},
			{
				key = "Return",
				action = "EnterFileOrDirectory",
				description = "Opens a file or enters a directory in the file explorer.",
				modifiers = "",
			},
			{
				key = "-",
				action = "GotoParentDirectory",
				description = "Moves up to the parent directory in the file explorer.",
				modifiers = "",
			},
			{
				key = "d",
				action = "DeleteInputModal",
				description = "Opens a dialog to confirm file or directory deletion.",
				modifiers = "",
			},
			{
				key = "r",
				action = "RenameInputModal",
				description = "Opens a dialog to rename a file or directory.",
				modifiers = "",
			},
			{
				key = "a",
				action = "CreateInputModal",
				description = "Opens a dialog to create a new file or directory.",
				modifiers = "",
			},
			{
				key = "i",
				action = "CreateInputModal",
				description = "Opens a dialog to create a new file or directory.",
				modifiers = "",
			},
			{
				key = "G",
				action = "EndOfFile",
				description = "Moves the cursor to the end of the file.",
				modifiers = "Shift",
			},
			{
				key = "gg",
				action = "StartOfFile",
				description = "return at the start of the file",
				modifiers = "",
			},
		},
	},
}
