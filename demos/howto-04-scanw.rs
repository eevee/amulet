/** A simple scanw example
 *
 * Taken from the ncurses programming howto, section 7.4:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/scanw.html
 */

extern crate amulet;

fn main() {
    let mesg = "Enter a string: ";

    let mut term = amulet::Terminal::new();
    let mut canvas = term.enter_fullscreen();
    let (rows, cols) = canvas.size();

    canvas.reposition(rows / 2, (cols - mesg.len()) / 2);
    canvas.write(mesg);
    canvas.repaint();

    let buf = canvas.read_line();

    canvas.reposition(rows - 2, 0);
    canvas.write(format!("You entered: {}", buf).as_slice());
    canvas.repaint();

    canvas.pause();
}
