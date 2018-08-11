//! With this module you can perform actions that are input related.
//! Like reading a line, reading a character and reading asynchronously.

mod input;

#[cfg(not(target_os = "windows"))]
mod unix_input;
#[cfg(target_os = "windows")]
mod windows_input;

#[cfg(not(target_os = "windows"))]
use self::unix_input::UnixInput;
#[cfg(target_os = "windows")]
use self::windows_input::WindowsInput;

pub use self::input::{input, TerminalInput};
use super::Stdout;

use std::io::{self, Read};
use std::sync::{mpsc, Arc};
use Screen;

/// This trait defines the actions that can be preformed with the terminal color.
/// This trait can be implemented so that an concrete implementation of the ITerminalColor can forfill
/// the wishes to work on an specific platform.
///
/// ## For example:
///
/// This trait is implemented for Windows and UNIX systems.
/// Unix is using the tty and windows is using libc C functions to read the input.
trait ITerminalInput {
    /// Read one line from the user input
    fn read_line(&self, screen_manger: &Arc<Stdout>) -> io::Result<String>;
    /// Read one character from the user input
    fn read_char(&self, screen_manger: &Arc<Stdout>) -> io::Result<char>;
    /// Read the input asynchronously from the user.
    fn read_async(&self, screen_manger: &Arc<Stdout>) -> AsyncReader;
    ///  Read the input asynchronously until a certain character is hit.
    fn read_until_async(&self, delimiter: u8, screen_manger: &Arc<Stdout>) -> AsyncReader;
}

/// This is a wrapper for reading from the input asynchronously.
/// This wrapper has a channel receiver that receives the input from the user whenever it typed something.
/// You only need to check whether there are new characters available.
pub struct AsyncReader {
    recv: mpsc::Receiver<io::Result<u8>>,
}

impl Read for AsyncReader {
    /// Read from the byte stream.
    ///
    /// This will never block, but try to drain the event queue until empty. If the total number of
    /// bytes written is lower than the buffer's length, the event queue is empty or that the event
    /// stream halted.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;

        loop {
            if total >= buf.len() {
                break;
            }

            match self.recv.try_recv() {
                Ok(Ok(b)) => {
                    buf[total] = b;
                    total += 1;
                }
                Ok(Err(e)) => return Err(e),
                Err(_) => break,
            }
        }

        Ok(total)
    }
}
