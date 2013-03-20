/** Initialization function usage
 *
 * Taken from the ncurses programming howto, section 4.7:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/init.html
 */

extern mod amulet;

fn main() {
    let bold = amulet::ll::Style().bold();

    let term = amulet::ll::Terminal();
    do term.fullscreen |window| {
        // XXX these have got to go
        amulet::c::noecho();

        window.write("Type any character to see it in bold\n");

        let ch = window.read_key();
        match ch {
            amulet::ll::FunctionKey(n) if n == 1 => {
                window.write("F1 key pressed");
            }
            _ => {
                window.write("The pressed key is ");
                window.attrwrite(fmt!("%?", ch), bold);
            }
        }

        window.repaint();
        window.pause();
    }
}
