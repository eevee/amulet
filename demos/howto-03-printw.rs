/** A simple printw example
 *
 * Taken from the ncurses programming howto, section 6.3:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/printw.html
 */

extern mod amulet;

fn main() {
    let mesg = "Just a string";

    let term = amulet::Terminal::new();
    do term.fullscreen_canvas |canvas| {
        let (rows, cols) = canvas.size();

        // TODO seems like centering should be easier?
        //mvprintw(row/2,(col-strlen(mesg))/2,"%s",mesg);
        //mvprintw(row-2,0,"This screen has %d rows and %d columns\n",row,col);

        // TODO do i want the combined mvprint et al.?
        canvas.move(rows / 2, (cols - mesg.len())/2);
        canvas.write(mesg);
        canvas.move(rows - 2, 0);
        canvas.write(fmt!("This screen has %u rows and %u columns\n", rows, cols));

        canvas.write("Try resizing your window (if possible) and then run this program again");
        canvas.repaint();
        canvas.pause();
    }
}
