/** Hello, world!
 *
 * Taken from the ncurses programming howto, section 2.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/helloworld.html
 */

extern mod amulet;

fn main() {
    let term = amulet::Terminal::new();
    do term.fullscreen_canvas |canvas| {
        canvas.write("Hello World !!!");
        canvas.repaint();
        canvas.pause();
    }
}
