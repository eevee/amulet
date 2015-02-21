use std::str;
use std::vec;
use std::rc::Rc;

use ll::{Key,Style};  // TODO move these somewhere dealing with keys and text and terminal properties
use ll::TerminalInfo;

struct CanvasCell {
    dirty: bool,
    glyph: char,
    style: Style,
}

struct CanvasRow {
    is_dirty: bool,
    last_dirty: usize,
    first_dirty: usize,
    cells: Vec<CanvasCell>,
}

pub struct Canvas<'a, 'b> {
    terminfo: &'b TerminalInfo<'b>,
    start_row: usize,
    start_col: usize,
    cur_row: usize,
    cur_col: usize,
    height: usize,
    width: usize,

    rows: Vec<CanvasRow>,
    pub guards: Vec<Box<Drop + 'a>>,
}

pub fn Canvas<'a, 'b>(terminfo: &'b TerminalInfo<'b>, start_row: usize, start_col: usize, height: usize, width: usize) -> Canvas<'a, 'b> {
    let rows = range(0, height).map(|_row| {
        CanvasRow{
            is_dirty: false,
            last_dirty: 0,
            first_dirty: 0,
            cells: range(0, width).map(|_col| {
                CanvasCell{
                    dirty: false,
                    glyph: ' ',
                    style: Style(),
                }
            }).collect(),
        }
    }).collect();
    return Canvas{
        terminfo: terminfo,

        start_row: start_row,
        start_col: start_col,
        cur_row: 0,
        cur_col: 0,
        height: height,
        width: width,

        rows: rows,
        guards: vec![],
    };
}

impl<'a, 'b> Canvas<'a, 'b> {
    // -------------------------------------------------------------------------
    // Creation

    pub fn spawn(&self, start_row: usize, start_col: usize, height: usize, width: usize) -> Canvas<'a, 'b> {
        // TODO verify new height/width will fit?  or don't?  at least verify
        // h/w aren't negative or zero
        let real_height;
        if height == 0 {
            real_height = self.height - start_row;
        }
        else {
            real_height = height;
        }

        let real_width;
        if width == 0 {
            real_width = self.width - start_col;
        }
        else {
            real_width = width;
        }

        panic!("todo not done yet");
    }

    // -------------------------------------------------------------------------
    // Accessors

    /** Returns the size of the canvas as (rows, columns). */
    pub fn size(&self) -> (usize, usize) {
        return (self.height, self.width);
    }

    pub fn position(&self) -> (usize, usize) {
        return (self.cur_row, self.cur_col);
    }

    pub fn reposition(&mut self, row: usize, col: usize) {
        self.cur_row = row;
        self.cur_col = col;
    }


    // -------------------------------------------------------------------------
    // Output

    pub fn clear(&mut self) {
        // TODO clearing the screen can be done with a single termcap, but how
        // do i remember that
        for row in self.rows.iter_mut() {
            row.is_dirty = true;
            row.last_dirty = self.width - 1;
            row.first_dirty = 0;
            for cell in row.cells.iter_mut() {
                *cell = CanvasCell{
                    dirty: true,
                    glyph: ' ',
                    style: Style(),
                };
            }
        }
    }

    pub fn attrwrite(&mut self, s: &str, style: Style) {
        for glyph in s.chars() {
            if glyph == '\n' {
                // TODO this probably needs (a) more cases, (b) termcap
                // influence
                self.cur_row += 1;
                self.cur_col = 0;

                if self.cur_row >= self.height {
                    panic!("TODO");
                }

                // TODO is there ever anything to write here?
                return;
            }

            {
                let row = &mut self.rows[self.cur_row];
                row.cells[self.cur_col] = CanvasCell{
                    dirty: true,
                    glyph: glyph,
                    style: style.clone(),
                };
                row.is_dirty = true;
                if self.cur_col > row.last_dirty {
                    row.last_dirty = self.cur_col;
                }
                if self.cur_col < row.first_dirty {
                    row.first_dirty = self.cur_col;
                }
            }

            self.cur_col += 1;
            if self.cur_col >= self.width {
                self.cur_row += 1;
                self.cur_col = 0;
            }
            if self.cur_row >= self.height {
                panic!("TODO");
            }
        }
    }

    pub fn restyle(&mut self, style: Style) {
        let row = &mut self.rows[self.cur_row];
        row.cells[self.cur_col].style = style;

        // TODO this is basically duplicated from above
        row.is_dirty = true;
        if self.cur_col > row.last_dirty {
            row.last_dirty = self.cur_col;
        }
        if self.cur_col < row.first_dirty {
            row.first_dirty = self.cur_col;
        }
    }

    pub fn write(&mut self, s: &str) {
        self.attrwrite(s, Style());
    }

    pub fn repaint(&mut self) {
        // TODO wrap this
        // TODO check for existence of cup?  fallback?
        //self.terminfo.write_cap2("cup", self.start_col as int, self.start_row as int);

        let mut is_bold = false;
        let mut fg = 0;

        for row_i in range(0, self.height) {
            let row = &mut self.rows[row_i];
            if ! row.is_dirty {
                continue;
            }

            // TODO the terminal could track its cursor position and optimize this move away
            self.terminfo.reposition(self.start_col + row.first_dirty, self.start_row + row_i);
            // TODO with this level of optimization, imo, there should also be a method for forcibly redrawing the entire screen from (presumed) scratch
            for col in range(row.first_dirty, row.last_dirty + 1) {
                let cell = &mut row.cells[col];

                // Deal with formatting
                if cell.style.is_bold && ! is_bold {
                    self.terminfo.write_cap("bold");
                    is_bold = true;
                }
                else if is_bold && ! cell.style.is_bold {
                    // TODO this resets formatting entirely -- there's no way
                    // to turn off bold/underline individually  :|
                    self.terminfo.write_cap("sgr0");
                    is_bold = false;
                }

                if cell.style.fg_color != fg {
                    fg = cell.style.fg_color;

                    // Replace -1 with an arbitrarily large number, which
                    // apparently functions as resetting to the default color
                    // with setaf.  Maybe.
                    // TODO i am not sure how reliable this is
                    let actual_fg = match fg {
                        -1 => 65535,
                        _ => fg,
                    };
                    // TODO what if setaf doesn't exist?  fall back to setf i
                    // guess, but what's the difference?
                    self.terminfo.write_cap1("setaf", actual_fg);
                }

                self.terminfo.write(cell.glyph.to_string().as_slice());
                cell.dirty = false;
            }

            row.is_dirty = false;
            row.first_dirty = self.width;
            row.last_dirty = 0;
        }

        // Clean up attribute settings when done
        // TODO optimization possibilities here if we remember the current cursor style -- which we may need to do anyway once we're tracking more than bold
        if is_bold {
            self.terminfo.write_cap("sgr0");
        }

        // TODO move the cursor to its original position if that's not where it is now
    }

    // -------------------------------------------------------------------------
    // Input

    // TODO should this auto-repaint?  seems to make sense and i think curses
    // does
    pub fn read_key(&mut self) -> Key {
        // Thanks to urwid for already doing much of this work in a readable
        // manner!
        // TODO this doesn't time out, doesn't check for key sequences, etc
        // etc.  it's hilariously sad.
        // TODO should have a timeout after Esc...  et al.?
        // TODO this could probably stand to be broken out a bit
        let byte = match self.terminfo.in_file.borrow_mut().read_byte() {
            Ok(byte) => byte,
            // TODO how can this actually happen?
            Err(err) => panic!("couldn't read a byte?!  {:?}", err),
        };

        let mut bytes = vec![byte];

        if 32 <= byte && byte <= 126 {
            // ASCII character
            return Key::Character(byte as char);
        }

        // XXX urwid here checks for some specific keys: tab, enter, backspace

        // XXX is this cross-terminal?
        if 0 < byte && byte < 27 {
            // Ctrl-x
            // TODO no nice way to return this though, so just do the character
            return Key::Character(byte as char);
        }
        if 27 < byte && byte < 32 {
            // Ctrl-X
            // TODO no nice way to return this though, so just do the character
            return Key::Character(byte as char);
        }

        // TODO supporting other encodings would be...  nice...  but hard.
        // what does curses do here; is this where it uses the locale?
        let encoding = "utf8";
        if encoding == "utf8" && byte > 127 && byte < 256 {
            let mut utf8buf: [u8; 6] = [byte, 0, 0, 0, 0, 0];

            let need_more;
            if byte & 0xe0 == 0xc0 {
                // two-byte form
                need_more = 1;
            }
            else if byte & 0xf0 == 0xe0 {
                // three-byte form
                need_more = 2;
            }
            else if byte & 0xf8 == 0xf0 {
                // four-byte form
                need_more = 3;
            }
            else {
                panic!(format!("junk byte {:?}", byte));
            }

            // TODO this returns IoResult; should catch, convert to error if amount read is less
            // than need_more, and then do...  something.
            self.terminfo.in_file.borrow_mut().read(&mut utf8buf[1..need_more]);
            // TODO umm this all only works for utf8
            // TODO and what if it's bogus utf8?
            let decoded = str::from_utf8(bytes.as_slice()).unwrap();
            if decoded.len() != 1 {
                panic!("unexpected decoded string length!");
            }

            return Key::Character(decoded.char_at(0));
        }

        // XXX urwid has if byte > 127 && byte < 256...  but that's covered
        // above because we are always utf8.


        // OK, check for cute terminal escapes
        loop {
            let (maybe_key, _remaining_bytes) = self.terminfo.keypress_trie.find_prefix(bytes.as_slice());
            match maybe_key {
                Some(key) => {
                    return key;
                }
                None => (),
            }
            // XXX right now we are discarding any leftover bytes -- but we also only read one at a time so there are never leftovers
            // bytes = remaining_bytes;
            // XXX i don't know a better way to decide when to give up reading a sequence?  i guess it should be time-based
            if bytes.len() > 8 {
                break;
            }
            match self.terminfo.in_file.borrow_mut().read_byte() {
                Ok(byte) => bytes.push(byte),
                Err(_) => break,
            }
        }

        // TODO uhh lol this doesn't seem like a useful fallback?  now we just have a pile of bytes
        // and throw them all away whoops
        return Key::Character(byte as char);
    }

    // TODO unclear whether the trailing \n should be included
    // TODO this doesn't handle backspace etc; it's a pretty piss-poor readline
    // implementation really  :)
    pub fn read_line(&mut self) -> String {
        let mut chars: Vec<char> = vec![];
        loop {
            match self.read_key() {
                Key::Character(ch) => {
                    // TODO should \r become \n?  my raw() impl disables that
                    // TODO ...actually maybe this is a termcap??
                    if ch == '\r' {
                        chars.push('\n');
                    }
                    else {
                        chars.push(ch);
                    }
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                },
                _ => (),
            }
        }

        return chars.into_iter().collect();
    }

    /** Blocks until a key is pressed.
     *
     * This is identical to `read_key()`, except it returns nothing and reads
     * a little better if you don't care which key was pressed.
     */
    pub fn pause(&mut self) {
        self.read_key();
    }


}
