use iced::{
    Task,
    widget::text_editor::{self, Action, Edit},
};
use ropey::Rope;

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    index: usize,
    col: usize,
    row: usize,
}

#[derive(Debug, Clone, Copy)]
struct ViewportDetails {
    capacity: usize,
    start: usize,
}

#[derive(Debug, Clone)]
pub struct VirtualizedEditor {
    pub display_content: text_editor::Content,
    pub content_buffer: Rope,
    pub cursor_pos: CursorPosition,
    viewport_details: ViewportDetails,
}

impl Default for VirtualizedEditor {
    fn default() -> Self {
        Self {
            display_content: text_editor::Content::with_text(""),
            content_buffer: Rope::new(),
            cursor_pos: CursorPosition {
                index: 0,
                col: 0,
                row: 0,
            },
            viewport_details: ViewportDetails {
                capacity: 5000,
                start: 0,
            },
        }
    }
}

enum UpdatePos {
    Update(isize),
    Set(usize),
    Min(usize),
    Max(usize),
}

impl VirtualizedEditor {
    pub fn new(text: &str, line_char_capacity: usize) -> Self {
        let content_buffer = Rope::from_str(text);

        let mut editor = Self {
            display_content: text_editor::Content::new(),
            content_buffer,
            cursor_pos: CursorPosition {
                index: 0,
                col: 0,
                row: 0,
            },
            viewport_details: ViewportDetails {
                capacity: line_char_capacity,
                start: 0,
            },
        };

        editor.rebuild_display();
        editor
    }

    fn rebuild_display(&mut self) {
        let total_lines = self.content_buffer.len_lines();

        let mut display_text = String::new();

        for line_idx in 0..total_lines {
            let line = self.content_buffer.line(line_idx);
            let line_len = line.len_chars();

            // Get viewport offset for this specific line
            let line_offset = if line_idx == self.cursor_pos.row {
                self.viewport_details.start
            } else {
                0
            };

            // If long line, show only visible portion
            if line_len > self.viewport_details.capacity {
                let end = (line_offset + self.viewport_details.capacity).min(line_len);
                let slice = line.slice(line_offset..end);
                display_text.push_str(&slice.to_string());
            } else {
                display_text.push_str(&line.to_string());
            }
        }

        self.display_content = text_editor::Content::with_text(&display_text);
        self.position_cursor_in_display();
    }

    fn position_cursor_in_display(&mut self) {
        let display_row = self.cursor_pos.row;

        let current_line_len = if self.cursor_pos.row < self.content_buffer.len_lines() {
            self.content_buffer.line(self.cursor_pos.row).len_chars()
        } else {
            0
        };

        let display_col = if current_line_len > self.viewport_details.capacity {
            self.cursor_pos
                .col
                .saturating_sub(self.viewport_details.start)
        } else {
            self.cursor_pos.col
        };

        self.display_content.perform(text_editor::Action::Move(
            text_editor::Motion::DocumentStart,
        ));

        for _ in 0..display_row {
            self.display_content
                .perform(text_editor::Action::Move(text_editor::Motion::Down));
        }
        for _ in 0..display_col {
            self.display_content
                .perform(text_editor::Action::Move(text_editor::Motion::Right));
        }
    }

    fn adjust_line_viewport_offset(&mut self) {
        if self.cursor_pos.row >= self.content_buffer.len_lines() {
            return;
        }

        let line_len = self.content_buffer.line(self.cursor_pos.row).len_chars();

        // Only adjust if line is longer than capacity
        if line_len > self.viewport_details.capacity {
            let buffer_zone = self.viewport_details.capacity / 4;

            // If cursor is too close to left edge, scroll left
            if self.cursor_pos.col < self.viewport_details.start + buffer_zone {
                self.viewport_details.start = self.cursor_pos.col.saturating_sub(buffer_zone);
            }

            // If cursor is too close to right edge, scroll right
            if self.cursor_pos.col
                >= self.viewport_details.start + self.viewport_details.capacity - buffer_zone
            {
                self.viewport_details.start = (self.cursor_pos.col + buffer_zone)
                    .saturating_sub(self.viewport_details.capacity);
            }

            // Clamp offset
            self.viewport_details.start = self
                .viewport_details
                .start
                .min(line_len.saturating_sub(self.viewport_details.capacity));
        } else {
            self.viewport_details.start = 0;
        }
    }

    pub fn calculate_cursor_position(&self, iced_cursor: text_editor::Cursor) -> CursorPosition {
        let display_row = iced_cursor.position.line;
        let display_col = iced_cursor.position.column;

        let row = display_row;

        // If cursor is in the displayed content or outside
        if row < self.content_buffer.len_lines() {
            let line_len = self.content_buffer.line(row).len_chars();

            let col = if line_len > self.viewport_details.capacity {
                self.viewport_details.start + display_col
            } else {
                display_col
            };

            let line_start = self.content_buffer.line_to_char(row);
            let index = line_start + col;

            CursorPosition { index, col, row }
        } else {
            let total_lines = self.content_buffer.len_lines();
            let last_row = total_lines.saturating_sub(1);
            let last_col = self.content_buffer.line(last_row).len_chars();
            let last_index = self.content_buffer.len_chars();

            CursorPosition {
                index: last_index,
                col: last_col,
                row: last_row,
            }
        }
    }

    pub fn calculate_selection_range(
        &self,
        iced_cursor: text_editor::Cursor,
    ) -> Option<(CursorPosition, CursorPosition)> {
        iced_cursor.selection.map(|selection_pos| {
            let start = self.calculate_cursor_position(iced_cursor);

            // End position
            let display_row = selection_pos.line;
            let display_col = selection_pos.column;
            let row = display_row;

            let (col, index) = if row < self.content_buffer.len_lines() {
                let line_len = self.content_buffer.line(row).len_chars();
                let col = if line_len > self.viewport_details.capacity {
                    self.viewport_details.start + display_col
                } else {
                    display_col
                };
                let line_start = self.content_buffer.line_to_char(row);
                (col, line_start + col)
            } else {
                let total_lines = self.content_buffer.len_lines();
                let last_row = total_lines.saturating_sub(1);
                let last_col = self.content_buffer.line(last_row).len_chars();
                (last_col, self.content_buffer.len_chars())
            };

            let end = CursorPosition { index, col, row };

            (start, end)
        })
    }

    fn sync_cursor_from_display(&mut self) {
        let iced_cursor = self.display_content.cursor();
        self.cursor_pos = self.calculate_cursor_position(iced_cursor);
    }

    pub fn perform(&mut self, action: Action) -> Task<()> {
        match &action {
            Action::Edit(edit) => {
                self.handle_edit(edit.clone());
            }
            Action::Move(motion) => {
                self.handle_move(motion.clone());
            }
            Action::Select(_) => {
                self.display_content.perform(action);
                self.sync_cursor_from_display();
            }
            _ => {
                self.display_content.perform(action);
            }
        }

        Task::none()
    }

    fn update_row_col_index(&mut self, row_update_by: UpdatePos, col_update_by: UpdatePos) {
        self.cursor_pos.row = match row_update_by {
            UpdatePos::Update(x) => self.cursor_pos.row.saturating_add_signed(x),
            UpdatePos::Set(x) => x,
            UpdatePos::Min(x) => self.cursor_pos.row.min(x),
            UpdatePos::Max(x) => self.cursor_pos.row.max(x),
        };

        self.cursor_pos.col = match col_update_by {
            UpdatePos::Update(x) => self.cursor_pos.col.saturating_add_signed(x),
            UpdatePos::Set(x) => x,
            UpdatePos::Min(x) => self.cursor_pos.col.min(x),
            UpdatePos::Max(x) => self.cursor_pos.col.max(x),
        };

        self.update_index()
    }

    fn update_index(&mut self) {
        let row = self.cursor_pos.row;
        let col = self.cursor_pos.col;

        self.cursor_pos.index = if row < self.content_buffer.len_lines() {
            let line_len = self.content_buffer.line(row).len_chars();

            let col = if line_len > self.viewport_details.capacity {
                self.viewport_details.start + col
            } else {
                col
            };

            let line_start = self.content_buffer.line_to_char(row);
            line_start + col
        } else {
            self.content_buffer.len_chars()
        }
    }

    fn handle_move(&mut self, motion: text_editor::Motion) {
        let total_lines = self.content_buffer.len_lines();
        let line_len = if self.cursor_pos.row < total_lines {
            self.content_buffer.line(self.cursor_pos.row).len_chars()
        } else {
            0
        };

        println!(
            "motion: {:?}, cursor_pos {:?}, line_len {}, total_lines {}",
            motion, self.cursor_pos, line_len, total_lines
        );

        match motion {
            text_editor::Motion::Left => {
                if self.cursor_pos.col > 0 {
                    self.update_row_col_index(UpdatePos::Update(0), UpdatePos::Update(-1));
                } else if self.cursor_pos.row > 0 {
                    let new_line_len = self
                        .content_buffer
                        .line(self.cursor_pos.row - 1)
                        .len_chars();
                    let col = new_line_len.saturating_sub(1);
                    self.update_row_col_index(UpdatePos::Update(-1), UpdatePos::Set(col));
                }
            }
            text_editor::Motion::Right => {
                if self.cursor_pos.col < line_len {
                    self.update_row_col_index(UpdatePos::Update(0), UpdatePos::Update(1));
                } else if self.cursor_pos.row < total_lines - 1 {
                    self.update_row_col_index(UpdatePos::Update(1), UpdatePos::Set(0));
                }
            }
            text_editor::Motion::Up => {
                if self.cursor_pos.row > 0 {
                    let new_line_len = self
                        .content_buffer
                        .line(self.cursor_pos.row - 1)
                        .len_chars()
                        .saturating_sub(1);
                    self.update_row_col_index(UpdatePos::Update(-1), UpdatePos::Min(new_line_len));
                }
            }
            text_editor::Motion::Down => {
                if self.cursor_pos.row < total_lines - 1 {
                    let new_line = self.content_buffer.line(self.cursor_pos.row + 1);
                    let new_line_len =
                        if new_line.char(new_line.len_chars().saturating_sub(1)) == '\n' {
                            new_line.len_chars().saturating_sub(1)
                        } else {
                            new_line.len_chars()
                        };

                    self.update_row_col_index(UpdatePos::Update(1), UpdatePos::Min(new_line_len));
                }
            }
            text_editor::Motion::Home => {
                self.cursor_pos.col = 0;
                let line_start = self.content_buffer.line_to_char(self.cursor_pos.row);
                self.cursor_pos.index = line_start;
            }
            text_editor::Motion::End => {
                self.cursor_pos.col = line_len;
                let line_start = self.content_buffer.line_to_char(self.cursor_pos.row);
                self.cursor_pos.index = line_start + line_len;
            }
            text_editor::Motion::DocumentStart => {
                self.update_row_col_index(UpdatePos::Set(0), UpdatePos::Set(0));
            }
            text_editor::Motion::DocumentEnd => {
                self.cursor_pos.row = total_lines.saturating_sub(1);
                self.cursor_pos.col = self.content_buffer.line(self.cursor_pos.row).len_chars();
                self.cursor_pos.index = self.content_buffer.len_chars();
            }
            _ => {
                // Let display handle other motions
                self.display_content.perform(Action::Move(motion));
                self.sync_cursor_from_display();
                return;
            }
        }

        self.adjust_line_viewport_offset();
        self.rebuild_display();
    }

    fn handle_edit(&mut self, edit: Edit) {
        match edit {
            Edit::Insert(ch) => {
                self.content_buffer.insert_char(self.cursor_pos.index, ch);
                self.cursor_pos.index += 1;
                self.cursor_pos.col += 1;
            }
            Edit::Paste(text) => {
                let chars_added = text.chars().count();
                self.content_buffer.insert(self.cursor_pos.index, &text);

                // Handle newlines in pasted text
                if text.contains('\n') {
                    let newline_count = text.matches('\n').count();
                    self.cursor_pos.row += newline_count;

                    // Find column on new line
                    if let Some(last_newline_pos) = text.rfind('\n') {
                        self.cursor_pos.col = text[last_newline_pos + 1..].chars().count();
                    }
                } else {
                    self.cursor_pos.col += chars_added;
                }

                self.cursor_pos.index += chars_added;
            }
            Edit::Enter => {
                self.content_buffer.insert_char(self.cursor_pos.index, '\n');
                self.cursor_pos.index += 1;
                self.cursor_pos.row += 1;
                self.cursor_pos.col = 0;
            }
            Edit::Backspace => {
                if self.cursor_pos.index > 0 {
                    let deleted_char = self.content_buffer.char(self.cursor_pos.index - 1);
                    self.content_buffer
                        .remove(self.cursor_pos.index - 1..self.cursor_pos.index);
                    self.cursor_pos.index -= 1;

                    if deleted_char == '\n' && self.cursor_pos.row > 0 {
                        self.cursor_pos.row -= 1;
                        self.cursor_pos.col =
                            self.content_buffer.line(self.cursor_pos.row).len_chars();
                    } else {
                        self.cursor_pos.col = self.cursor_pos.col.saturating_sub(1);
                    }
                }
            }
            Edit::Delete => {
                if self.cursor_pos.index < self.content_buffer.len_chars() {
                    self.content_buffer
                        .remove(self.cursor_pos.index..self.cursor_pos.index + 1);
                }
            }
            Edit::Indent => {
                let cur_line = self.content_buffer.line(self.cursor_pos.row);
                let cur_line_chars = cur_line.len_chars();

                if cur_line_chars != 0 && cur_line.char(0) != '\n' {
                    self.content_buffer
                        .insert_char(self.cursor_pos.index - cur_line_chars, '\t');
                    self.cursor_pos.col += 1;
                    self.cursor_pos.index += 1;
                }
            }
            Edit::Unindent => {
                let cur_line = self.content_buffer.line(self.cursor_pos.row);
                let cur_line_chars = cur_line.len_chars();

                if cur_line_chars != 0 && cur_line.char(0) == '\t' {
                    let line_first_char_index = self.cursor_pos.index - cur_line_chars;

                    self.content_buffer
                        .remove(line_first_char_index..line_first_char_index + 1);
                    self.cursor_pos.col -= 1;
                    self.cursor_pos.index -= 1;
                }
            }
        }

        self.adjust_line_viewport_offset();
        self.rebuild_display();
    }
}

#[cfg(test)]
mod tests {
    use iced::Renderer;

    use super::*;

    #[test]
    fn test_cursor_calculation() {
        let text = "Line1\nLine2\nLine3";
        let virt_editor = VirtualizedEditor::new(text, 50);
        let content: text_editor::Content<Renderer> = text_editor::Content::with_text(text);

        let cursor_pos = virt_editor.calculate_cursor_position(content.cursor());

        assert_eq!(cursor_pos.row, 0);
        assert_eq!(cursor_pos.col, 0);
        assert_eq!(cursor_pos.index, 0);
    }

    fn test() {
        // Buffer::hit(&self, x, y)
    }

    #[test]
    fn test_cursor_calculation_10_lines() {
        let length = 10;
        let text_string = "short\n".to_string()
            + &"abc\n".repeat(3)
            + &"x".repeat(length)
            + &"abc\n".repeat(4)
            + "\nshort";
        let text = text_string.as_str();
        println!("{text}");

        let virt_editor = VirtualizedEditor::new(text, 50);
        let mut content: text_editor::Content<Renderer> = text_editor::Content::with_text(text);

        content.perform(text_editor::Action::Move(text_editor::Motion::Down));
        content.perform(text_editor::Action::Move(text_editor::Motion::Down));
        content.perform(text_editor::Action::Move(text_editor::Motion::Down));
        content.perform(text_editor::Action::Move(text_editor::Motion::Right));
        content.perform(text_editor::Action::Move(text_editor::Motion::Right));

        let cursor_pos = virt_editor.calculate_cursor_position(content.cursor());

        assert_eq!(cursor_pos.row, 3);
        assert_eq!(cursor_pos.col, 2);
        assert_eq!(cursor_pos.index, 16);
    }
}
