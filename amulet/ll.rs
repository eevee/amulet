/** Low-level ncurses wrapper, for simple or heavily customized applications. */

use libc::{c_char,c_int,c_schar,c_short,c_void,size_t};

use c;

extern {
    fn setlocale(category: c_int, locale: *c_char) -> *c_char;
}


struct Terminal {
    c_terminfo: *c::TERMINAL,

    // TODO drop?
}
pub fn Terminal() -> @Terminal {
    let error_code: c_int = 0;
    // NULL first arg means read TERM from env (TODO).  second arg is a fd to
    // spew to (eh).  third is error output.
    // TODO allegedly setupterm doesn't work on BSD?
    let res = c::setupterm(ptr::null(), 1, ptr::addr_of(&error_code));

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

    return @Terminal{ c_terminfo: terminfo };
}
impl Terminal {
    // Capability inspection

    // TODO curses actually exposes all the names of all the capabilities!  so
    // let's just turn this all into a couple maps at startup and avoid all
    // this garbage.
    fn cap_flag(name: &str) -> bool {
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

    fn cap_num(name: &str) -> uint {
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

    fn cap_str(name: &str) -> ~str unsafe {
        let mut value = ptr::null();
        do str::as_c_str(name) |bytes| {
            value = c::tigetstr(bytes);
        }

        if value == ptr::null() {
            // missing; should be None really
            fail;
        }
        else if value == cast::reinterpret_cast(&-1) {
            // wrong type
            fail;
        }

        return str::raw::from_c_str(value);
    }


    fn height() -> uint {
        // TODO should use ioctl, and cache unless WINCH, and...
        return self.cap_num("lines");
    }


    // Output
    fn print(s: &str) {
        // TODO well.  should be a bit more flexible, i guess.
        io::print(s);
    }


    // Some stuff
    fn at(x: uint, y: uint, cb: &fn()) unsafe {
        self.print(self.cap_str("sc"));  // save cursor
        // TODO check cup
        do str::as_c_str(self.cap_str("cup")) |bytes| {
            self.print(str::raw::from_c_str(c::tparm(bytes, y as c_int, x as c_int)));
        }
        cb();
        self.print(self.cap_str("rc"));  // restore cursor
    }
}


struct Window {
    c_window: *c::WINDOW,
    term: @Terminal,

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

    return @Window{ c_window: c_window, term: term };
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
        do str::as_c_str(msg) |bytes| {
            c::wprintw(self.c_window, bytes);
        }
    }

    fn repaint() {
        // TODO return value
        c::wrefresh(self.c_window);
    }

    fn clear() {
        // TODO return value
        c::wclear(self.c_window);
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

    fn attrprint(s: &str, style: Style) {
        // TODO this leaves state behind...  set back to normal after?
        // TODO should probably interact nicely with background color
        c::wattrset(self.c_window, style.c_value);
        // TODO variadic
        do str::as_c_str(s) |bytes| {
            c::wprintw(self.c_window, bytes);
        }
    }
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

    // TODO return value
    c::start_color();
    // TODO return value
    // TODO this is an ncurses extension...  but we're linking with ncurses,
    // so, eh
    c::use_default_colors();

    // TODO these are also global, yikes
    c::cbreak();
    //c::noecho();
    c::nonl();

    return init_window(c_window);
}


////////////////////////////////////////////////////////////////////////////////
// Attributes

pub struct Style {
    c_value: c_int,
}
pub fn Style() -> Style {
    return Style{ c_value: 0 };
}
impl Style {
    fn bold() -> Style {
        return Style{ c_value: self.c_value | c::A_BOLD };
    }

    fn underline() -> Style {
        return Style{ c_value: self.c_value | c::A_UNDERLINE };
    }

    // TODO this pretty much blows; color pairs are super archaic and i am
    // trying to hack around them until i just give up and bail on the curses
    // dependency.  works on my machine...
    // TODO this only works for the first 16 colors anyway
    // TODO calling start_color() resets all color pairs, so you can't use this
    // before capturing the window...  :|
    // TODO this doesn't handle default colors correctly, because those are
    // color index -1.
    fn fg(color: uint) -> Style {
        let current_pair = c::PAIR_NUMBER(self.c_value);
        let new_pair = current_pair & 0x0f | (color as c_int << 4);
        c::init_pair(new_pair as c_short, ((new_pair & 0xf0) >> 4) as c_short, (new_pair & 0x0f) as c_short);
        return Style{ c_value: self.c_value & !c::A_COLOR | c::COLOR_PAIR(new_pair) };
    }
    fn bg(color: uint) -> Style {
        let current_pair = c::PAIR_NUMBER(self.c_value);
        let new_pair = current_pair & 0xf0 | (color as c_int);
        c::init_pair(new_pair as c_short, ((new_pair & 0xf0) >> 4) as c_short, (new_pair & 0x0f) as c_short);
        return Style{ c_value: self.c_value & !c::A_COLOR | c::COLOR_PAIR(new_pair) };
    }
        
}


////////////////////////////////////////////////////////////////////////////////
// Key handling

enum SpecialKey {
    KEY_LEFT,
    KEY_RIGHT,
    KEY_UP,
    KEY_DOWN,
    KEY_ESC,
}

enum Key {
    Character(char),
    SpecialKey(SpecialKey),
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
