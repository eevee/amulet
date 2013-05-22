use core::str;

use ll::Terminal;
use ll::{Character,Key,Style};  // TODO move these somewhere dealing with keys and text and terminal properties

struct CanvasCell {
    dirty: bool,
    glyph: char,
    style: (),
}

struct Canvas {
    term: @Terminal,
    start_row: uint,
    start_col: uint,
    cur_row: uint,
    cur_col: uint,
    height: uint,
    width: uint,

    cells: ~[~[CanvasCell]],
}

pub fn Canvas(term: @Terminal, start_row: uint, start_col: uint, height: uint, width: uint) -> Canvas {
    let cells = vec::from_fn(height, |_row| {
        vec::from_fn(width, |_col| {
            CanvasCell{
                dirty: false,
                glyph: ' ',
                style: (),
            }
        })
    });
    return Canvas{
        term: term,

        start_row: start_row,
        start_col: start_col,
        cur_row: start_row,
        cur_col: start_col,
        height: height,
        width: width,

        cells: cells,
    };
}

impl Canvas {
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
        for self.cells.each_mut |row| {
            for row.each_mut |cell| {
                *cell = CanvasCell{
                    dirty: true,
                    glyph: ' ',
                    style: (),
                };
            }
        }
    }

    pub fn attrwrite(&mut self, s: &str, style: &Style) {
        for str::each_char(s) |glyph| {
            if glyph == '\n' {
                // TODO this probably needs (a) more cases, (b) termcap
                // influence
                self.cur_row += 1;
                self.cur_col = self.start_col;

                if self.cur_row >= self.start_row + self.height {
                    fail!(~"TODO");
                }

                // TODO is there ever anything to write here?
                return;
            }

            self.cells[self.cur_row][self.cur_col] = CanvasCell{
                dirty: true,
                glyph: glyph,
                style: (),
            };

            self.cur_col += 1;
            if self.cur_col >= self.start_col + self.width {
                self.cur_row += 1;
                self.cur_col = self.start_col;
            }
            if self.cur_row >= self.start_row + self.height {
                fail!(~"TODO");
            }
        }
    }

    pub fn write(&mut self, s: &str) {
        self.attrwrite(s, &Style());
    }

    pub fn repaint(&mut self) {
        // TODO wrap this
        // TODO check for existence of cup?  fallback?
        //self.term.write_cap2("cup", self.start_col as int, self.start_row as int);

        // TODO this does sc/rc which is not really necessary
        for uint::range(0, self.height) |row| {
            let mut dirty_col = 0;
            while dirty_col < self.width && ! self.cells[row][dirty_col].dirty {
                dirty_col += 1;
            }
            if dirty_col >= self.width {
                loop;
            }
            do self.term.at(self.start_col + dirty_col, self.start_row + row) {
                for uint::range(dirty_col, self.width) |col| {
                    self.term.write(fmt!("%c", self.cells[row][col].glyph));
                    self.cells[row][col].dirty = false;
                }
            }
        }
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

            bytes += self.term.in_file.read_bytes(need_more);
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
                option::Some(key) => {
                    return key;
                }
                option::None => (),
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
