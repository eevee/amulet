/** Low-level ncurses wrapper, for simple or heavily customized applications. */

use libc::{c_char,c_int,c_long,c_short};
use std::ffi::CString;
use std::ffi::c_str_to_bytes;
use std::ptr;
use std::str;
use std::str::from_c_str;
use libc;
use std::io;
use std::mem::transmute;
use std::vec;
use std::rc::Rc;
use std::cell::RefCell;

use c;
use termios;
use trie::Trie;

extern {
    fn setlocale(category: c_int, locale: *mut c_char) -> *mut c_char;

    // XXX why the fuck is this not available
    static stdout: *mut libc::FILE;
}


/** Prints a given termcap sequence when it goes out of scope. */
pub struct TidyTermcap<'a> {
    terminfo: &'a TerminalInfo<'a>,
    cap: &'static str,
}
#[unsafe_destructor]
impl<'a> Drop for TidyTermcap<'a> {
    fn drop(&mut self) {
        self.terminfo.write_cap(self.cap);
    }
}


pub struct TerminalInfo<'a> {
    pub in_fd: c_int,
    pub in_file: RefCell<Box<io::Reader + 'a>>,
    pub out_fd: c_int,
    out_file: RefCell<Box<io::Writer + 'a>>,

    pub keypress_trie: Trie<u8, Key>,

    c_terminfo: *mut c::TERMINAL,
    tidy_termstate: termios::TidyTerminalState,

    //term_type: &str,
}

#[unsafe_destructor]
impl<'a> Drop for TerminalInfo<'a> {
    fn drop(&mut self) {
        self.tidy_termstate.restore();
    }
}
impl<'a> TerminalInfo<'a> {
    #[fixed_stack_segment]
    pub fn new() -> TerminalInfo<'a> {
        let mut error_code: c_int = 0;
        // NULL first arg means read TERM from env (TODO).
        // second arg is a fd to spew to on error, but it's not used when there's
        // an error pointer.
        // third arg is a var to stick the error code in.
        // TODO allegedly setupterm doesn't work on BSD?
        unsafe {
            let res = c::setupterm(ptr::null(), -1, &mut error_code);

            if res != c::OK {
                if error_code == -1 {
                    panic!("Couldn't find terminfo database");
                }
                else if error_code == 0 {
                    panic!("Couldn't identify terminal");
                }
                else if error_code == 1 {
                    // The manual puts this as "terminal is hard-copy" but come on.
                    panic!("Terminal appears to be made of paper");
                }
                else {
                    panic!("Something is totally fucked");
                }
            }
        }

        // Okay; now terminfo is sitting in a magical global somewhere.  Snag a
        // pointer to it.
        let terminfo = c::cur_term;

        let mut keypress_trie = Trie::new();
        let mut p: *const *const c_char = &c::strnames[0];
        unsafe {
            while *p != ptr::null() {
                let capname = from_c_str(*p);

                if capname.char_at(0) == 'k' {
                    let cap = c::tigetstr(*p);
                    if cap != ptr::null() {
                        let cap_key = c_str_to_bytes(&cap);
                        keypress_trie.insert(cap_key, cap_to_key(capname));
                    }
                }

                p = p.offset(1);
            }
        }

        return TerminalInfo{
            // TODO would be nice to parametrize these, but Reader and Writer do
            // not yet expose a way to get the underlying fd, which makes the API
            // sucky
            in_fd: 0,
            in_file: RefCell::new(Box::new(io::stdin()) as Box<io::Reader>),
            out_fd: 1,
            out_file: RefCell::new(Box::new(io::stdout()) as Box<io::Writer>),

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
    pub fn height(&self) -> usize {
        // TODO rather not dip into `imp`, but `pub use` isn't working right
        let (_, height) = termios::imp::request_terminal_size(self.out_fd);
        return height;
        //return self.numeric_cap("lines");
    }
    pub fn width(&self) -> usize {
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
            let c_name = CString::from_slice(name.as_bytes());
            value = c::tigetflag(c_name.as_ptr());

            if value == -1 {
                // wrong type
                panic!("wrong type");
            }

            // Otherwise, is 0 or 1
            return value != 0;
        }
    }

    #[fixed_stack_segment]
    fn numeric_cap(&self, name: &str) -> u32 {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let mut value = -1;
            let c_name = CString::from_slice(name.as_bytes());
            value = c::tigetnum(c_name.as_ptr());

            if value == -2 {
                // wrong type
                panic!("wrong type");
            }
            else if value == -1 {
                // missing; should be None
                panic!("missing; should be None");
            }

            return value as u32;
        }
    }

    #[fixed_stack_segment]
    fn _string_cap_cstr(&self, name: &str) -> *const c_char {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let c_name = CString::from_slice(name.as_bytes());
            let value = c::tigetstr(c_name.as_ptr());

            if value == ptr::null() {
                // missing; should be None really
                panic!("missing; should be None really");
            }
            else if transmute::<_, isize>(value) == -1 {
                // wrong type
                panic!("wrong type");
            }

            return value;
        }
    }

    fn string_cap(&self, name: &str) -> &str {
        let value = self._string_cap_cstr(name);

        unsafe {
            return from_c_str(value);
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
    fn format_cap(&self, name: &str, args: Vec<isize>) -> &str {
        unsafe {
            c::set_curterm(self.c_terminfo);

            let template = self._string_cap_cstr(name);
            let padded_args = args + &[0, 0, 0, 0, 0, 0, 0, 0];
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

            return from_c_str(formatted);
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

            //unsafe { io::stderr().write_str(fmt!("%s\t%s\n", name, from_c_str(formatted))); }

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
    pub fn write_cap1(&self, cap_name: &str, arg1: isize) {
        self._write_capx(cap_name, arg1 as c_long, 0, 0, 0, 0, 0, 0, 0, 0);
    }
    pub fn write_cap2(&self, cap_name: &str, arg1: isize, arg2: isize) {
        self._write_capx(cap_name, arg1 as c_long, arg2 as c_long, 0, 0, 0, 0, 0, 0, 0);
    }

    pub fn write_tidy_cap(&'a self, do_cap: &str, undo_cap: &'static str) -> TidyTermcap<'a> {
        self.write_cap(do_cap);

        return TidyTermcap{ terminfo: self, cap: undo_cap };
    }

    // TODO should capabilities just have a method apiece, like blessings?

    // Output

    pub fn write(&self, s: &str) {
        let mut out_file = self.out_file.borrow_mut();
        out_file.flush();
        // TODO well.  should be a bit more flexible, i guess.
        out_file.write_str(s);
        out_file.flush();
    }



    // Some stuff
    pub fn reposition(&self, x: usize, y: usize) {
        // TODO check for existence of cup
        self.write_cap2("cup", y as isize, x as isize);
    }
}


////////////////////////////////////////////////////////////////////////////////
// Attributes

#[derive(Clone)]
pub struct Style {
    // TODO i guess these could be compacted into a bitstring, but eh.
    pub is_bold: bool,
    pub is_underline: bool,

    // TODO strictly speaking these should refer to entire colors, not just
    // color numbers, for compatability with a truckload of other kinds of
    // terminals.  but, you know.
    // TODO -1 for the default is super hokey and only for curses compat
    pub fg_color: isize,
    pub bg_color: isize,
}
pub fn Style() -> Style {
    return Style{ ..NORMAL };
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
    pub fn fg(&self, color: isize) -> Style {
        return Style{ fg_color: color, ..*self };
    }
    pub fn bg(&self, color: isize) -> Style {
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
#[derive(Clone, Show)]
pub enum SpecialKeyCode {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    ESC,

    // XXX temp kludge until i have all yonder keys
    UNKNOWN,
}

#[derive(Clone, Show)]
pub enum Key {
    Character(char),
    SpecialKey(SpecialKeyCode),
    FunctionKey(u32),
}

fn cap_to_key(cap: &str) -> Key {
    // TODO this matching would be much more efficient if it used, hurr, a
    // trie.  but seems silly to build one only to use it a few times.
    // TODO uh maybe this should use the happy C names
    return match cap {
        "kcuf1" => Key::SpecialKey(SpecialKeyCode::RIGHT),
        "kcub1" => Key::SpecialKey(SpecialKeyCode::LEFT),
        "kcuu1" => Key::SpecialKey(SpecialKeyCode::UP),
        "kcud1" => Key::SpecialKey(SpecialKeyCode::DOWN),
        "kf1" => Key::FunctionKey(1),
        "kf2" => Key::FunctionKey(2),
        "kf3" => Key::FunctionKey(3),
        "kf4" => Key::FunctionKey(4),
        "kf5" => Key::FunctionKey(5),
        "kf6" => Key::FunctionKey(6),
        "kf7" => Key::FunctionKey(7),
        "kf8" => Key::FunctionKey(8),
        "kf9" => Key::FunctionKey(9),
        "kf10" => Key::FunctionKey(10),
        "kf11" => Key::FunctionKey(11),
        "kf12" => Key::FunctionKey(12),
        _ => Key::SpecialKey(SpecialKeyCode::UNKNOWN),
    };
}
