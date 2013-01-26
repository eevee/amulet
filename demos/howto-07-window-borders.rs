/** Window border example
 *
 * Taken from the ncurses programming howto, section 9.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/windows.html
 */

extern mod amulet;

use libc::c_int;

fn main() {
    let window = amulet::ll::init_screen();

    let height = 3;
    let width = 10;
    let (rows, columns) = amulet::ll::screen_size();
    // Calculation for a center placement of the window
    let mut starty = (rows - height) / 2;
    let mut startx = (columns - width) / 2;

    window.write("Press F1 to exit");
    window.repaint();

    let mut my_win = create_newwin(height, width, starty, startx);

    loop {
        let ch = window.getch();
        if ch == amulet::c::KEY_F(1) as char {
            break;
        }
        else if ch == amulet::c::KEY_LEFT as char {
            destroy_win(my_win);
            startx -= 1;
            my_win = create_newwin(height, width, starty, startx);
        }
        else if ch == amulet::c::KEY_RIGHT as char {
            destroy_win(my_win);
            startx += 1;
            my_win = create_newwin(height, width, starty, startx);
        }
        else if ch == amulet::c::KEY_UP as char {
            destroy_win(my_win);
            starty -= 1;
            my_win = create_newwin(height, width, starty, startx);
        }
        else if ch == amulet::c::KEY_DOWN as char {
            destroy_win(my_win);
            starty += 1;
            my_win = create_newwin(height, width, starty, startx);
        }
    }
}

fn create_newwin(height: uint, width: uint, starty: uint, startx: uint) -> @amulet::ll::Window {
    let local_win = amulet::ll::new_window(height, width, starty, startx);
    // 0,0 gives default chars for the vertical and horizontal lines
    local_win.set_box(0 as char, 0 as char);

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
