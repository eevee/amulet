/** A simple printw example
 *
 * Taken from the ncurses programming howto, section 6.3:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/printw.html
 */

extern crate amulet;

fn main() {
    let mesg = "Just a string";

    let mut term = amulet::Terminal::new();
    let mut canvas = term.enter_fullscreen();
    let (rows, cols) = canvas.size();

    // TODO seems like centering should be easier?
    //mvprintw(row/2,(col-strlen(mesg))/2,"%s",mesg);
    //mvprintw(row-2,0,"This screen has %d rows and %d columns\n",row,col);

    // TODO do i want the combined mvprint et al.?
    canvas.reposition(rows / 2, (cols - mesg.len())/2);
    canvas.write(mesg);
    canvas.reposition(rows - 2, 0);
    canvas.write(format!("This screen has {} rows and {} columns\n", rows, cols).as_slice());

    canvas.write("Try resizing your window (if possible) and then run this program again");
    canvas.repaint();
    canvas.pause();
}
