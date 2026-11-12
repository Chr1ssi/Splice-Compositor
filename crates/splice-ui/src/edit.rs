//! A line of text being typed into (the search field of the launcher, the calendar's form fields).

use crate::{Measure, TextStyle};

/// A line of text being edited, with the cursor as a byte offset at a character boundary. The fields are
/// private so that this always holds: the text is cut at the cursor while drawing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Input {
    text: String,
    cursor: usize,
}

impl Input {
    pub fn new(text: &str) -> Self {
        Self { text: text.to_owned(), cursor: text.len() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The text left of the cursor.
    pub fn before_cursor(&self) -> &str {
        &self.text[..self.cursor]
    }

    /// Puts the cursor at `cursor`, or at the character boundary before it.
    pub fn set_cursor(&mut self, cursor: usize) {
        let mut cursor = cursor.min(self.text.len());
        while !self.text.is_char_boundary(cursor) {
            cursor -= 1;
        }
        self.cursor = cursor;
    }

    pub fn insert(&mut self, text: &str) {
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    fn previous(&self) -> usize {
        self.text[..self.cursor].char_indices().next_back().map_or(0, |(i, _)| i)
    }

    fn next(&self) -> usize {
        self.text[self.cursor..].chars().next().map_or(self.cursor, |c| self.cursor + c.len_utf8())
    }

    pub fn backspace(&mut self) {
        let start = self.previous();
        self.text.replace_range(start..self.cursor, "");
        self.cursor = start;
    }

    pub fn delete(&mut self) {
        let end = self.next();
        self.text.replace_range(self.cursor..end, "");
    }

    pub fn left(&mut self) {
        self.cursor = self.previous();
    }

    pub fn right(&mut self) {
        self.cursor = self.next();
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }

    /// The cursor position nearest to `x`, measured from the start of the text drawn in `style`.
    pub fn nearest(&self, measure: &mut impl Measure, style: TextStyle, x: f32) -> usize {
        let gaps = self.text.char_indices().map(|(i, _)| i).chain([self.text.len()]);
        let distances = gaps.map(|end| (end, (measure.width(&self.text[..end], style) - x).abs()));
        distances.min_by(|a, b| a.1.total_cmp(&b.1)).map_or(0, |(end, _)| end)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_moves_by_characters() {
        let mut input = Input::new("Zahnärzt");
        input.left();
        input.left();
        input.left();
        input.backspace();
        assert_eq!((input.text.as_str(), input.cursor), ("Zahnrzt", 4), "the ä goes as one");
        input.insert("ä");
        input.delete();
        assert_eq!((input.text.as_str(), input.cursor), ("Zahnäzt", 6));
        input.home();
        input.backspace();
        input.right();
        assert_eq!(input.cursor, 1);
        input.end();
        input.delete();
        assert_eq!(input.cursor, input.text.len());
    }

    #[test]
    fn the_cursor_only_stands_between_characters() {
        let mut input = Input::new("aä");
        input.set_cursor(2);
        assert_eq!((input.cursor(), input.before_cursor()), (1, "a"), "not inside the ä");
        input.set_cursor(99);
        assert_eq!(input.before_cursor(), "aä");
    }
}
