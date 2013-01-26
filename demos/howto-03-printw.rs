/** A simple printw example
 *
 * Taken from the ncurses programming howto, section 6.3:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/printw.html
 */

extern mod amulet;

use libc::c_int;

fn main() {
    let mesg = "Just a string";

    let term = amulet::ll::Terminal();
    do term.fullscreen |window| {
        let (rows, cols) = window.size();

        // TODO seems like centering should be easier?
        //mvprintw(row/2,(col-strlen(mesg))/2,"%s",mesg);
        //mvprintw(row-2,0,"This screen has %d rows and %d columns\n",row,col);

        // TODO do i want the combined mvprint et al.?
        window.mv(rows / 2, (cols - str::len(mesg))/2);
        window.write(mesg);
        window.mv(rows - 2, 0);
        window.write(fmt!("This screen has %u rows and %u columns\n", rows, cols));

        window.write("Try resizing your window (if possible) and then run this program again");
        window.repaint();
        window.pause();
    }
}
