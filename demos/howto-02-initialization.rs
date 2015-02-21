/** Initialization function usage
 *
 * Taken from the ncurses programming howto, section 4.7:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/init.html
 */

extern crate amulet;

fn main() {
    let bold = amulet::ll::Style().bold();

    let mut term = amulet::Terminal::new();
    let mut canvas = term.enter_fullscreen();
    // TODO implement me -- right now there's NEVER echo, so
    // TODO also i don't like this curses-style api; maybe a "set_options"
    // or something
    // term.noecho();

    canvas.write("Type any character to see it in bold\n");
    canvas.repaint();

    let ch = canvas.read_key();
    match ch {
        amulet::ll::Key::FunctionKey(n) if n == 1 => {
            canvas.write("F1 key pressed");
        }
        _ => {
            canvas.write("The pressed key is ");
            canvas.attrwrite(format!("{:?}", ch).as_slice(), bold);
        }
    }

    canvas.repaint();
    canvas.pause();
}
