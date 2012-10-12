/** Initialization function usage
 *
 * Taken from the ncurses programming howto, section 4.7:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/init.html
 */

extern mod amulet;

use libc::c_int;

fn main() {
    let window = amulet::ll::init_screen();
    amulet::c::raw();
    amulet::c::noecho();

    window.print("Type any character to see it in bold\n");

    let ch: int = window.getch();
    if ch == amulet::c::KEY_F(1 as c_int) as int {
        window.print("F1 key pressed");
    }
    else {
        window.print("The pressed key is ");
        window.attron(amulet::c::A_BOLD);
        window.print(#fmt("%c", ch as char));
        window.attroff(amulet::c::A_BOLD);
    }

    window.refresh();
    window.getch();
    window.end();
}
