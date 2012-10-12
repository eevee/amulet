/** Low-level ncurses wrapper, for simple or heavily customized applications. */

use libc::{c_char,c_int,c_void,size_t};

use c;

pub struct Window {
    c_window: *c::WINDOW,
}
pub fn Window(c_window: *c::WINDOW) -> Window {
    // Always enable keypad (function keys, arrow keys, etc.)
    // TODO what are multiple screens?
    c::keypad(c_window, 1);

    return Window{ c_window: c_window };
}

impl Window {
    ////// Properties

    /** Returns the size of the window as (rows, columns). */
    fn size() -> (uint, uint) {
        return (c::getmaxy(self.c_window) as uint,
            c::getmaxx(self.c_window) as uint);
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

    // Input

    fn getch() -> int {
        // TODO return value, kind of important here
        // TODO this name sucks
        return c::wgetch(self.c_window) as int;
    }

    fn readln() -> ~str unsafe {
        // TODO return value
        // TODO what should maximum buffer length be?
        const buflen: uint = 80;
        let buf = libc::malloc(buflen as size_t) as *c_char;
        c::wgetnstr(self.c_window, buf, buflen as c_int);

        let out = str::raw::from_c_str_len(buf, buflen);
        libc::free(buf as *c_void);

        return out;
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
    let c_window = c::initscr();

    if c_window == ptr::null() {
        // Should only fail due to memory pressure
        fail;
    }

    return Window(c_window);
}
