/** Low-level ncurses wrapper, for simple or heavily customized applications. */

extern mod std;

use libc::{c_char,c_int,c_long,c_schar,c_short,c_void,size_t};

use std::map::HashMap;

use c;
use termios;
use trie::Trie;

extern {
    fn setlocale(category: c_int, locale: *c_char) -> *c_char;

    // XXX why the fuck is this not available
    const stdout: *libc::FILE;
}


/** Prints a given termcap sequence when it goes out of scope. */
struct TidyTermcap {
    term: &Terminal,
    cap: &str,

    drop {
        self.term.write_cap(self.cap);
    }
}


struct Terminal {
    in_fd: c_int,
    in_file: io::Reader,
    out_fd: c_int,
    out_file: io::Writer,

    keypress_trie: @Trie<u8, Key>,

    priv c_terminfo: *c::TERMINAL,
    priv tidy_termstate: ~termios::TidyTerminalState,

    //term_type: ~str,

    drop {
        // TODO any C freeage needed here?

        // Undo the setup inflicted on the terminal
        c::nl();
        //c::echo();
        c::nocbreak();
    }
}
pub fn Terminal() -> @Terminal {
    let error_code: c_int = 0;
    // NULL first arg means read TERM from env (TODO).
    // second arg is a fd to spew to on error, but it's not used when there's
    // an error pointer.
    // third arg is a var to stick the error code in.
    // TODO allegedly setupterm doesn't work on BSD?
    let res = c::setupterm(ptr::null(), -1, ptr::addr_of(&error_code));

    if res != c::OK {
        if error_code == -1 {
            fail ~"Couldn't find terminfo database";
        }
        else if error_code == 0 {
            fail ~"Couldn't identify terminal";
        }
        else if error_code == 1 {
            // The manual puts this as "terminal is hard-copy" but come on.
            fail ~"Terminal appears to be made of paper";
        }
        else {
            fail ~"Something is totally fucked";
        }
    }

    // Okay; now terminfo is sitting in a magical global somewhere.  Snag a
    // pointer to it.
    let terminfo = c::cur_term;

    let keypress_trie = Trie();
    let mut p: **c_char = ptr::to_unsafe_ptr(&c::strnames[0]);
    unsafe {
        while *p != ptr::null() {
            let capname = str::raw::from_c_str(*p);

            if capname[0] == "k"[0] {
                let cap = c::tigetstr(*p);
                if cap != ptr::null() {
                    let cap_key = vec::raw::from_buf_raw(cap, libc::strlen(cap) as uint).map(|el| *el as u8);
                    keypress_trie.insert(cap_key, cap_to_key(capname));
                }
            }

            p = ptr::offset(p, 1);
        }
    }

    let term = @Terminal{
        // TODO would be nice to parametrize these, but Reader and Writer do
        // not yet expose a way to get the underlying fd, which makes the API
        // sucky
        in_fd: 0,
        in_file: io::stdin(),
        out_fd: 1,
        out_file: io::stdout(),

        keypress_trie: keypress_trie,

        c_terminfo: terminfo,
        tidy_termstate: termios::TidyTerminalState(0),
    };

    return term;
}
impl Terminal {
    // ------------------------------------------------------------------------
    // Capability inspection
    // TODO ideally, ultimately, every useful cap will be covered here...

    // TODO these should use ioctl, and cache unless WINCH, and...
    fn height() -> uint {
        return self.numeric_cap("lines");
    }
    fn width() -> uint {
        return self.numeric_cap("cols");
    }

    // ------------------------------------------------------------------------
    // Very-low-level capability inspection

    fn flag_cap(name: &str) -> bool {
        c::set_curterm(self.c_terminfo);

        let mut value = 0;
        do str::as_c_str(name) |bytes| {
            value = c::tigetflag(bytes);
        }

        if value == -1 {
            // wrong type
            fail;
        }

        // Otherwise, is 0 or 1
        return value as bool;
    }

    fn numeric_cap(name: &str) -> uint {
        c::set_curterm(self.c_terminfo);

        let mut value = -1;
        do str::as_c_str(name) |bytes| {
            value = c::tigetnum(bytes);
        }

        if value == -2 {
            // wrong type
            fail;
        }
        else if value == -1 {
            // missing; should be None
            fail;
        }

        return value as uint;
    }

    fn _string_cap_cstr(name: &str) -> *c_char unsafe {
        c::set_curterm(self.c_terminfo);

        let mut value = ptr::null();
        do str::as_c_str(name) |bytes| {
            value = c::tigetstr(bytes);
        }

        if value == ptr::null() {
            // missing; should be None really
            fail;
        }
        else if value == cast::transmute(-1) {
            // wrong type
            fail;
        }

        return value;
    }

    fn string_cap(name: &str) -> ~str unsafe {
        let value = self._string_cap_cstr(name);

        return str::raw::from_c_str(value);
    }

    // TODO i am not really liking the string capability handling anywhere in
    // here.
    // the variable arguments thing sucks ass.  we should at LEAST check for
    // how many arguments are expected, and ideally enforce it at compile time
    // somehow -- perhaps with a method for every string cap.  (yikes.  having
    // keys be a separate thing would help, though.)

    /** Returns a string capability, formatted with the passed vector of
     * arguments.
     *
     * Passing the correct number of arguments is your problem, though any
     * missing arguments become zero.  No capability requires more than 9
     * arguments.
     */
    fn format_cap(name: &str, args: &[int]) -> ~str {
        c::set_curterm(self.c_terminfo);

        let template = self._string_cap_cstr(name);
        let padded_args = args.to_vec() + [0, .. 8];
        let formatted = c::tparm(
            template,
            padded_args[0] as c_long,
            padded_args[1] as c_long,
            padded_args[2] as c_long,
            padded_args[3] as c_long,
            padded_args[4] as c_long,
            padded_args[5] as c_long,
            padded_args[6] as c_long,
            padded_args[7] as c_long,
            padded_args[8] as c_long
        );

        let rv = unsafe { str::raw::from_c_str(formatted) };

        return rv;
    }

    fn _write_capx(name: &str,
            arg1: c_long, arg2: c_long, arg3: c_long,
            arg4: c_long, arg5: c_long, arg6: c_long,
            arg7: c_long, arg8: c_long, arg9: c_long)
    {
        c::set_curterm(self.c_terminfo);

        let template = self._string_cap_cstr(name);

        let formatted = c::tparm(
            template, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);

        // TODO we are supposed to use curses's tputs(3) to print formatted
        // capabilities, because they sometimes contain "padding" of the form
        // $<5>.  alas, tputs always prints to stdout!  but we don't yet allow
        // passing in a custom fd so i guess that's okay.  would be nice to
        // reimplement this in Rust code someday though.
        c::putp(formatted);

        // TODO another reason dipping into C sucks: we have to manually flush
        // its buffer every time we do so.
        libc::fflush(stdout);
    }

    // TODO seems it would make sense to cache non-formatted capabilities (as
    // well as numeric/flag ones), which i think blessings does
    fn write_cap(cap_name: &str) {
        // If we're calling this function then this capability really shouldn't
        // take any arguments, but someone might have screwed up, or it may
        // have an escaped % or something.  Best do the whole formatting thing.
        self._write_capx(cap_name, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    }
    fn write_cap2(cap_name: &str, arg1: int, arg2: int) {
        self._write_capx(cap_name, arg1 as c_long, arg2 as c_long, 0, 0, 0, 0, 0, 0, 0);
    }

    fn write_tidy_cap(&self, do_cap: &str, undo_cap: &self/str) -> ~TidyTermcap/&self {
        self.write_cap(do_cap);

        return ~TidyTermcap{ term: self, cap: undo_cap };
    }



    // Output

    fn write(s: &str) {
        io::stdout().flush();
        // TODO well.  should be a bit more flexible, i guess.
        io::print(s);
        io::stdout().flush();
    }

    fn attrwrite(s: &str, style: &Style) {
        // TODO try to cut down on the amount of back-and-forth between c
        // strings and rust strings all up in here
        if style.is_underline {
            self.write_cap("smul");
        }

        // TODO this may need some escaping or whatever -- or maybe that
        // belongs in write()
        self.write(s);

        // Clean up after ourselves: reset style to default
        // TODO this is ripe for some optimizing
        self.write_cap("sgr0");
    }


    // Some stuff
    fn at(x: uint, y: uint, cb: &fn()) unsafe {
        self.write_cap("sc");  // save cursor
        // TODO check for existence of cup
        self.write_cap2("cup", y as int, x as int);

        cb();

        self.write_cap("rc");  // restore cursor
    }

    // Full-screen

    // TODO wonder if this could use a borrowed or unique window, since it's
    // local to this function?  (unique would also prevent its escape from
    // here.)  really really REALLY sucks that we can't let the caller decide
    // easily.
    fn fullscreen(@self, cb: &fn(@Window)) {
        // Enter fullscreen
        let _tidy_cup = self.write_tidy_cap("smcup", "rmcup");

        // Enable keypad mode
        let _tidy_kx = self.write_tidy_cap("smkx", "rmkx");

        // And clear the screen first
        self.write_cap("clear");

        // TODO intrflush, or is that a curses thing?

        // TODO so, we need to switch to raw mode *some*where.  is this an
        // appropriate place?  i assume if you have a fullscreen app then you
        // want to get keypresses.
        let tidy_termstate = termios::TidyTerminalState(self.in_fd);
        tidy_termstate.raw();

        let win = @Window{
            c_window: ptr::null(),  // TODO obviously

            term: self,
            x: 0,
            y: 0,
            width: self.width(),
            height: self.height(),
        };
        cb(win);
    }
}


struct Window {
    c_window: *c::WINDOW,
    term: @Terminal,

    x: uint,
    y: uint,
    // TODO: what happens to width and height on resize?  default is obviously
    // "nothing", but seems it would be nice to support default behavior like
    // preserving bottom/right margins or scaling proportionally
    width: uint,
    height: uint,

    drop {
        // TODO with multiple windows, need a Rust-level reference to the parent

        // TODO return value, though fuck if i know how this could ever fail
        c::endwin();

        // TODO do i need to do this with stdscr?  does it hurt?
        c::delwin(self.c_window);
    }
}
// TODO this probably doesn't need to be here quite like this
fn init_window(c_window: *c::WINDOW) -> @Window {
    // Something, something.  TODO explain
    c::intrflush(c_window, 0);

    // Always enable keypad (function keys, arrow keys, etc.)
    // TODO what are multiple screens?
    c::keypad(c_window, 1);

    let term = Terminal();

    return @Window{ c_window: c_window, term: term,   x: 0, y: 0, width: 0, height: 0 };
}

impl Window {
    ////// Properties

    /** Returns the size of the window as (rows, columns). */
    fn size() -> (uint, uint) {
        return (
            c::getmaxy(self.c_window) as uint,
            c::getmaxx(self.c_window) as uint);
    }

    /** Returns the current cursor position as (row, column). */
    fn position() -> (uint, uint) {
        return (
            c::getcury(self.c_window) as uint,
            c::getcurx(self.c_window) as uint);
    }

    ////// Methods

    fn mv(row: uint, col: uint) {
        // TODO return value
        c::wmove(self.c_window, row as c_int, col as c_int);
    }

    fn print(msg: &str) {
        // TODO return value
        // TODO this is variadic; string template exploits abound, should use %s really
        // TODO also should handle literal escape sequences somehow...?  strip?  escape?
        do str::as_c_str(msg) |bytes| {
            c::wprintw(self.c_window, bytes);
        }
    }

    fn write(msg: &str) {
        // TODO write to a buffer, only paint on repaint()
        self.term.write(msg);
    }

    fn attrwrite(s: &str, style: &Style) {
        // TODO same as above
        self.term.attrwrite(s, style);
    }

    fn repaint() {
        // TODO return value
        //c::wrefresh(self.c_window);

        // TODO implement me

        io::stdout().flush();
    }

    fn clear() {
        // TODO only touch in-memory screen and update on repaint
        // TODO this only works for the root window with no child windows; cute
        // optimization but not reliable in general.  also, moves the cursor,
        // which may not be desired.
        // TODO should this be done on fullscreen by default, or is it anyway?

        // TODO should this be a method on the terminal, since it's just a cap?

        self.term.write_cap("clear");
    }

    // Input

    fn getch() -> char {
        // TODO this name sucks
        let ch: c::wint_t = 0;
        let res = c::wget_wch(self.c_window, ptr::addr_of(&ch));
        if res == c::OK {
            return ch as char;
        }
        else if res == c::KEY_CODE_YES {
            // TODO this is super wrong; the keycodes overlap with legit
            // characters
            return ch as char;
        }
        else if res == c::ERR {
            fail;
        }
        else {
            // TODO wat
            fail;
        }
        // TODO what if you get WEOF...?
    }

    fn read_key() -> Key {
        // Thanks to urwid for already doing much of this work in a readable
        // manner!
        // TODO this doesn't time out, doesn't check for key sequences, etc
        // etc.  it's hilariously sad.
        // TODO should have a timeout after Esc...  et al.?
        // TODO this could probably stand to be broken out a bit
        let raw_byte = self.term.in_file.read_byte();
        if raw_byte < 0 {
            // TODO how can this actually happen?
            fail ~"couldn't read a byte?!";
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
                fail fmt!("junk byte %?", byte);
            }

            bytes += (self.term.in_file as io::ReaderUtil).read_bytes(need_more);
            // TODO umm this only works for utf8
            let decoded = str::from_bytes(bytes);
            if decoded.len() != 1 {
                fail ~"unexpected decoded string length!";
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


        // TODO 
        // TODO this name sucks
        let ch: c::wint_t = 0;
        let res = c::wget_wch(self.c_window, ptr::addr_of(&ch));
        if res == c::OK {
            return Character(ch as char);
        }
        else if res == c::KEY_CODE_YES {
            return SpecialKey(
                if ch == c::KEY_UP { KEY_UP }
                else if ch == c::KEY_DOWN { KEY_DOWN }
                else if ch == c::KEY_LEFT { KEY_LEFT }
                else if ch == c::KEY_RIGHT { KEY_RIGHT }
                else { fail; }
            );
        }
        else if res == c::ERR {
            fail;
        }
        else {
            // TODO wat
            fail;
        }
        // TODO what if you get WEOF...?
    }

    /** Blocks until a key is pressed.
     *
     * This is identical to `read_key()`, except it returns nothing and reads
     * a little better if you don't care which key was pressed.
     */
    fn pause() {
        self.read_key();
    }


    fn readln() -> ~str unsafe {
        // TODO what should maximum buffer length be?
        // TODO or perhaps i should reimplement the getnstr function myself with getch.
        const buflen: uint = 80;
        let buf = libc::malloc(buflen * sys::size_of::<c::wint_t>() as size_t)
            as *c::wint_t;
        let res = c::wgetn_wstr(self.c_window, buf, buflen as c_int);

        if res != c::OK {
            fail;
        }

        let vec = do vec::from_buf(buf, buflen).map |ch| { *ch as char };
        libc::free(buf as *c_void);

        return str::from_chars(vec);
    }

    // Attributes

    fn restyle(num_chars: int, attrflags: int, color_index: int) {
        // NOTE: chgat() returns a c_int, but documentation indicates the value
        // is meaningless.
        c::chgat(num_chars as c_int, attrflags as c::attr_t, color_index as c_short, ptr::null());
    }


    // Drawing

    fn set_box(vert: char, horiz: char) {
        // TODO return value
        c::box_set(self.c_window, ptr::addr_of(&__char_to_cchar_t(vert)), ptr::addr_of(&__char_to_cchar_t(horiz)));
    }

    fn set_border(l: char, r: char, t: char, b: char, tl: char, tr: char, bl: char, br: char) {
        // TODO return value
        c::wborder_set(self.c_window,
            ptr::addr_of(&__char_to_cchar_t(l)),
            ptr::addr_of(&__char_to_cchar_t(r)),
            ptr::addr_of(&__char_to_cchar_t(t)),
            ptr::addr_of(&__char_to_cchar_t(b)),
            ptr::addr_of(&__char_to_cchar_t(tl)),
            ptr::addr_of(&__char_to_cchar_t(tr)),
            ptr::addr_of(&__char_to_cchar_t(bl)),
            ptr::addr_of(&__char_to_cchar_t(br))
        );
    }


    // Misc

    // TODO flesh this out.  apparently '2' for 'very visible' is also
    // supported?
    fn hide_cursor() {
        // TODO return value
        // TODO this belongs to the terminal, not a window
        c::curs_set(0);
    }
}

pub fn init_screen() -> @Window {
    // TODO not sure all this stuff should be initialized /here/ really
    // TODO perhaps ensure this is only called once?  or make it a context
    // manager ish thing?

    // TODO this is not very cross-platform, but LC_ALL is a macro  :|
    // setlocale(LC_ALL, "") reads the locale from the environment
    let empty_string: c_char = 0;
    setlocale(6, ptr::addr_of(&empty_string));

    let c_window = c::initscr();
    if c_window == ptr::null() {
        // Should only fail due to memory pressure
        fail;
    }

    return init_window(c_window);
}


////////////////////////////////////////////////////////////////////////////////
// Attributes

pub struct Style {
    // TODO i guess these could be compacted into a bitstring, but eh.
    is_bold: bool,
    is_underline: bool,

    // TODO strictly speaking these should refer to entire colors, not just
    // color numbers, for compatability with a truckload of other kinds of
    // terminals.  but, you know.
    // TODO -1 for the default is super hokey and only for curses compat
    fg_color: int,
    bg_color: int,
}
pub fn Style() -> Style {
    return NORMAL;
}
impl Style {
    fn bold() -> Style {
        return Style{ is_bold: true, ..self };
    }

    fn underline() -> Style {
        return Style{ is_underline: true, ..self };
    }

    // TODO this pretty much blows; color pairs are super archaic and i am
    // trying to hack around them until i just give up and bail on the curses
    // dependency.  works on my machine...
    // TODO this only works for the first 16 colors anyway
    // TODO calling start_color() resets all color pairs, so you can't use this
    // before capturing the window...  :|
    // TODO this doesn't handle default colors correctly, because those are
    // color index -1.
    fn fg(color: int) -> Style {
        return Style{ fg_color: color, ..self };
    }
    fn bg(color: int) -> Style {
        return Style{ bg_color: color, ..self };
    }

    fn c_value() -> c_int {
        let mut rv: c_int = 0;

        if self.is_bold {
            rv |= c::A_BOLD;
        }
        if self.is_underline {
            rv |= c::A_UNDERLINE;
        }
        
        // Calculate a pair number to consume.  It's a signed short, so use the
        // lower 8 bits for fg and upper 7 for bg
        // TODO without ext_colors, this gets all fucked up if the pair number
        // is anything over 255
        let fg = match self.fg_color {
            -1 => 14,
            _ => self.fg_color,
        };
        let bg = match self.bg_color {
            -1 => 14,
            _ => self.bg_color,
        };
        //let pair = ((bg << 4) | fg) as c_short;
        let pair = fg as c_short;
        c::init_pair(pair, self.fg_color as c_short, self.bg_color as c_short);

        rv |= c::COLOR_PAIR(pair as c_int);

        return rv;
    }
}

pub const NORMAL: Style = Style{ is_bold: false, is_underline: false, fg_color: -1, bg_color: -1 };


////////////////////////////////////////////////////////////////////////////////
// Key handling

// TODO: i don't know how to handle ctrl-/alt- sequences.
// 1. i don't know how to represent them type-wise
// 2. i don't know how to parse them!  they aren't in termcap.
enum SpecialKey {
    KEY_LEFT,
    KEY_RIGHT,
    KEY_UP,
    KEY_DOWN,
    KEY_ESC,

    // XXX temp kludge until i have all yonder keys
    KEY_UNKNOWN,
}

enum Key {
    Character(char),
    SpecialKey(SpecialKey),
    FunctionKey(uint),
}

fn cap_to_key(cap: ~str) -> Key {
    // TODO this matching would be much more efficient if it used, hurr, a
    // trie.  but seems silly to build one only to use it a few times.
    // TODO uh maybe this should use the happy C names
    return match cap {
        ~"kcuf1" => SpecialKey(KEY_RIGHT),
        ~"kcub1" => SpecialKey(KEY_LEFT),
        ~"kcup1" => SpecialKey(KEY_UP),
        ~"kcud1" => SpecialKey(KEY_DOWN),
        ~"kf1" => FunctionKey(1),
        ~"kf2" => FunctionKey(2),
        ~"kf3" => FunctionKey(3),
        ~"kf4" => FunctionKey(4),
        ~"kf5" => FunctionKey(5),
        ~"kf6" => FunctionKey(6),
        ~"kf7" => FunctionKey(7),
        ~"kf8" => FunctionKey(8),
        ~"kf9" => FunctionKey(9),
        ~"kf10" => FunctionKey(10),
        ~"kf11" => FunctionKey(11),
        ~"kf12" => FunctionKey(12),
        _ => SpecialKey(KEY_UNKNOWN),
    };
}

////////////////////////////////////////////////////////////////////////////////
// Misc that should probably go away

/** Returns the screen size as (rows, columns). */
pub fn screen_size() -> (uint, uint) {
    return (c::LINES as uint, c::COLS as uint);
}

// Attribute definition

pub fn define_color_pair(color_index: int, fg: c_short, bg: c_short) {
    // TODO return value
    c::init_pair(color_index as c_short, fg, bg);
}

pub fn new_window(height: uint, width: uint, starty: uint, startx: uint) -> @Window {
    let c_window = c::newwin(height as c_int, width as c_int, starty as c_int, startx as c_int);

    if c_window == ptr::null() {
        // TODO?
        fail;
    }

    return init_window(c_window);
}




fn __char_to_cchar_t(ch: char) -> c::cchar_t {
    return c::cchar_t{
        attr: c::A_NORMAL as c::attr_t,
        chars: [ch as c::wchar_t, 0, 0, 0, 0],
    };
}
