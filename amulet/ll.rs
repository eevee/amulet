/** Low-level ncurses wrapper, for simple or heavily customized applications. */

import c;

class Window {
    let c_window: *c::WINDOW;

    new(c_window: *c::WINDOW) {
        self.c_window = c_window;
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

    fn getch() {
        // TODO return value, kind of important here
        // TODO this name sucks
        c::bindgen::wgetch(self.c_window);
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
