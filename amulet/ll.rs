/** Low-level ncurses wrapper, for simple or heavily customized applications. */

import libc::c_int;

import c;

class Window {
    let c_window: *c::WINDOW;

    new(c_window: *c::WINDOW) {
        self.c_window = c_window;

        // Always enable keypad (function keys, arrow keys, etc.)
        // TODO what are multiple screens?
        c::bindgen::keypad(c_window, true);
    }

    fn print(msg: str) {
        // TODO return value
        // TODO this is variadic; string template exploits abound, should use %s really
        do str::as_c_str(msg) |bytes| {
            c::bindgen::wprintw(self.c_window, bytes);
        }
    }

    fn refresh() {
        // TODO return value
        c::bindgen::wrefresh(self.c_window);
    }

    fn getch() -> int {
        // TODO return value, kind of important here
        // TODO this name sucks
        ret c::bindgen::wgetch(self.c_window) as int;
    }

    fn attron(arg: c_int) {
        // TODO return value
        // TODO this is a fucking stupid way to do this
        c::bindgen::wattron(self.c_window, arg as c_int);
    }
    fn attroff(arg: c_int) {
        // TODO return value
        // TODO this is a fucking stupid way to do this
        c::bindgen::wattroff(self.c_window, arg as c_int);
    }

    fn end() {
        // TODO this feels too manual; i think this should be a bit smarter and use drop() to end curses mode.
        // TODO return value, though fuck if i know how this could ever fail
        c::bindgen::endwin();
    }
}

fn init_screen() -> Window {
    let c_window = c::bindgen::initscr();

    if c_window == ptr::null() {
        // Should only fail due to memory pressure
        fail;
    }

    ret Window(c_window);
}
