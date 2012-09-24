/** A simple scanw example
 *
 * Taken from the ncurses programming howto, section 7.4:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/scanw.html
 */

use amulet;

import libc::c_int;

fn main(_args: ~[str]) {
    let mesg = "Enter a string: ";

    let window = amulet::ll::init_screen();
    let (rows, cols) = window.size();

    let buf: str;

    window.move(rows / 2, (cols - str::len(mesg)) / 2);
    window.print(mesg);

    buf = window.readln();

    // TODO bindgen doesn't give me access to LINES
    //mvprintw(LINES - 2, 0, "You Entered: %s", str);
    window.move(rows - 2, 0);
    window.print(#fmt("You entered: %s", buf));

    window.getch();
    window.end();
}
