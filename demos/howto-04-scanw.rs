/** A simple scanw example
 *
 * Taken from the ncurses programming howto, section 7.4:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/scanw.html
 */

extern mod amulet;

fn main() {
    let mesg = "Enter a string: ";

    let window = amulet::ll::init_screen();
    let (rows, cols) = window.size();

    let buf: ~str;

    window.mv(rows / 2, (cols - str::len(mesg)) / 2);
    window.write(mesg);

    buf = window.readln();

    // TODO bindgen doesn't give me access to LINES
    //mvprintw(LINES - 2, 0, "You Entered: %s", str);
    window.mv(rows - 2, 0);
    window.write(fmt!("You entered: %s", buf));

    window.getch();
}
