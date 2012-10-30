/** Low-level ncurses wrapper, for simple or heavily customized applications. */

use libc::{c_char,c_int,c_short,c_void,size_t};

use c;

extern {
    fn setlocale(category: c_int, locale: *c_char) -> *c_char;
}

struct Window {
    c_window: *c::WINDOW,

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

    return @Window{ c_window: c_window };
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
