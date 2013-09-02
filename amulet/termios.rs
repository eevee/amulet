use std::clone::Clone;
use std::libc::c_int;
use std::ptr;

// -----------------------------------------------------------------------------
// Platform-specific implementations

// So, termios, this cool POSIX standard, relies on (a) a struct definition
// that lives in a C header and varies by platform and (b) a bunch of #define'd
// constants that live in a C header file and vary by platform.
// This is what I've got on my platform.  Feel free to add yours!
// Hopefully someday rustc will be able to parse this stuff out itself.

#[cfg(target_os="linux")]
mod imp {
    use std::libc::{c_int,c_uint,c_ushort,c_void};
    use std::ptr;

    static NCCS: int = 32;
    type cc_t = c_int;
    type tcflag_t = c_uint;
    type speed_t = c_uint;


    // Constants.
    /* c_cc characters */
    pub static VINTR:    c_uint = 0;
    pub static VQUIT:    c_uint = 1;
    pub static VERASE:   c_uint = 2;
    pub static VKILL:    c_uint = 3;
    pub static VEOF:     c_uint = 4;
    pub static VTIME:    c_uint = 5;
    pub static VMIN:     c_uint = 6;
    pub static VSWTC:    c_uint = 7;
    pub static VSTART:   c_uint = 8;
    pub static VSTOP:    c_uint = 9;
    pub static VSUSP:    c_uint = 10;
    pub static VEOL:     c_uint = 11;
    pub static VREPRINT: c_uint = 12;
    pub static VDISCARD: c_uint = 13;
    pub static VWERASE:  c_uint = 14;
    pub static VLNEXT:   c_uint = 15;
    pub static VEOL2:    c_uint = 16;

    /* c_iflag bits */
    pub static IGNBRK:   c_uint = 0x00001;
    pub static BRKINT:   c_uint = 0x00002;
    pub static IGNPAR:   c_uint = 0x00004;
    pub static PARMRK:   c_uint = 0x00008;
    pub static INPCK:    c_uint = 0x00010;
    pub static ISTRIP:   c_uint = 0x00020;
    pub static INLCR:    c_uint = 0x00040;
    pub static IGNCR:    c_uint = 0x00080;
    pub static ICRNL:    c_uint = 0x00100;
    pub static IUCLC:    c_uint = 0x00200;
    pub static IXON:     c_uint = 0x00400;
    pub static IXANY:    c_uint = 0x00800;
    pub static IXOFF:    c_uint = 0x01000;
    pub static IMAXBEL:  c_uint = 0x02000;
    pub static IUTF8:    c_uint = 0x04000;

    /* c_oflag bits */
    pub static OPOST:    c_uint = 0x00001;
    pub static OLCUC:    c_uint = 0x00002;
    pub static ONLCR:    c_uint = 0x00004;
    pub static OCRNL:    c_uint = 0x00008;
    pub static ONOCR:    c_uint = 0x00010;
    pub static ONLRET:   c_uint = 0x00020;
    pub static OFILL:    c_uint = 0x00040;
    pub static OFDEL:    c_uint = 0x00080;

    /* c_cflag bit meaning */
    pub static  B0:      c_uint = 0x00000;  // hang up
    pub static  B50:     c_uint = 0x00001;
    pub static  B75:     c_uint = 0x00002;
    pub static  B110:    c_uint = 0x00003;
    pub static  B134:    c_uint = 0x00004;
    pub static  B150:    c_uint = 0x00005;
    pub static  B200:    c_uint = 0x00006;
    pub static  B300:    c_uint = 0x00007;
    pub static  B600:    c_uint = 0x00008;
    pub static  B1200:   c_uint = 0x00009;
    pub static  B1800:   c_uint = 0x0000a;
    pub static  B2400:   c_uint = 0x0000b;
    pub static  B4800:   c_uint = 0x0000c;
    pub static  B9600:   c_uint = 0x0000d;
    pub static  B19200:  c_uint = 0x0000e;
    pub static  B38400:  c_uint = 0x0000f;
    pub static CSIZE:    c_uint = 0x00030;
    pub static   CS5:    c_uint = 0x00000;
    pub static   CS6:    c_uint = 0x00010;
    pub static   CS7:    c_uint = 0x00020;
    pub static   CS8:    c_uint = 0x00030;
    pub static CSTOPB:   c_uint = 0x00040;
    pub static CREAD:    c_uint = 0x00080;
    pub static PARENB:   c_uint = 0x00100;
    pub static PARODD:   c_uint = 0x00200;
    pub static HUPCL:    c_uint = 0x00400;
    pub static CLOCAL:   c_uint = 0x00800;
    pub static  B57600:  c_uint = 0x01001;
    pub static  B115200: c_uint = 0x01002;
    pub static  B230400: c_uint = 0x01003;
    pub static  B460800: c_uint = 0x01004;
    pub static  B500000: c_uint = 0x01005;
    pub static  B576000: c_uint = 0x01006;
    pub static  B921600: c_uint = 0x01007;
    pub static  B1000000: c_uint = 0x01008;
    pub static  B1152000: c_uint = 0x01009;
    pub static  B1500000: c_uint = 0x0100a;
    pub static  B2000000: c_uint = 0x0100b;
    pub static  B2500000: c_uint = 0x0100c;
    pub static  B3000000: c_uint = 0x0100d;
    pub static  B3500000: c_uint = 0x0100e;
    pub static  B4000000: c_uint = 0x0100f;

    /* c_lflag bits */
    pub static ISIG:     c_uint = 0x00001;
    pub static ICANON:   c_uint = 0x00002;
    pub static ECHO:     c_uint = 0x00008;
    pub static ECHOE:    c_uint = 0x00010;
    pub static ECHOK:    c_uint = 0x00020;
    pub static ECHONL:   c_uint = 0x00040;
    pub static NOFLSH:   c_uint = 0x00080;
    pub static TOSTOP:   c_uint = 0x00100;
    pub static IEXTEN:   c_uint = 0x08000;

    /* tcsetattr uses these */
    pub static TCSANOW:   c_int = 0;
    pub static TCSADRAIN: c_int = 1;
    pub static TCSAFLUSH: c_int = 2;

    /* ioctls */
    pub static TIOCGWINSZ: c_int = 0x5413;


    pub struct termios {
        c_iflag: tcflag_t,      // input modes
        c_oflag: tcflag_t,      // output modes
        c_cflag: tcflag_t,      // control modes
        c_lflag: tcflag_t,      // local modes
        // why is this here?  what is going on?  who knows
        c_line: cc_t,           // "line discipline"
        // NOTE: 32 is the value of NCCS
        c_cc: [cc_t, ..32],      // control characters
        c_ispeed: speed_t,      // input speed
        c_ospeed: speed_t,      // output speed
    }

    // deriving(Clone) doesn't work for fixed-size vectors (#7622)
    impl Clone for termios {
        fn clone(&self) -> termios {
            return termios{
                // ...also it's a syntax error to not have at least one pair
                c_iflag: self.c_iflag,
                ..*self
            };
        }
    }

    // Need this to be able to create blank structs on the stack
    pub fn blank_termios() -> termios {
        return termios{
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0, ..32],
            c_ispeed: 0,
            c_ospeed: 0,
        };
    }



    struct winsize {
        ws_row:     c_ushort,
        ws_col:     c_ushort,
        ws_xpixel:  c_ushort,  // unused
        ws_ypixel:  c_ushort,  // unused
    }

    extern {
        #[link_name = "ioctl"]
        fn ioctl_p(fd: c_int, request: c_int, arg1: *c_void) -> c_int;
    }

    #[fixed_stack_segment]
    pub fn request_terminal_size(fd: c_int) -> (uint, uint) {
        let size = winsize{ ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };

        let res = unsafe { ioctl_p(fd, TIOCGWINSZ, ptr::to_unsafe_ptr(&size) as *c_void) };

        // XXX return value is -1 on failure
        // returns width, height
        return (size.ws_col as uint, size.ws_row as uint);
    }
}

// End of platform-specific implementations.
// -----------------------------------------------------------------------------

extern {
    fn tcgetattr(fd: c_int, termios_p: *imp::termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *imp::termios) -> c_int;
}

/** Self-reverting access to termios state changes.
  *
  * When this object goes out of scope, it will restore the tty to whatever
  * settings it had when this object was created.  It's like RAII, except
  * without the C++ braindamage.
  */
pub struct TidyTerminalState {
    priv c_fd: c_int,
    priv c_termios_orig: imp::termios,
    priv c_termios_cur: @mut imp::termios,
}

#[unsafe_destructor]
impl Drop for TidyTerminalState {
    fn drop(&self) {
        self.restore_term();
    }
}

#[fixed_stack_segment]
pub fn TidyTerminalState(fd: c_int) -> TidyTerminalState {
    let c_termios = imp::blank_termios();

    // TODO this has a retval, but...  eh...
    unsafe {
        tcgetattr(fd as c_int, ptr::to_unsafe_ptr(&c_termios));
    }

    return TidyTerminalState{
        c_fd: fd as c_int,
        c_termios_cur: @mut c_termios.clone(),
        c_termios_orig: c_termios,
    };
}

// TODO: i want this impl only for ~T but that makes the drop not work
impl TidyTerminalState {
    #[fixed_stack_segment]
    fn restore_term (&self) {
        unsafe {
            tcsetattr(self.c_fd, imp::TCSAFLUSH, ptr::to_unsafe_ptr(&self.c_termios_orig));
        }
    }

    /** Explicitly restore the terminal to its pristine state. */
    pub fn restore(&self) {
        self.restore_term();
        *self.c_termios_cur = self.c_termios_orig.clone();
    }


    // --------------------------------------------------------------------------
    // Raw and cbreak.  (There's no "cooked" because that is, presumably, the
    // default, and you can just use .restore().)
    // There are a lot of bits to be twiddled here, and not really any fixed
    // definition of which ones "should" be.  The following is based on a
    // combination of ncurses, Python's tty module, and Linux's termios(3).

    /** Switch an fd to "raw" mode.
      *
      * In raw mode, absolutely every keypress is passed along to the application
      * untouched.  This means, for example, that ^C doesn't send a SIGINT.
      */
    pub fn raw(&self) {
        // Disable SIGINT
        self.c_termios_cur.c_iflag &= !imp::BRKINT;
        // Ignore usual signal-generating keys
        self.c_termios_cur.c_lflag &= !imp::ISIG;

        // Everything else is covered by cbreak
        self.cbreak();
    }

    /** Switch an fd to "cbreak" mode.
      *
      * This is identical to raw mode, except that ^C and other signal keys
      * work as normal instead of going to the application.
      */
    #[fixed_stack_segment]
    pub fn cbreak(&self) {
        self.c_termios_cur.c_iflag &= !(
            imp::IXON       // ignore XON/XOFF, i.e. ^S ^Q
            | imp::ISTRIP   // don't strip the 8th bit (?!)
            | imp::INPCK    // don't check for parity errors
            | imp::ICRNL    // don't convert cr to nl
            | imp::INLCR    // don't convert nl to cr
            | imp::IGNCR    // don't drop cr
            | imp::PARMRK   // don't prepend \377 \0 to error nuls
        );

        self.c_termios_cur.c_oflag &= !(
            // TODO turning these off make \n act as a literal newline with no
            // cursor return -- not what anyone expects from \n.  should i
            // convert \n manually, or disable every other possible output flag
            // here?
            //imp::OPOST      // turn off "impl-specific processing" -- this
            //                // includes, e.g., converting tabs to spaces
            //| imp::ONLCR    // don't convert nl to cr
            0
        );
        // TODO in the meantime, make sure \n works right
        self.c_termios_cur.c_oflag |= imp::OPOST | imp::ONLCR;

        self.c_termios_cur.c_cflag &= !(
            imp::PARENB     // turn off parity generation/checking
        );
        // Set 8 bits per character.
        // NOTE: it's unclear why this is part of "raw" mode and not just
        // something any modern terminal would want, but this is done in every
        // raw impl I've found *except* curses...
        self.c_termios_cur.c_cflag =
            (self.c_termios_cur.c_cflag & !imp::CSIZE) | imp::CS8;

        self.c_termios_cur.c_lflag &= !(
            imp::ICANON     // turn off "canonical" mode -- this is the primary
                            // char-at-a-time flag
            | imp::IEXTEN   // turn off "impl-specific processing"
            | imp::ECHO     // turn off local echo
        );

        unsafe {
            // TCSAFLUSH: make the changes once all output thusfar has been
            // sent, and clear the input buffer
            // TODO this returns something, but even success is hokey, so what
            // is there to do about it
            // TODO do i want this in a separate 'commit()' method?  for
            // chaining etc?
            tcsetattr(self.c_fd, imp::TCSAFLUSH, ptr::to_unsafe_ptr(&*self.c_termios_cur));
        }
    }
}


/*
    For reference, as the interpretation of "raw" differs greatly.  This will
    go away once I'm confident about the correctness of everything above.

    p is Python; c is ncurses; i is ncurses's initscr(); L is the Linux
    ttyraw() function; ✓ is this module.

    RAW:
    i - BRKINT      p c   L ✓
    i - ICRNL       p   i L ✓
    i - INLCR           i L ✓
    i - IGNCR           i L ✓
    i - INPCK       p       ✓
    i - ISTRIP      p     L ✓
    i - IXON        p c   L ✓
    i - PARMRK        c   L ✓
    i - IGNBRK            L
    o - OPOST       p     L ✓
    o - ONLCR           i   ✓
    c - CSIZE       p     L ✓
    c - PARENB      p     L ✓
    c + CS8         p     L ✓
    l - ECHO        p   i L ✓
    l - ECHONL          i L   (has no effect without ICANON)
    l - ICANON      p c   L ✓
    l - IEXTEN      p c   L ✓
    l - ISIG        p c   L ✓

    CBREAK:
    i - ICRNL         c
    l - ECHO        p
    l - ICANON      p c
    l + ISIG          c


    NORAW:
    i + IXON
    i + BRKINT
    i + PARMRK
    l + ISIG
    l + ICANON
    l . IEXTEN
    l - everything else

    NOCBREAK:
    i + ICRNL
    l + ICANON
*/
