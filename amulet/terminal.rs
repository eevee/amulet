use canvas::Canvas;
use ll::Style;
use ll::TerminalInfo;
use termios;

struct Terminal {
    info: @TerminalInfo,
}

impl Terminal {
    pub fn new() -> Terminal {
        let info = TerminalInfo::new();

        return Terminal{
            info: info,
        };
    }


    // ------------------------------------------------------------------------
    // Inspection
    #[inline]
    pub fn height(&self) -> uint {
        return self.info.height();
    }

    #[inline]
    pub fn width(&self) -> uint {
        return self.info.width();
    }


    pub fn at(&self, x: uint, y: uint, cb: &fn()) {
        self.info.write_cap("sc");  // save cursor
        // TODO check for existence of cup
        self.info.write_cap2("cup", y as int, x as int);

        cb();

        self.info.write_cap("rc");  // restore cursor
    }

    // Output

    #[inline]
    pub fn write(&self, s: &str) {
        self.info.write(s);
    }

    pub fn attrwrite(&self, s: &str, style: Style) {
        // TODO try to cut down on the amount of back-and-forth between c
        // strings and rust strings all up in here
        if style.is_underline {
            self.info.write_cap("smul");
        }

        // TODO this may need some escaping or whatever -- or maybe that
        // belongs in write()
        self.write(s);

        // Clean up after ourselves: reset style to default
        // TODO this is ripe for some optimizing
        self.info.write_cap("sgr0");
    }

    // Full-screen

    pub fn fullscreen_canvas(&self, cb: &fn(&mut Canvas)) {
        // Enter fullscreen
        let _tidy_cup = self.info.write_tidy_cap("smcup", "rmcup");

        // Enable keypad mode
        let _tidy_kx = self.info.write_tidy_cap("smkx", "rmkx");

        // And clear the screen first
        self.info.write_cap("clear");

        // TODO intrflush, or is that a curses thing?

        // TODO so, we need to switch to raw mode *some*where.  is this an
        // appropriate place?  i assume if you have a fullscreen app then you
        // want to get keypresses.
        // TODO seems weird to create a second one of these.  stick a
        // .checkpoint() on the one attached to the terminal?
        let tidy_termstate = termios::TidyTerminalState(self.info.in_fd);
        tidy_termstate.cbreak();

        let mut canv = Canvas(self.info, 0, 0, self.height(), self.width());
        cb(&mut canv);
    }

    // Enter fullscreen manually.  Cleaning up with exit_fullscreen is YOUR
    // responsibility!  If you don't do it in a drop, you risk leaving the
    // terminal in a fucked-up state on early exit!
    pub fn enter_fullscreen(&self) -> Canvas {
        // Same stuff as above.  Enter fullscreen; enter keypad mode; clear the
        // screen.
        let tidy_cup = self.info.write_tidy_cap("smcup", "rmcup");
        let tidy_kx = self.info.write_tidy_cap("smkx", "rmkx");
        self.info.write_cap("clear");

        // TODO intrflush, as above...?

        let tidy_termstate = termios::TidyTerminalState(self.info.in_fd);
        tidy_termstate.cbreak();

        let mut canv = Canvas(self.info, 0, 0, self.height(), self.width());
        canv.tidyables.tidy_termcaps.push_all_move(~[tidy_kx, tidy_cup]);
        canv.tidyables.tidy_termstates.push(tidy_termstate);
        return canv;
    }
}
