/** Window border example
 *
 * Taken from the ncurses programming howto, section 9.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/windows.html
 */

extern mod amulet;

use libc::c_int;

fn main() {
    let term = amulet::ll::Terminal();
    let window = term.enter_fullscreen();

    let height = 3;
    let width = 10;
    let (rows, columns) = (term.height(), term.width());
    // Calculation for a center placement of the window
    let mut starty = (rows - height) / 2;
    let mut startx = (columns - width) / 2;

    window.write("Press F1 to exit");
    window.repaint();

    let mut my_win = create_newwin(window, height, width, starty, startx);

    loop {
        match window.read_key() {
            amulet::ll::FunctionKey(1) => {
                break;
            }
            amulet::ll::SpecialKey(amulet::ll::KEY_LEFT) => {
                destroy_win(my_win);
                startx -= 1;
                my_win = create_newwin(window, height, width, starty, startx);
            }
            amulet::ll::SpecialKey(amulet::ll::KEY_RIGHT) => {
                destroy_win(my_win);
                startx += 1;
                my_win = create_newwin(window, height, width, starty, startx);
            }
            amulet::ll::SpecialKey(amulet::ll::KEY_UP) => {
                destroy_win(my_win);
                starty -= 1;
                my_win = create_newwin(window, height, width, starty, startx);
            }
            amulet::ll::SpecialKey(amulet::ll::KEY_DOWN) => {
                destroy_win(my_win);
                starty += 1;
                my_win = create_newwin(window, height, width, starty, startx);
            }
            _ => (),
        }
    }
}

fn create_newwin(window: &amulet::ll::Window, height: uint, width: uint, starty: uint, startx: uint) -> @amulet::ll::Window {
    let local_win = window.create_window(height, width, starty, startx);
    // 0,0 gives default chars for the vertical and horizontal lines
    //local_win.set_box(0 as char, 0 as char);
    // TODO: box borders don't belong on Window since they are more a UI thing.
    // Window is just a region for you to draw in.  tbh i'm not even sure it
    // should be a thing.
    for uint::range(0, width) |_n| {
        local_win.write("box...\n");
    }

    // Show that box
    local_win.repaint();

    return local_win;
}

fn destroy_win(local_win: &amulet::ll::Window) {
    // 'box' won't erase the window; it'll leave the corners behind.

    // border's params are L, R, T, B, TL, TR, BL, BR
    local_win.set_border(' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ');

    local_win.repaint();
}
