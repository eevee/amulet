/** Hello, world!
 *
 * Taken from the ncurses programming howto, section 2.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/helloworld.html
 */

extern crate amulet;

fn main() {
    let mut term = amulet::Terminal::new();
    let mut canvas = term.enter_fullscreen();
    canvas.write("Hello World !!!");
    canvas.repaint();
    canvas.pause();
}
