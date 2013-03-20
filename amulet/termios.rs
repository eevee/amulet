// -----------------------------------------------------------------------------
// Platform-specific implementations

// So, termios, this cool POSIX standard, relies on (a) a struct definition
// that lives in a C header and varies by platform and (b) a bunch of #define'd
// constants that live in a C header file and vary by platform.
// This is what I've got on my platform.  Feel free to add yours!
// Hopefully someday rustc will be able to parse this stuff out itself.

#[cfg(target_os="linux")]
mod imp {
    use core::libc::{c_int,c_uint,c_ushort,c_void};

    const NCCS: int = 32;
    type cc_t = c_int;
    type tcflag_t = c_uint;
    type speed_t = c_uint;


    // Constants.
    /* c_cc characters */
    pub const VINTR:    c_uint = 0;
    pub const VQUIT:    c_uint = 1;
    pub const VERASE:   c_uint = 2;
    pub const VKILL:    c_uint = 3;
    pub const VEOF:     c_uint = 4;
    pub const VTIME:    c_uint = 5;
    pub const VMIN:     c_uint = 6;
    pub const VSWTC:    c_uint = 7;
    pub const VSTART:   c_uint = 8;
    pub const VSTOP:    c_uint = 9;
    pub const VSUSP:    c_uint = 10;
    pub const VEOL:     c_uint = 11;
    pub const VREPRINT: c_uint = 12;
    pub const VDISCARD: c_uint = 13;
    pub const VWERASE:  c_uint = 14;
    pub const VLNEXT:   c_uint = 15;
    pub const VEOL2:    c_uint = 16;

    /* c_iflag bits */
    pub const IGNBRK:   c_uint = 0x00001;
    pub const BRKINT:   c_uint = 0x00002;
    pub const IGNPAR:   c_uint = 0x00004;
    pub const PARMRK:   c_uint = 0x00008;
    pub const INPCK:    c_uint = 0x00010;
    pub const ISTRIP:   c_uint = 0x00020;
    pub const INLCR:    c_uint = 0x00040;
    pub const IGNCR:    c_uint = 0x00080;
    pub const ICRNL:    c_uint = 0x00100;
    pub const IUCLC:    c_uint = 0x00200;
    pub const IXON:     c_uint = 0x00400;
    pub const IXANY:    c_uint = 0x00800;
    pub const IXOFF:    c_uint = 0x01000;
    pub const IMAXBEL:  c_uint = 0x02000;
    pub const IUTF8:    c_uint = 0x04000;

    /* c_oflag bits */
    pub const OPOST:    c_uint = 0x00001;
    pub const OLCUC:    c_uint = 0x00002;
    pub const ONLCR:    c_uint = 0x00004;
    pub const OCRNL:    c_uint = 0x00008;
    pub const ONOCR:    c_uint = 0x00010;
    pub const ONLRET:   c_uint = 0x00020;
    pub const OFILL:    c_uint = 0x00040;
    pub const OFDEL:    c_uint = 0x00080;

    /* c_cflag bit meaning */
    pub const  B0:      c_uint = 0x00000;  // hang up
    pub const  B50:     c_uint = 0x00001;
    pub const  B75:     c_uint = 0x00002;
    pub const  B110:    c_uint = 0x00003;
    pub const  B134:    c_uint = 0x00004;
    pub const  B150:    c_uint = 0x00005;
    pub const  B200:    c_uint = 0x00006;
    pub const  B300:    c_uint = 0x00007;
    pub const  B600:    c_uint = 0x00008;
    pub const  B1200:   c_uint = 0x00009;
    pub const  B1800:   c_uint = 0x0000a;
    pub const  B2400:   c_uint = 0x0000b;
    pub const  B4800:   c_uint = 0x0000c;
    pub const  B9600:   c_uint = 0x0000d;
    pub const  B19200:  c_uint = 0x0000e;
    pub const  B38400:  c_uint = 0x0000f;
    pub const CSIZE:    c_uint = 0x00030;
    pub const   CS5:    c_uint = 0x00000;
    pub const   CS6:    c_uint = 0x00010;
    pub const   CS7:    c_uint = 0x00020;
    pub const   CS8:    c_uint = 0x00030;
    pub const CSTOPB:   c_uint = 0x00040;
    pub const CREAD:    c_uint = 0x00080;
    pub const PARENB:   c_uint = 0x00100;
    pub const PARODD:   c_uint = 0x00200;
    pub const HUPCL:    c_uint = 0x00400;
    pub const CLOCAL:   c_uint = 0x00800;
    pub const  B57600:  c_uint = 0x01001;
    pub const  B115200: c_uint = 0x01002;
    pub const  B230400: c_uint = 0x01003;
    pub const  B460800: c_uint = 0x01004;
    pub const  B500000: c_uint = 0x01005;
    pub const  B576000: c_uint = 0x01006;
    pub const  B921600: c_uint = 0x01007;
    pub const  B1000000: c_uint = 0x01008;
    pub const  B1152000: c_uint = 0x01009;
    pub const  B1500000: c_uint = 0x0100a;
    pub const  B2000000: c_uint = 0x0100b;
    pub const  B2500000: c_uint = 0x0100c;
    pub const  B3000000: c_uint = 0x0100d;
    pub const  B3500000: c_uint = 0x0100e;
    pub const  B4000000: c_uint = 0x0100f;

    /* c_lflag bits */
    pub const ISIG:     c_uint = 0x00001;
    pub const ICANON:   c_uint = 0x00002;
    pub const ECHO:     c_uint = 0x00008;
    pub const ECHOE:    c_uint = 0x00010;
    pub const ECHOK:    c_uint = 0x00020;
    pub const ECHONL:   c_uint = 0x00040;
    pub const NOFLSH:   c_uint = 0x00080;
    pub const TOSTOP:   c_uint = 0x00100;
    pub const IEXTEN:   c_uint = 0x08000;

    /* tcsetattr uses these */
    pub const TCSANOW:   c_int = 0;
    pub const TCSADRAIN: c_int = 1;
    pub const TCSAFLUSH: c_int = 2;

    /* ioctls */
    pub const TIOCGWINSZ: c_int = 0x5413;


    pub struct termios {
        c_iflag: tcflag_t,      // input modes
        c_oflag: tcflag_t,      // output modes
        c_cflag: tcflag_t,      // control modes
        c_lflag: tcflag_t,      // local modes
        // why is this here?  what is going on?  who knows
        c_line: cc_t,           // "line discipline"
        // NOTE: 32 is the value of NCCS
        c_cc: [cc_t * 32],      // control characters
        c_ispeed: speed_t,      // input speed
        c_ospeed: speed_t,      // output speed
    }

    // Need this to be able to create blank structs on the stack
    pub const BLANK_TERMIOS: termios = termios{
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        c_ispeed: 0,
        c_ospeed: 0,
    };



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

    pub fn request_terminal_size(fd: c_int) -> (uint, uint) {
        let size = winsize{ ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };

        let res = unsafe { ioctl_p(fd, TIOCGWINSZ, ptr::addr_of(&size) as *c_void) };

        // XXX return value is -1 on failure
        // returns width, height
        return (size.ws_col as uint, size.ws_row as uint);
    }
}

use core::libc::{c_int,c_uint,c_ushort,c_void};

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
    priv c_termios_cur: imp::termios,
}

impl Drop for TidyTerminalState {
    fn finalize (&self) {
        self.restore_term();
    }
}

pub fn TidyTerminalState(fd: c_int) -> ~TidyTerminalState {
    let c_termios = copy imp::BLANK_TERMIOS;

    // TODO this has a retval, but...  eh...
    unsafe {
        tcgetattr(fd as c_int, ptr::addr_of(&c_termios));
    }

    return ~TidyTerminalState{
        c_fd: fd as c_int,
        c_termios_cur: copy c_termios,
        c_termios_orig: c_termios,
    };
}

// TODO: i want this impl only for ~T but that makes the drop not work
impl TidyTerminalState {
    fn restore_term (&self) {
        unsafe {
            tcsetattr(self.c_fd, imp::TCSAFLUSH, ptr::addr_of(&self.c_termios_orig));
        }
    }

    /** Explicitly restore the terminal to its pristine state. */
    fn restore(&mut self) {
        self.restore_term();
        self.c_termios_cur = copy self.c_termios_orig;
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
    pub fn raw(&mut self) {
        self.c_termios_cur.c_iflag &= !(
            imp::IXON       // ignore XON/XOFF, i.e. ^S ^Q
            | imp::ISTRIP   // don't strip the 8th bit (?!)
            | imp::INPCK    // don't check for parity errors
            | imp::BRKINT   // don't send SIGINT on BREAK
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
            | imp::ISIG     // ignore usual signal-generating keys
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
            tcsetattr(self.c_fd, imp::TCSAFLUSH, ptr::addr_of(&self.c_termios_cur));
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
