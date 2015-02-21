/** Window border example
 *
 * Taken from the ncurses programming howto, section 9.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/windows.html
 */

extern crate amulet;

use amulet::canvas::Canvas;

fn main() {
    let mut term = amulet::Terminal::new();
    let mut canvas = term.enter_fullscreen();

    let height = 3;
    let width = 10;
    // TODO this used to be (term.height(), term.width()), but that was disallowed for "immutable
    // borrow" when the canvas itself is already a mutable borrow.  this seems...  all suboptimal.
    let (rows, columns) = canvas.size();
    // Calculation for a center placement of the window
    let mut starty = (rows - height) / 2;
    let mut startx = (columns - width) / 2;

    canvas.write("Press F1 to exit");
    canvas.repaint();

    let mut my_win = create_newwin(&canvas, height, width, starty, startx);

    loop {
        match canvas.read_key() {
            amulet::ll::Key::FunctionKey(1) => {
                break;
            }
            amulet::ll::Key::SpecialKey(amulet::ll::SpecialKeyCode::LEFT) => {
                destroy_win(&mut my_win);
                startx -= 1;
                my_win = create_newwin(&canvas, height, width, starty, startx);
            }
            amulet::ll::Key::SpecialKey(amulet::ll::SpecialKeyCode::RIGHT) => {
                destroy_win(&mut my_win);
                startx += 1;
                my_win = create_newwin(&canvas, height, width, starty, startx);
            }
            amulet::ll::Key::SpecialKey(amulet::ll::SpecialKeyCode::UP) => {
                destroy_win(&mut my_win);
                starty -= 1;
                my_win = create_newwin(&canvas, height, width, starty, startx);
            }
            amulet::ll::Key::SpecialKey(amulet::ll::SpecialKeyCode::DOWN) => {
                destroy_win(&mut my_win);
                starty += 1;
                my_win = create_newwin(&canvas, height, width, starty, startx);
            }
            _ => (),
        }
    }
}

fn create_newwin<'a, 'b>(canvas: &Canvas<'a, 'b>, height: usize, width: usize, starty: usize, startx: usize) -> Canvas<'a, 'b> {
    let mut local_win = canvas.spawn(startx, starty, width, height);
    // 0,0 gives default chars for the vertical and horizontal lines
    //local_win.set_box(0 as char, 0 as char);
    // TODO: box borders don't belong on Canvas since they are more a UI thing.
    // probably a really good first widget though.
    for _n in range(0, width) {
        local_win.write("box...\n");
    }

    // Show that box
    local_win.repaint();

    return local_win;
}

fn destroy_win(local_win: &mut Canvas) {
    // 'box' won't erase the window; it'll leave the corners behind.

    // border's params are L, R, T, B, TL, TR, BL, BR
    //local_win.set_border(' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ');

    local_win.repaint();
}
