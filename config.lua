return {
	keybinds = {
		normal = {
			{ key = "v", action = "EnterMode Normal", description = "Switches to Visual mode." },
			{
				key = "zz",
				action = "CenterLine",
				description = "allow us to center the line",
			},

			{
				key = "<leader>-",
				action = "SwapViewportToExplorer",
				description = "swap between viewport and file explorer",
			},
			{
				key = "<leader>e",
				action = "SwapViewportToPopupExplorer",
				description = "swap between viewport and file explorer popup",
			},
			{ key = "u", action = "Undo", description = "Reverts the last performed action." },
			{ key = ":", action = "EnterMode Command", description = "Switches to Command mode." },
			{ key = "p", action = "Past", description = "Pastes previously copied text." },
			{
				key = "Esc",
				action = "ClearToNormalMode",
				description = "Returns to Normal mode and clears the current state.",
			},

			-- Search Actions
			{ key = "/", action = "EnterMode Search", description = "Switches to Search mode." },
			{ key = "n", action = "IterNextSearch", description = "Jumps to the next search occurrence." },

			-- Insert Actions
			{ key = "i", action = "EnterMode Insert", description = "Switches to Insert mode." },
			{ key = "a", action = "EnterMode Insert", description = "Switches to Insert mode." },

			-- Delete Actions
			-- { key = "x", action = "RemoveCharAt", description = "Deletes a character at a specific position." },
			{
				key = "dd",
				action = "DeleteLine",
				description = "DeleteLine on cursor",
			},
			{
				key = "dw",
				action = "DeleteWord",
				description = "DeleteWord on cursor",
			},
			-- Create line Actions
			{
				key = "o",
				action = "NewLineInsertionBelowCursor",
				description = "Inserts a new line below the cursor and enters Insert mode.",
			},

			-- Yank Actions
			{
				key = "yy",
				action = "YankLine",
				description = "copy the line on cursor",
			},

			-- Movement Actions
			{ key = "PageUp", action = "PageUp", description = "Scrolls up by one page." },
			{ key = "PageDown", action = "PageDown", description = "Scrolls down by one page." },

			{
				key = "gg",
				action = "StartOfFile",
				description = "return at the start of the file",
			},
			{ key = "$", action = "EndOfFile", description = "Moves the cursor to the end of the file." },

			{
				key = "0",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
			},
			{
				key = "Home",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
			},
			{ action = "EndOfFile", key = "G", description = "Moves the cursor to the end of the file." },
		},
		visual = {
			{ key = "Esc", action = "EnterMode Normal", description = "Switches to Normal mode." },
			{ key = ":", action = "EnterMode Command", description = "Switches to Command mode." },

			-- Actions
			{ key = "d", action = "DeleteBlock", description = "Deletes a selected block of text." },
			{ key = "y", action = "YankBlock", description = "Copies a selected block of text." },

			-- Movement Actions
			{ key = "PageUp", action = "PageUp", description = "Scrolls up by one page." },
			{ key = "PageDown", action = "PageDown", description = "Scrolls down by one page." },
			{ key = "G", action = "EndOfFile", description = "Moves the cursor to the end of the file." },
			{
				key = "gg",
				action = "WaitingCmd g",
				description = "Waits for a second key press to execute a complex command.",
			},
			{ key = "$", action = "EndOfLine", description = "Moves the cursor to the end of the current line." },
			{ key = "End", action = "EndOfLine", description = "Moves the cursor to the end of the current line." },
			{
				key = "0",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
			},
			{
				key = "Home",
				action = "StartOfLine",
				description = "Moves the cursor to the beginning of the current line.",
			},
		},

		insert = {
			{ key = "Esc", action = "EnterMode Normal", description = "Switches to Normal mode." },
			{ key = "Backspace", action = "RemoveChar", description = "Deletes the character before the cursor." },
			{ key = "Return", action = "NewLine", description = "Inserts a new line below the current line." },
			{
				key = "Tab",
				action = "AddStr /space/space",
				description = "Adds a string of text at the cursor position.",
			},
		},

		command = {
			{ key = "Return", action = "EnterMode Normal", description = "Switches to Normal mode." },
			{
				key = "Esc",
				action = "ClearToNormalMode",
				description = "Returns to Normal mode and clears the current state.",
			},
			{
				key = "Backspace",
				action = "RemoveCharFrom false",
				description = "Deletes a character based on context (e.g., search or command).",
			},
			{ key = "Return", action = "ExecuteCommand", description = "Executes the entered command." },
		},
	},
}
