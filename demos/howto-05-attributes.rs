/** A simple attributes example
 *
 * Taken from the ncurses programming howto, section 8:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/attrib.html
 */

extern mod amulet;

/* pager functionality by Joseph Spainhou <spainhou@bellsouth.net> */

fn main() {
    let mut ch;
    let mut prev = 0;

    let args = os::args();
    if args.len() != 2 {
        io::println(fmt!("Usage: %s <a C filename>", args[0]));
        unsafe { libc::exit(1) };
    }

    let fh;
    match io::file_reader(&path::Path(args[1])) {
        result::Ok(res) => { fh = res; }
        result::Err(msg) => {
            io::println(msg);
            unsafe { libc::exit(1) };
        }
    }

    // cannot open input file...

    let window = amulet::ll::init_screen();
    let (rows, _cols) = window.size();

    let plain = ~amulet::ll::Style();
    let bold = plain.bold();
    let mut cur_style = copy plain;

    while ! fh.eof() {
        ch = fh.read_byte();
        let (row, col) = window.position();

        if row == rows - 1 {
            window.write("<-Press Any Key->");
            window.getch();
            window.clear();
            window.mv(0, 0);
        }

        if prev as char == '/' && ch as char == '*' {
            //window.write(#fmt("%c", ch as char));
            cur_style = copy bold;

            window.mv(row, col - 1);
            window.attrwrite(fmt!("%c%c", prev as char, ch as char), cur_style);
        }
        else {
            window.attrwrite(fmt!("%c", ch as char), cur_style);
        }

        window.repaint();

        if prev as char == '*' && ch as char == '/' {
            cur_style = copy plain;
        }

        prev = ch;
    }
}
