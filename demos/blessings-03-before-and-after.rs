/** "Before and after".  (This is after.)
 *
 * Taken from the blessings README:
 * https://github.com/erikrose/blessings/blob/master/README.rst
 */

extern mod amulet;

fn main() {
    let term = amulet::ll::Terminal();

    do term.at(0, term.height() - 1) {
        term.write("This is ");
        term.attrwrite("pretty!", amulet::ll::Style().underline());
    }
}
