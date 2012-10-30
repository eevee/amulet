/** Initialization function usage
 *
 * Taken from the ncurses programming howto, section 4.7:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/init.html
 */

extern mod amulet;

use libc::c_int;

fn main() {
    let bold = amulet::ll::Style().bold();

    let window = amulet::ll::init_screen();
    amulet::c::raw();
    amulet::c::noecho();

    window.print("Type any character to see it in bold\n");

    let ch = window.getch();
    // TODO this ain't quite right, yo.  function keys are distinct from
    // characters
    if ch == amulet::c::KEY_F(1 as c_int) as char {
        window.print("F1 key pressed");
    }
    else {
        window.print("The pressed key is ");
        window.attrprint(#fmt("%c", ch), bold);
    }

    window.repaint();
    window.getch();
}
