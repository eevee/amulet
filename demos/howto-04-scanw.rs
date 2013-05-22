/** A simple scanw example
 *
 * Taken from the ncurses programming howto, section 7.4:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/scanw.html
 */

extern mod amulet;

fn main() {
    let mesg = "Enter a string: ";

    let term = amulet::ll::Terminal();
    do term.fullscreen_canvas |canvas| {
        let (rows, cols) = canvas.size();

        let buf: ~str;

        canvas.move(rows / 2, (cols - str::len(mesg)) / 2);
        canvas.write(mesg);
        canvas.repaint();

        buf = canvas.read_line();

        canvas.move(rows - 2, 0);
        canvas.write(fmt!("You entered: %s", buf));
        canvas.repaint();

        canvas.pause();
    }
}
