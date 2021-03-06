use crossterm_winapi::{ConsoleMode, Handle};
use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

use lazy_static::lazy_static;

use crate::Result;

/// Toggle virtual terminal processing.
///
/// This method attempts to toggle virtual terminal processing for this
/// console. If there was a problem toggling it, then an error returned.
/// On success, the caller may assume that toggling it was successful.
///
/// When virtual terminal processing is enabled, characters emitted to the
/// console are parsed for VT100 and similar control character sequences
/// that control color and other similar operations.
pub(crate) fn set_virtual_terminal_processing(yes: bool) -> Result<()> {
    let mask = ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    let console_mode = ConsoleMode::from(Handle::current_out_handle()?);
    let old_mode = console_mode.mode()?;

    let new_mode = if yes {
        old_mode | mask
    } else {
        old_mode & !mask
    };

    if old_mode != new_mode {
        console_mode.set_mode(new_mode)?;
    }

    Ok(())
}

lazy_static! {
    static ref SUPPORTS_ANSI_ESCAPE_CODES: bool = {
        // Some terminals on windows like GitBash can't use WinaApi calls directly
        // so when we try to enable the ANSI-flag for windows this won't work.
        // Because of that we should check first if the TERM-variable is set
        // and see if the current terminal is a terminal who does support ANSI.
        if is_specific_term() {
            true
        } else {
            // if it is not listed we should try with WinApi to check if we do support ANSI-codes.
            set_virtual_terminal_processing(true)
                .map(|_| true)
                .unwrap_or(false)
        }
    };
}

/// Checks if the current terminal supports ansi escape sequences
pub fn supports_ansi() -> bool {
    *SUPPORTS_ANSI_ESCAPE_CODES
}

// Checks if the 'TERM' environment variable is set to check if the terminal supports ANSI-codes.
// I got the list of terminals from here: https://github.com/keqingrong/supports-ansi/blob/master/index.js
fn is_specific_term() -> bool {
    const TERMS: [&str; 15] = [
        "xterm",  // xterm, PuTTY, Mintty
        "rxvt",   // RXVT
        "eterm",  // Eterm
        "screen", // GNU screen, tmux
        "tmux",   // tmux
        "vt100", "vt102", "vt220", "vt320",   // DEC VT series
        "ansi",    // ANSI
        "scoansi", // SCO ANSI
        "cygwin",  // Cygwin, MinGW
        "linux",   // Linux console
        "konsole", // Konsole
        "bvterm",  // Bitvise SSH Client
    ];

    match std::env::var("TERM") {
        Ok(val) => val != "dumb" || TERMS.contains(&val.as_str()),
        Err(_) => false,
    }
}
