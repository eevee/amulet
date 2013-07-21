/** chgat() usage example
 *
 * Taken from the ncurses programming howto, section 8.6:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/attrib.html
 */

extern mod amulet;

use amulet::ll::Style;

fn main() {
    let term = amulet::ll::Terminal();
    let mut canvas = term.enter_fullscreen();

    canvas.write("A big string which I didn't care to type fully");

    canvas.move(0, 0);
    // TODO the original curses function also takes an argument for how many
    // characters to change, with -1 meaning "to end of line" (which i am not
    // in love with)
    canvas.restyle(Style().fg(13));

    canvas.repaint();
    canvas.pause();
}
