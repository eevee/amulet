/** Low-level ncurses wrapper, for simple or heavily customized applications. */

use libc::{c_char,c_int,c_void,size_t};

use c;

extern {
    fn setlocale(category: c_int, locale: *c_char) -> *c_char;
}

pub struct Window {
    c_window: *c::WINDOW,
}
pub fn Window(c_window: *c::WINDOW) -> Window {
    // Something, something.  TODO explain
    c::intrflush(c_window, 0);

    // Always enable keypad (function keys, arrow keys, etc.)
    // TODO what are multiple screens?
    c::keypad(c_window, 1);

    return Window{ c_window: c_window };
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

    fn refresh() {
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
            fail;
        }
        // TODO what if you get WEOF...?
    }

    fn readln() -> ~str unsafe {
        // TODO what should maximum buffer length be?
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

    fn attron(arg: c_int) {
        // TODO return value
        // TODO this is a fucking stupid way to do this
        c::wattron(self.c_window, arg as c_int);
    }
    fn attroff(arg: c_int) {
        // TODO return value
        // TODO this is a fucking stupid way to do this
        c::wattroff(self.c_window, arg as c_int);
    }

    fn end() {
        // TODO this feels too manual; i think this should be a bit smarter and use drop() to end curses mode.
        // TODO return value, though fuck if i know how this could ever fail
        c::endwin();
    }
}

pub fn init_screen() -> Window {
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

    // TODO these are also global, yikes
    c::cbreak();
    //c::noecho();
    c::nonl();

    return Window(c_window);
}
