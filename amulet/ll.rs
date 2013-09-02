/** Low-level ncurses wrapper, for simple or heavily customized applications. */

use std::libc::{c_char,c_int,c_long,c_schar,c_short,c_void,size_t,wchar_t};
use std::io::ReaderUtil;
use std::ptr;
use std::str;
use std::libc;
use std::io;
use std::cast;
use std::vec::raw;
use std::vec;
use std::sys;

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
struct TidyTermcap {
    terminfo: @TerminalInfo,
    cap: &'static str,
}
#[unsafe_destructor]
impl Drop for TidyTermcap {
    fn drop(&self) {
        self.terminfo.write_cap(self.cap);
    }
}

/** Wraps a couple other droppables.  Exists because Rust currently doesn't
 * handle borrowed pointers to traits very well; see
 * https://github.com/mozilla/rust/issues/5708
 */
pub struct TidyBundle {
    tidy_termcaps: ~[TidyTermcap],
    tidy_termstates: ~[termios::TidyTerminalState],
}


struct TerminalInfo {
    in_fd: c_int,
    in_file: @io::Reader,
    out_fd: c_int,
    out_file: @io::Writer,

    keypress_trie: @mut Trie<u8, Key>,

    priv c_terminfo: *c::TERMINAL,
    priv tidy_termstate: termios::TidyTerminalState,

    //term_type: ~str,
}

#[unsafe_destructor]
impl Drop for TerminalInfo {
    fn drop(&self) {
        self.tidy_termstate.restore();
    }
}
impl TerminalInfo {
    #[fixed_stack_segment]
    pub fn new() -> @TerminalInfo {
        let error_code: c_int = 0;
        // NULL first arg means read TERM from env (TODO).
        // second arg is a fd to spew to on error, but it's not used when there's
        // an error pointer.
        // third arg is a var to stick the error code in.
        // TODO allegedly setupterm doesn't work on BSD?
        unsafe {
            let res = c::setupterm(ptr::null(), -1, ptr::to_unsafe_ptr(&error_code));

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

        return @TerminalInfo{
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
    }


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

    #[fixed_stack_segment]
    fn flag_cap(&self, name: &str) -> bool {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = 0;
            do name.to_c_str().with_ref |bytes| {
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

    #[fixed_stack_segment]
    fn numeric_cap(&self, name: &str) -> uint {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = -1;
            do name.to_c_str().with_ref |bytes| {
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

    #[fixed_stack_segment]
    fn _string_cap_cstr(&self, name: &str) -> *c_char {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = ptr::null();
            do name.to_c_str().with_ref |bytes| {
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
    #[fixed_stack_segment]
    fn format_cap(&self, name: &str, args: ~[int]) -> ~str {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let template = self._string_cap_cstr(name);
            let padded_args = args + ~[0, 0, 0, 0, 0, 0, 0, 0];
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

    #[fixed_stack_segment]
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
    pub fn write_cap(&self, cap_name: &str) {
        // If we're calling this function then this capability really shouldn't
        // take any arguments, but someone might have screwed up, or it may
        // have an escaped % or something.  Best do the whole formatting thing.
        self._write_capx(cap_name, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    }
    pub fn write_cap1(&self, cap_name: &str, arg1: int) {
        self._write_capx(cap_name, arg1 as c_long, 0, 0, 0, 0, 0, 0, 0, 0);
    }
    pub fn write_cap2(&self, cap_name: &str, arg1: int, arg2: int) {
        self._write_capx(cap_name, arg1 as c_long, arg2 as c_long, 0, 0, 0, 0, 0, 0, 0);
    }

    pub fn write_tidy_cap(@self, do_cap: &str, undo_cap: &'static str) -> TidyTermcap {
        self.write_cap(do_cap);

        return TidyTermcap{ terminfo: self, cap: undo_cap };
    }

    // TODO should capabilities just have a method apiece, like blessings?

    // Output

    pub fn write(&self, s: &str) {
        self.out_file.flush();
        // TODO well.  should be a bit more flexible, i guess.
        self.out_file.write_str(s);
        self.out_file.flush();
    }



    // Some stuff
    pub fn move(&self, x: uint, y: uint) {
        // TODO check for existence of cup
        self.write_cap2("cup", y as int, x as int);
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
    pub fn bold(&self) -> Style {
        return Style{ is_bold: true, ..*self };
    }

    pub fn underline(&self) -> Style {
        return Style{ is_underline: true, ..*self };
    }

    // TODO this pretty much blows; color pairs are super archaic and i am
    // trying to hack around them until i just give up and bail on the curses
    // dependency.  works on my machine...
    // TODO this only works for the first 16 colors anyway
    // TODO calling start_color() resets all color pairs, so you can't use this
    // before capturing the window...  :|
    // TODO this doesn't handle default colors correctly, because those are
    // color index -1.
    pub fn fg(&self, color: int) -> Style {
        return Style{ fg_color: color, ..*self };
    }
    pub fn bg(&self, color: int) -> Style {
        return Style{ bg_color: color, ..*self };
    }

    #[fixed_stack_segment]
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
#[deriving(Clone)]
enum SpecialKeyCode {
    KEY_LEFT,
    KEY_RIGHT,
    KEY_UP,
    KEY_DOWN,
    KEY_ESC,

    // XXX temp kludge until i have all yonder keys
    KEY_UNKNOWN,
}

#[deriving(Clone)]
pub enum Key {
    Character(char),
    SpecialKey(SpecialKeyCode),
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
