/** Hello, world!
 *
 * Taken from the ncurses programming howto, section 2.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/helloworld.html
 */

extern mod amulet;

fn main() {
    let term = amulet::ll::Terminal();
    do term.fullscreen |window| {
        window.write("Hello World !!!");
        window.repaint();
        window.pause();
    }
}
