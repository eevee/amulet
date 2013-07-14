use std::str;
use std::uint;
use std::vec;

use ll::Terminal;
use ll::TidyBundle;
use ll::{Character,Key,Style};  // TODO move these somewhere dealing with keys and text and terminal properties

struct CanvasCell {
    dirty: bool,
    glyph: char,
    style: Style,
}

struct CanvasRow {
    is_dirty: bool,
    last_dirty: uint,
    first_dirty: uint,
    cells: ~[CanvasCell],
}

struct Canvas<'self> {
    term: &'self Terminal,
    start_row: uint,
    start_col: uint,
    cur_row: uint,
    cur_col: uint,
    height: uint,
    width: uint,

    rows: ~[CanvasRow],
    tidyables: TidyBundle<'self>,
}

pub fn Canvas<'terminal>(term: &'terminal Terminal, start_row: uint, start_col: uint, height: uint, width: uint) -> ~Canvas<'terminal> {
    let rows = vec::from_fn(height, |_row| {
        CanvasRow{
            is_dirty: false,
            last_dirty: 0,
            first_dirty: 0,
            cells: vec::from_fn(width, |_col| {
                CanvasCell{
                    dirty: false,
                    glyph: ' ',
                    style: Style(),
                }
            }),
        }
    });
    return ~Canvas{
        term: term,

        start_row: start_row,
        start_col: start_col,
        cur_row: 0,
        cur_col: 0,
        height: height,
        width: width,

        rows: rows,
        tidyables: TidyBundle{ tidy_termcaps: ~[], tidy_termstates: ~[] },
    };
}

impl<'self> Canvas<'self> {
    // -------------------------------------------------------------------------
    // Creation

    pub fn spawn(&self, start_row: uint, start_col: uint, height: uint, width: uint) -> ~Canvas {
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

        fail!(~"todo not done yet");
    }

    // -------------------------------------------------------------------------
    // Accessors

    /** Returns the size of the canvas as (rows, columns). */
    pub fn size(&self) -> (uint, uint) {
        return (self.height, self.width);
    }

    pub fn position(&self) -> (uint, uint) {
        return (self.cur_row, self.cur_col);
    }

    pub fn move(&mut self, row: uint, col: uint) {
        self.cur_row = row;
        self.cur_col = col;
    }


    // -------------------------------------------------------------------------
    // Output

    pub fn clear(&mut self) {
        // TODO clearing the screen can be done with a single termcap, but how
        // do i remember that
        for self.rows.mut_iter().advance |row| {
            row.is_dirty = true;
            row.last_dirty = self.width - 1;
            row.first_dirty = 0;
            for row.cells.mut_iter().advance |cell| {
                *cell = CanvasCell{
                    dirty: true,
                    glyph: ' ',
                    style: Style(),
                };
            }
        }
    }

    pub fn attrwrite(&mut self, s: &str, style: Style) {
        for s.iter().advance |glyph| {
            if glyph == '\n' {
                // TODO this probably needs (a) more cases, (b) termcap
                // influence
                self.cur_row += 1;
                self.cur_col = 0;

                if self.cur_row >= self.height {
                    fail!(~"TODO");
                }

                // TODO is there ever anything to write here?
                return;
            }

            {
                let row = &mut self.rows[self.cur_row];
                row.cells[self.cur_col] = CanvasCell{
                    dirty: true,
                    glyph: glyph,
                    style: copy style,
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
                fail!(~"TODO");
            }
        }
    }

    pub fn restyle(&mut self, style: Style) {
        let row = &mut self.rows[self.cur_row];
        row.cells[self.cur_col].style = copy style;

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
        //self.term.write_cap2("cup", self.start_col as int, self.start_row as int);

        let mut is_bold = false;
        let mut fg = 0;

        for uint::range(0, self.height) |row_i| {
            let row = &mut self.rows[row_i];
            if ! row.is_dirty {
                loop;
            }

            // TODO the terminal could track its cursor position and optimize this move away
            self.term.move(self.start_col + row.first_dirty, self.start_row + row_i);
            // TODO with this level of optimization, imo, there should also be a method for forcibly redrawing the entire screen from (presumed) scratch
            for uint::range(row.first_dirty, row.last_dirty + 1) |col| {
                let cell = row.cells[col];

                // Deal with formatting
                if cell.style.is_bold && ! is_bold {
                    self.term.write_cap("bold");
                    is_bold = true;
                }
                else if is_bold && ! cell.style.is_bold {
                    // TODO this resets formatting entirely -- there's no way
                    // to turn off bold/underline individually  :|
                    self.term.write_cap("sgr0");
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
                    self.term.write_cap1("setaf", actual_fg);
                }

                self.term.write(fmt!("%c", cell.glyph));
                row.cells[col].dirty = false;
            }

            row.is_dirty = false;
            row.first_dirty = self.width;
            row.last_dirty = 0;
        }

        // Clean up attribute settings when done
        // TODO optimization possibilities here if we remember the current cursor style -- which we may need to do anyway once we're tracking more than bold
        if is_bold {
            self.term.write_cap("sgr0");
        }

        // TODO move the cursor to its original position if that's not where it is now
    }

    // -------------------------------------------------------------------------
    // Input

    // TODO should this auto-repaint?  seems to make sense and i think curses
    // does
    pub fn read_key(&self) -> Key {
        // Thanks to urwid for already doing much of this work in a readable
        // manner!
        // TODO this doesn't time out, doesn't check for key sequences, etc
        // etc.  it's hilariously sad.
        // TODO should have a timeout after Esc...  et al.?
        // TODO this could probably stand to be broken out a bit
        let raw_byte = self.term.in_file.read_byte();
        if raw_byte < 0 {
            // TODO how can this actually happen?
            fail!(~"couldn't read a byte?!");
        }

        let mut byte = raw_byte as u8;
        let mut bytes = ~[byte];

        if 32 <= byte && byte <= 126 {
            // ASCII character
            return Character(byte as char);
        }

        // XXX urwid here checks for some specific keys: tab, enter, backspace

        // XXX is this cross-terminal?
        if 0 < byte && byte < 27 {
            // Ctrl-x
            // TODO no nice way to return this though, so just do the character
            return Character(byte as char);
        }
        if 27 < byte && byte < 32 {
            // Ctrl-X
            // TODO no nice way to return this though, so just do the character
            return Character(byte as char);
        }

        // TODO supporting other encodings would be...  nice...  but hard.
        // what does curses do here; is this where it uses the locale?
        let encoding = "utf8";
        if encoding == "utf8" && byte > 127 && byte < 256 {
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
                fail!(fmt!("junk byte %?", byte));
            }

            bytes.push_all_move(self.term.in_file.read_bytes(need_more));
            // TODO umm this only works for utf8
            let decoded = str::from_bytes(bytes);
            if decoded.len() != 1 {
                fail!(~"unexpected decoded string length!");
            }

            return Character(decoded.char_at(0));
        }

        // XXX urwid has if byte > 127 && byte < 256...  but that's covered
        // above because we are always utf8.

        
        // OK, check for cute terminal escapes
        loop {
            let (maybe_key, remaining_bytes) = self.term.keypress_trie.find_prefix(bytes);
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
            // XXX again, why does this return an int?  can it be negative on error?
            bytes.push(self.term.in_file.read_byte() as u8);
        }


        return Character(byte as char);
    }

    // TODO unclear whether the trailing \n should be included
    // TODO this doesn't handle backspace etc; it's a pretty piss-poor readline
    // implementation really  :)
    pub fn read_line(&mut self) -> ~str {
        let mut chars: ~[char] = ~[];
        loop {
            match self.read_key() {
                Character(ch) => {
                    // TODO should \r become \n?  my raw() impl disables that
                    // TODO ...actually maybe this is a termcap??
                    if ch == '\r' {
                        ch = '\n';
                    }
                    chars.push(ch);
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                },
                _ => (),
            }
        }

        return str::from_chars(chars);
    }

    /** Blocks until a key is pressed.
     *
     * This is identical to `read_key()`, except it returns nothing and reads
     * a little better if you don't care which key was pressed.
     */
    pub fn pause(&self) {
        self.read_key();
    }


}
