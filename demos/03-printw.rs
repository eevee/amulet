/** A simple printw example
 *
 * Taken from the ncurses programming howto, section 6.3:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/printw.html
 */

use amulet;

import libc::c_int;

fn main(_args: ~[str]) {
    let mesg: str = "Just a string";

    let window = amulet::ll::init_screen();
    let (rows, cols) = window.size();

    // TODO seems like centering should be easier?
    //mvprintw(row/2,(col-strlen(mesg))/2,"%s",mesg);
    //mvprintw(row-2,0,"This screen has %d rows and %d columns\n",row,col);

    // TODO do i want the combined mvprint et al.?
    window.move(rows / 2, (cols - str::len(mesg))/2);
    window.print(mesg);
    window.move(rows - 2, 0);
    window.print(#fmt("This screen has %u rows and %u columns\n", rows, cols));

    window.print("Try resizing your window (if possible) and then run this program again");
    window.refresh();
    window.getch();
    window.end();
}
