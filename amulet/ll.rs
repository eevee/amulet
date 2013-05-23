/** Low-level ncurses wrapper, for simple or heavily customized applications. */

extern mod std;

use core::libc::{c_char,c_int,c_long,c_schar,c_short,c_void,size_t,wchar_t};
use core::io::ReaderUtil;

use c;
use termios;
use trie::Trie;

use canvas::Canvas;

extern {
    fn setlocale(category: c_int, locale: *c_char) -> *c_char;

    // XXX why the fuck is this not available
    static stdout: *libc::FILE;
}


/** Prints a given termcap sequence when it goes out of scope. */
struct TidyTermcap<'self> {
    term: &'self Terminal,
    cap: &'self str,
}
#[unsafe_destructor]
impl<'self> Drop for TidyTermcap<'self> {
    fn finalize(&self) {
        self.term.write_cap(self.cap);
    }
}


struct Terminal {
    in_fd: c_int,
    in_file: @io::Reader,
    out_fd: c_int,
    out_file: @io::Writer,

    keypress_trie: @mut Trie<u8, Key>,

    priv c_terminfo: *c::TERMINAL,
    priv tidy_termstate: termios::TidyTerminalState,

    //term_type: ~str,
}

pub fn Terminal() -> @Terminal {
    let error_code: c_int = 0;
    // NULL first arg means read TERM from env (TODO).
    // second arg is a fd to spew to on error, but it's not used when there's
    // an error pointer.
    // third arg is a var to stick the error code in.
    // TODO allegedly setupterm doesn't work on BSD?
    unsafe {
        let res = c::setupterm(ptr::null(), -1, ptr::addr_of(&error_code));

        if res != c::OK {
            if error_code == -1 {
                fail!(~"Couldn't find terminfo database");
            }
            else if error_code == 0 {
                fail!(~"Couldn't identify terminal");
            }
            else if error_code == 1 {
                // The manual puts this as "terminal is hard-copy" but come on.
                fail!(~"Terminal appears to be made of paper");
            }
            else {
                fail!(~"Something is totally fucked");
            }
        }
    }

    // Okay; now terminfo is sitting in a magical global somewhere.  Snag a
    // pointer to it.
    let terminfo = c::cur_term;

    let keypress_trie = Trie();
    let mut p: **c_char = ptr::to_unsafe_ptr(&c::strnames[0]);
    unsafe {
        while *p != ptr::null() {
            let capname = str::raw::from_c_str(*p);

            if capname[0] == "k"[0] {
                let cap = c::tigetstr(*p);
                if cap != ptr::null() {
                    let cap_key = vec::raw::from_buf_raw(cap, libc::strlen(cap) as uint).map(|el| *el as u8);
                    keypress_trie.insert(cap_key, cap_to_key(capname));
                }
            }

            p = ptr::offset(p, 1);
        }
    }

    let term = @Terminal{
        // TODO would be nice to parametrize these, but Reader and Writer do
        // not yet expose a way to get the underlying fd, which makes the API
        // sucky
        in_fd: 0,
        in_file: io::stdin(),
        out_fd: 1,
        out_file: io::stdout(),

        keypress_trie: keypress_trie,

        c_terminfo: terminfo,
        tidy_termstate: termios::TidyTerminalState(0),
    };

    return term;
}
#[unsafe_destructor]
impl Drop for Terminal {
    fn finalize(&self) {
        self.tidy_termstate.restore();
    }
}
impl Terminal {
    // ------------------------------------------------------------------------
    // Capability inspection
    // TODO ideally, ultimately, every useful cap will be covered here...

    // TODO these should:
    // - only do the ioctl once and cache it
    // - handle failure in ioctl
    // - handle TIOCGSIZE instead of that other thing
    // - handle SIGWINCH
    // - fall back to environment
    // - THEN fall back to termcap
    pub fn height(&self) -> uint {
        // TODO rather not dip into `imp`, but `pub use` isn't working right
        let (_, height) = termios::imp::request_terminal_size(self.out_fd);
        return height;
        //return self.numeric_cap("lines");
    }
    pub fn width(&self) -> uint {
        let (width, _) = termios::imp::request_terminal_size(self.out_fd);
        return width;
        //return self.numeric_cap("cols");
    }

    // ------------------------------------------------------------------------
    // Very-low-level capability inspection

    fn flag_cap(&self, name: &str) -> bool {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = 0;
            do str::as_c_str(name) |bytes| {
                value = c::tigetflag(bytes);
            }

            if value == -1 {
                // wrong type
                fail!(~"wrong type");
            }

            // Otherwise, is 0 or 1
            return value as bool;
        }
    }

    fn numeric_cap(&self, name: &str) -> uint {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = -1;
            do str::as_c_str(name) |bytes| {
                value = c::tigetnum(bytes);
            }

            if value == -2 {
                // wrong type
                fail!(~"wrong type");
            }
            else if value == -1 {
                // missing; should be None
                fail!(~"missing; should be None");
            }

            return value as uint;
        }
    }

    fn _string_cap_cstr(&self, name: &str) -> *c_char {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = ptr::null();
            do str::as_c_str(name) |bytes| {
                value = c::tigetstr(bytes);
            }

            if value == ptr::null() {
                // missing; should be None really
                fail!(~"missing; should be None really");
            }
            else if value == cast::transmute(-1) {
                // wrong type
                fail!(~"wrong type");
            }

            return value;
        }
    }

    fn string_cap(&self, name: &str) -> ~str {
        let value = self._string_cap_cstr(name);

        unsafe {
            return str::raw::from_c_str(value);
        }
    }

    // TODO i am not really liking the string capability handling anywhere in
    // here.
    // the variable arguments thing sucks ass.  we should at LEAST check for
    // how many arguments are expected, and ideally enforce it at compile time
    // somehow -- perhaps with a method for every string cap.  (yikes.  having
    // keys be a separate thing would help, though.)

    /** Returns a string capability, formatted with the passed vector of
     * arguments.
     *
     * Passing the correct number of arguments is your problem, though any
     * missing arguments become zero.  No capability requires more than 9
     * arguments.
     */
    fn format_cap(&self, name: &str, args: &[int]) -> ~str {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let template = self._string_cap_cstr(name);
            let padded_args = args.to_vec() + [0, .. 8];
            let formatted = c::tparm(
                template,
                padded_args[0] as c_long,
                padded_args[1] as c_long,
                padded_args[2] as c_long,
                padded_args[3] as c_long,
                padded_args[4] as c_long,
                padded_args[5] as c_long,
                padded_args[6] as c_long,
                padded_args[7] as c_long,
                padded_args[8] as c_long
            );

            return str::raw::from_c_str(formatted);
        }
    }

    fn _write_capx(&self, name: &str,
            arg1: c_long, arg2: c_long, arg3: c_long,
            arg4: c_long, arg5: c_long, arg6: c_long,
            arg7: c_long, arg8: c_long, arg9: c_long)
    {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let template = self._string_cap_cstr(name);

            let formatted = c::tparm(
                template, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);

            //unsafe { io::stderr().write_str(fmt!("%s\t%s\n", name, str::raw::from_c_str(formatted))); }

            // TODO we are supposed to use curses's tputs(3) to print formatted
            // capabilities, because they sometimes contain "padding" of the
            // form $<5>.  alas, tputs always prints to stdout!  but we don't
            // yet allow passing in a custom fd so i guess that's okay.  would
            // be nice to reimplement this in Rust code someday though.
            c::putp(formatted);

            // TODO another reason dipping into C sucks: we have to manually
            // flush its buffer every time we do so.
            libc::fflush(stdout);
        }
    }

    // TODO seems it would make sense to cache non-formatted capabilities (as
    // well as numeric/flag ones), which i think blessings does
    fn write_cap(&self, cap_name: &str) {
        // If we're calling this function then this capability really shouldn't
        // take any arguments, but someone might have screwed up, or it may
        // have an escaped % or something.  Best do the whole formatting thing.
        self._write_capx(cap_name, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    }
    fn write_cap2(&self, cap_name: &str, arg1: int, arg2: int) {
        self._write_capx(cap_name, arg1 as c_long, arg2 as c_long, 0, 0, 0, 0, 0, 0, 0);
    }

    fn write_tidy_cap(&self, do_cap: &str, undo_cap: &'self str) -> TidyTermcap<'self> {
        self.write_cap(do_cap);

        return TidyTermcap{ term: self, cap: undo_cap };
    }

    // TODO should capabilities just have a method apiece, like blessings?



    // Output

    pub fn write(&self, s: &str) {
        io::stdout().flush();
        // TODO well.  should be a bit more flexible, i guess.
        io::print(s);
        io::stdout().flush();
    }

    pub fn attrwrite(&self, s: &str, style: &Style) {
        // TODO try to cut down on the amount of back-and-forth between c
        // strings and rust strings all up in here
        if style.is_underline {
            self.write_cap("smul");
        }

        // TODO this may need some escaping or whatever -- or maybe that
        // belongs in write()
        self.write(s);

        // Clean up after ourselves: reset style to default
        // TODO this is ripe for some optimizing
        self.write_cap("sgr0");
    }


    // Some stuff
    pub fn move(&self, x: uint, y: uint) {
        // TODO check for existence of cup
        self.write_cap2("cup", y as int, x as int);
    }

    pub fn at(&self, x: uint, y: uint, cb: &fn()) {
        self.write_cap("sc");  // save cursor
        // TODO check for existence of cup
        self.write_cap2("cup", y as int, x as int);

        cb();

        self.write_cap("rc");  // restore cursor
    }

    // Full-screen

    // TODO wonder if this could use a borrowed or unique window, since it's
    // local to this function?  (unique would also prevent its escape from
    // here.)  really really REALLY sucks that we can't let the caller decide
    // easily.
    pub fn fullscreen(@self, cb: &fn(@Window)) {
        // Enter fullscreen
        let _tidy_cup = self.write_tidy_cap("smcup", "rmcup");

        // Enable keypad mode
        let _tidy_kx = self.write_tidy_cap("smkx", "rmkx");

        // And clear the screen first
        self.write_cap("clear");

        // TODO intrflush, or is that a curses thing?

        // TODO so, we need to switch to raw mode *some*where.  is this an
        // appropriate place?  i assume if you have a fullscreen app then you
        // want to get keypresses.
        // TODO seems weird to create a second one of these.  stick a
        // .checkpoint() on the one attached to the terminal?
        let tidy_termstate = termios::TidyTerminalState(self.in_fd);
        tidy_termstate.cbreak();

        let win = @Window{
            c_window: ptr::null(),  // TODO obviously

            term: self,
            row: 0,
            col: 0,
            width: self.width(),
            height: self.height(),

            tidyables: ~[],
        };
        cb(win);
    }

    pub fn fullscreen_canvas(@self, cb: &fn(&mut Canvas)) {
        // Enter fullscreen
        let _tidy_cup = self.write_tidy_cap("smcup", "rmcup");

        // Enable keypad mode
        let _tidy_kx = self.write_tidy_cap("smkx", "rmkx");

        // And clear the screen first
        self.write_cap("clear");

        // TODO intrflush, or is that a curses thing?

        // TODO so, we need to switch to raw mode *some*where.  is this an
        // appropriate place?  i assume if you have a fullscreen app then you
        // want to get keypresses.
        // TODO seems weird to create a second one of these.  stick a
        // .checkpoint() on the one attached to the terminal?
        let tidy_termstate = termios::TidyTerminalState(self.in_fd);
        tidy_termstate.cbreak();

        let mut canv = Canvas(self, 0, 0, self.height(), self.width());
        cb(&mut canv);
    }

    // Enter fullscreen manually.  Cleaning up with exit_fullscreen is YOUR
    // responsibility!  If you don't do it in a drop, you risk leaving the
    // terminal in a fucked-up state on early exit!
    pub fn enter_fullscreen(@self) -> @Window {
        // Same stuff as above.  Enter fullscreen; enter keypad mode; clear the
        // screen.
        let tidy_cup = self.write_tidy_cap("smcup", "rmcup");
        let tidy_kx = self.write_tidy_cap("smkx", "rmkx");
        self.write_cap("clear");

        // TODO intrflush, as above...?

        let tidy_termstate = termios::TidyTerminalState(self.in_fd);
        tidy_termstate.cbreak();

        return @Window{
            c_window: ptr::null(),  // TODO obviously

            term: self,
            row: 0,
            col: 0,
            width: self.width(),
            height: self.height(),

            tidyables: ~[@tidy_termstate as @Drop, @tidy_kx as @Drop, @tidy_cup as @Drop],
        };
    }
}


struct Window {
    c_window: *c::WINDOW,
    term: @Terminal,

    row: uint,
    col: uint,
    // TODO: what happens to width and height on resize?  default is obviously
    // "nothing", but seems it would be nice to support default behavior like
    // preserving bottom/right margins or scaling proportionally
    width: uint,
    height: uint,

    // TODO still not super sure about whether this is a good idea; with an
    // @-pointer, it's never quite clear when the window will go away...  but
    // for now I don't have a better idea for dealing with fullscreen outside a
    // `do` reliably.  ask #rust?
    // TODO actually I'd like to just have a set of termcaps that need to be
    // reversed when the /terminal/ goes away, and add/remove those as
    // appropriate.  or maybe just a list of flags would be faster.  good idea?
    // how does that affect termstate?  i mean, windows that are fullscreen
    // also want to reverse termstate (since they activate raw) as well as
    // termcaps (which do fullscreen), so.  idk i don't like using scope guards
    // everywhere for all this.
    priv tidyables: ~[@Drop],
}

impl Window {
    ////// Properties

    /** Returns the size of the window as (rows, columns). */
    pub fn size(&self) -> (uint, uint) {
        return (self.term.height(), self.term.width());
    }

    /** Returns the current cursor position as (row, column). */
    pub fn position(&self) -> (uint, uint) {
        unsafe {
            return (
                c::getcury(self.c_window) as uint,
                c::getcurx(self.c_window) as uint);
        }
    }

    ////// Structural methods

    // TODO how does the parent/child relationship work here?  does the child
    // know the parent?  vice versa?  what happens when you destroy one?  how
    // does this relate to fullscreen?
    pub fn create_window(&self, height: uint, width: uint, row: uint, col: uint) -> @Window {
        let actual_height = if height == 0 { self.height } else { height };
        let actual_width = if width == 0 { self.width } else { width };

        return @Window{
            c_window: ptr::null(),  // TODO obviously

            term: self.term,
            row: row,
            col: col,
            width: actual_width,
            height: actual_height,

            tidyables: ~[],
        };
    }


    ////// Drawing methods

    pub fn mv(&self, row: uint, col: uint) {
        // TODO write to a buffer, only paint on repaint()
        self.term.write_cap2("cup", (self.row + row) as int, (self.col + col) as int);
        // TODO return value
        //c::wmove(self.c_window, row as c_int, col as c_int);
    }

    pub fn write(&self, msg: &str) {
        // TODO write to a buffer, only paint on repaint()
        self.term.write(msg);
    }

    pub fn attrwrite(&self, s: &str, style: &Style) {
        // TODO same as above
        self.term.attrwrite(s, style);
    }

    pub fn repaint(&self) {
        // TODO return value
        //c::wrefresh(self.c_window);

        // TODO implement me

        io::stdout().flush();
    }

    pub fn clear(&self) {
        // TODO only touch in-memory screen and update on repaint
        // TODO this only works for the root window with no child windows; cute
        // optimization but not reliable in general.  also, moves the cursor,
        // which may not be desired.
        // TODO should this be done on fullscreen by default, or is it anyway?

        // TODO should this be a method on the terminal, since it's just a cap?

        self.term.write_cap("clear");
    }

    // Input

    pub fn getch(&self) -> char {
        // TODO this name sucks
        let ch: c::wint_t = 0;
        let res = unsafe { c::wget_wch(self.c_window, ptr::addr_of(&ch)) };
        if res == c::OK {
            return ch as char;
        }
        else if res == c::KEY_CODE_YES {
            // TODO this is super wrong; the keycodes overlap with legit
            // characters
            return ch as char;
        }
        else if res == c::ERR {
            fail!(~"ERR");
        }
        else {
            // TODO wat
            fail!(~"wat");
        }
        // TODO what if you get WEOF...?
    }

    pub fn read_key(&self) -> Key {
        // Thanks to urwid for already doing much of this work in a readable
        // manner!
        // TODO this doesn't time out, doesn't check for key sequences, etc
        // etc.  it's hilariously sad.
        // TODO should have a timeout after Esc...  et al.?
        // TODO this could probably stand to be broken out a bit
        let raw_byte = self.term.in_file.read_byte();
        if raw_byte < 0 {
            // TODO how can this actually happen?
            fail!(~"couldn't read a byte?!");
        }

        let mut byte = raw_byte as u8;
        let mut bytes = ~[byte];

        if 32 <= byte && byte <= 126 {
            // ASCII character
            return Character(byte as char);
        }

        // XXX urwid here checks for some specific keys: tab, enter, backspace

        // XXX is this cross-terminal?
        if 0 < byte && byte < 27 {
            // Ctrl-x
            // TODO no nice way to return this though, so just do the character
            return Character(byte as char);
        }
        if 27 < byte && byte < 32 {
            // Ctrl-X
            // TODO no nice way to return this though, so just do the character
            return Character(byte as char);
        }

        // TODO supporting other encodings would be...  nice...  but hard.
        // what does curses do here; is this where it uses the locale?
        let encoding = "utf8";
        if encoding == "utf8" && byte > 127 && byte < 256 {
            let need_more;
            if byte & 0xe0 == 0xc0 {
                // two-byte form
                need_more = 1;
            }
            else if byte & 0xf0 == 0xe0 {
                // three-byte form
                need_more = 2;
            }
            else if byte & 0xf8 == 0xf0 {
                // four-byte form
                need_more = 3;
            }
            else {
                fail!(fmt!("junk byte %?", byte));
            }

            bytes += self.term.in_file.read_bytes(need_more);
            // TODO umm this only works for utf8
            let decoded = str::from_bytes(bytes);
            if decoded.len() != 1 {
                fail!(~"unexpected decoded string length!");
            }

            return Character(decoded.char_at(0));
        }

        // XXX urwid has if byte > 127 && byte < 256...  but that's covered
        // above because we are always utf8.

        
        // OK, check for cute terminal escapes
        loop {
            let (maybe_key, remaining_bytes) = self.term.keypress_trie.find_prefix(bytes);
            match maybe_key {
                option::Some(key) => {
                    return key;
                }
                option::None => (),
            }
            // XXX right now we are discarding any leftover bytes -- but we also only read one at a time so there are never leftovers
            // bytes = remaining_bytes;
            // XXX i don't know a better way to decide when to give up reading a sequence?  i guess it should be time-based
            if bytes.len() > 8 {
                break;
            }
            // XXX again, why does this return an int?  can it be negative on error?
            bytes.push(self.term.in_file.read_byte() as u8);
        }


        return Character(byte as char);
    }

    /** Blocks until a key is pressed.
     *
     * This is identical to `read_key()`, except it returns nothing and reads
     * a little better if you don't care which key was pressed.
     */
    pub fn pause(&self) {
        self.read_key();
    }


    pub fn readln(&self) -> ~str {
        // TODO what should maximum buffer length be?
        // TODO or perhaps i should reimplement the getnstr function myself with getch.
        static buflen: uint = 80;
        unsafe {
            let buf = libc::malloc(buflen * sys::size_of::<c::wint_t>() as size_t)
                as *c::wint_t;
            let res = c::wgetn_wstr(self.c_window, buf, buflen as c_int);

            if res != c::OK {
                fail!(~"not ok");
            }

            let vec = do vec::from_buf(buf, buflen).map |ch| { *ch as char };
            libc::free(buf as *c_void);

            return str::from_chars(vec);
        }
    }

    // Attributes

    pub fn restyle(&self, num_chars: int, attrflags: int, color_index: int) {
        unsafe {
            // NOTE: chgat() returns a c_int, but documentation indicates the
            // value is meaningless.
            c::chgat(num_chars as c_int, attrflags as c::attr_t, color_index as c_short, ptr::null());
        }
    }


    // Drawing

    pub fn set_box(&self, vert: char, horiz: char) {
        unsafe {
            // TODO return value
            c::box_set(self.c_window, ptr::addr_of(&__char_to_cchar_t(vert)), ptr::addr_of(&__char_to_cchar_t(horiz)));
        }
    }

    pub fn set_border(&self, l: char, r: char, t: char, b: char, tl: char, tr: char, bl: char, br: char) {
        unsafe {
            // TODO return value
            c::wborder_set(self.c_window,
                ptr::addr_of(&__char_to_cchar_t(l)),
                ptr::addr_of(&__char_to_cchar_t(r)),
                ptr::addr_of(&__char_to_cchar_t(t)),
                ptr::addr_of(&__char_to_cchar_t(b)),
                ptr::addr_of(&__char_to_cchar_t(tl)),
                ptr::addr_of(&__char_to_cchar_t(tr)),
                ptr::addr_of(&__char_to_cchar_t(bl)),
                ptr::addr_of(&__char_to_cchar_t(br))
            );
        }
    }


    // Misc

    // TODO flesh this out.  apparently '2' for 'very visible' is also
    // supported?
    fn hide_cursor(&self) {
        unsafe {
            // TODO return value
            // TODO this belongs to the terminal, not a window
            c::curs_set(0);
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Attributes

pub struct Style {
    // TODO i guess these could be compacted into a bitstring, but eh.
    is_bold: bool,
    is_underline: bool,

    // TODO strictly speaking these should refer to entire colors, not just
    // color numbers, for compatability with a truckload of other kinds of
    // terminals.  but, you know.
    // TODO -1 for the default is super hokey and only for curses compat
    fg_color: int,
    bg_color: int,
}
pub fn Style() -> Style {
    return NORMAL;
}
impl Style {
    pub fn bold(&self) -> ~Style {
        return ~Style{ is_bold: true, ..*self };
    }

    pub fn underline(&self) -> ~Style {
        return ~Style{ is_underline: true, ..*self };
    }

    // TODO this pretty much blows; color pairs are super archaic and i am
    // trying to hack around them until i just give up and bail on the curses
    // dependency.  works on my machine...
    // TODO this only works for the first 16 colors anyway
    // TODO calling start_color() resets all color pairs, so you can't use this
    // before capturing the window...  :|
    // TODO this doesn't handle default colors correctly, because those are
    // color index -1.
    pub fn fg(&self, color: int) -> ~Style {
        return ~Style{ fg_color: color, ..*self };
    }
    pub fn bg(&self, color: int) -> ~Style {
        return ~Style{ bg_color: color, ..*self };
    }

    fn c_value(&self) -> c_int {
        let mut rv: c_int = 0;

        if self.is_bold {
            rv |= c::A_BOLD;
        }
        if self.is_underline {
            rv |= c::A_UNDERLINE;
        }
        
        // Calculate a pair number to consume.  It's a signed short, so use the
        // lower 8 bits for fg and upper 7 for bg
        // TODO without ext_colors, this gets all fucked up if the pair number
        // is anything over 255
        let fg = match self.fg_color {
            -1 => 14,
            _ => self.fg_color,
        };
        let bg = match self.bg_color {
            -1 => 14,
            _ => self.bg_color,
        };
        //let pair = ((bg << 4) | fg) as c_short;
        let pair = fg as c_short;
        unsafe {
            c::init_pair(pair, self.fg_color as c_short, self.bg_color as c_short);
            rv |= c::COLOR_PAIR(pair as c_int);
        }

        return rv;
    }
}

pub static NORMAL: Style = Style{ is_bold: false, is_underline: false, fg_color: -1, bg_color: -1 };


////////////////////////////////////////////////////////////////////////////////
// Key handling

// TODO: i don't know how to handle ctrl-/alt- sequences.
// 1. i don't know how to represent them type-wise
// 2. i don't know how to parse them!  they aren't in termcap.
enum SpecialKey {
    KEY_LEFT,
    KEY_RIGHT,
    KEY_UP,
    KEY_DOWN,
    KEY_ESC,

    // XXX temp kludge until i have all yonder keys
    KEY_UNKNOWN,
}

pub enum Key {
    Character(char),
    SpecialKey(SpecialKey),
    FunctionKey(uint),
}

fn cap_to_key(cap: ~str) -> Key {
    // TODO this matching would be much more efficient if it used, hurr, a
    // trie.  but seems silly to build one only to use it a few times.
    // TODO uh maybe this should use the happy C names
    return match cap {
        ~"kcuf1" => SpecialKey(KEY_RIGHT),
        ~"kcub1" => SpecialKey(KEY_LEFT),
        ~"kcuu1" => SpecialKey(KEY_UP),
        ~"kcud1" => SpecialKey(KEY_DOWN),
        ~"kf1" => FunctionKey(1),
        ~"kf2" => FunctionKey(2),
        ~"kf3" => FunctionKey(3),
        ~"kf4" => FunctionKey(4),
        ~"kf5" => FunctionKey(5),
        ~"kf6" => FunctionKey(6),
        ~"kf7" => FunctionKey(7),
        ~"kf8" => FunctionKey(8),
        ~"kf9" => FunctionKey(9),
        ~"kf10" => FunctionKey(10),
        ~"kf11" => FunctionKey(11),
        ~"kf12" => FunctionKey(12),
        _ => SpecialKey(KEY_UNKNOWN),
    };
}

////////////////////////////////////////////////////////////////////////////////
// Misc that should probably go away

/** Returns the screen size as (rows, columns). */
pub fn screen_size() -> (uint, uint) {
    return (c::LINES as uint, c::COLS as uint);
}

// Attribute definition

pub fn define_color_pair(color_index: int, fg: c_short, bg: c_short) {
    // TODO return value
    unsafe {
        c::init_pair(color_index as c_short, fg, bg);
    }
}




fn __char_to_cchar_t(ch: char) -> c::cchar_t {
    return c::cchar_t{
        attr: c::A_NORMAL as c::attr_t,
        chars: [ch as wchar_t, 0, 0, 0, 0],
    };
}
