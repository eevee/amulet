/** A simple attributes example
 *
 * Taken from the ncurses programming howto, section 8:
 * http://tldp.org/HOWTO/NCURSES-Programming-HOWTO/attrib.html
 */

extern mod amulet;

/* pager functionality by Joseph Spainhou <spainhou@bellsouth.net> */

fn main() {
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

    let term = amulet::ll::Terminal();
    do term.fullscreen_canvas |canvas| {
        let mut ch;
        let (rows, _cols) = canvas.size();

        let plain = ~amulet::ll::Style();
        let bold = plain.bold();
        let mut cur_style = copy plain;

        while ! fh.eof() {
            ch = fh.read_byte();
            let (row, col) = canvas.position();

            if row == rows - 1 {
                canvas.write("<-Press Any Key->");
                canvas.repaint();
                canvas.pause();
                canvas.clear();
                canvas.move(0, 0);
            }

            if prev as char == '/' && ch as char == '*' {
                //canvas.write(#fmt("%c", ch as char));
                cur_style = copy bold;

                canvas.move(row, col - 1);
                canvas.attrwrite(fmt!("%c%c", prev as char, ch as char), cur_style);
            }
            else {
                canvas.attrwrite(fmt!("%c", ch as char), cur_style);
            }

            canvas.repaint();

            if prev as char == '*' && ch as char == '/' {
                cur_style = copy plain;
            }

            prev = ch;
        }
    }
}
