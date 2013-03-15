/** chgat() usage example
 *
 * Taken from the ncurses programming howto, section 8.6:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/attrib.html
 */

extern mod amulet;

fn main() {
    let window = amulet::ll::Terminal().enter_fullscreen();

    amulet::ll::define_color_pair(1, amulet::c::COLOR_CYAN, amulet::c::COLOR_BLACK);
    window.write("A big string which I didn't care to type fully");

    window.mv(0, 0);
    // First: Number of characters to update (-1 means until end of line)
    // Second: Attribute bitmask
    // Third: Color index to use
    window.restyle(-1, amulet::c::A_BLINK as int, 1);

    window.repaint();
    window.pause();
}
