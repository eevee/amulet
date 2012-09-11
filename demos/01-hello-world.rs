/** Hello, world!
 *
 * Taken from the ncurses programming howto, section 2.1:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/helloworld.html
 */

use amulet;

fn main(_args: ~[str]) {
    let window = amulet::ll::init_screen();
    window.print("Hello World !!!");
    window.refresh();
    window.getch();
    window.end();
}
