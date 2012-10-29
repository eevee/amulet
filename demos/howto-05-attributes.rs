/** A simple attributes example
 *
 * Taken from the ncurses programming howto, section 8:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/attrib.html
 */

extern mod amulet;

use libc::c_int;

/* pager functionality by Joseph Spainhour" <spainhou@bellsouth.net> */

fn main() {
    let mut ch;
    let mut prev = 0;

    let args = os::args();
    if args.len() != 2 {
        io::println(fmt!("Usage: %s <a C filename>", args[0]));
        os::set_exit_status(1);
        return;
    }

    let fh;
    match io::file_reader(&path::Path(args[1])) {
        result::Ok(res) => { fh = res; }
        result::Err(msg) => {
            io::println(msg);
            os::set_exit_status(1);
            return;
        }
    }

    // cannot open input file...

    let window = amulet::ll::init_screen();
    let (rows, _cols) = window.size();

    while ! fh.eof() {
        ch = fh.read_byte();
        let (row, col) = window.position();

        if row == rows - 1 {
            window.print("<-Press Any Key->");
            window.getch();
            window.clear();
            window.mv(0, 0);
        }

        if prev as char == '/' && ch as char == '*' {
            window.attron(amulet::c::A_BOLD);
            //window.print(#fmt("%c", ch as char));

            window.mv(row, col - 1);
            window.print(fmt!("%c%c", prev as char, ch as char));
        }
        else {
            window.print(fmt!("%c", ch as char));
        }

        window.refresh();

        if prev as char == '*' && ch as char == '/' {
            window.attroff(amulet::c::A_BOLD);
        }

        prev = ch;
    }

    window.end();

    // close fh?
}
