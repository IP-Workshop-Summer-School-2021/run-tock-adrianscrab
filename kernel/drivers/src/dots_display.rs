use kernel::hil::led::Led;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;
use kernel::process::{Error, ProcessId};

use core::cell::Cell;

pub const DRIVER_NUM: usize = 0xa0001;

// https://www.dafont.com/5x5.font
const DIGITS:[u32; 10] = [
    0b11111_10011_10101_11001_11111,    // 0
    0b00100_01100_00100_00100_01110,    // 1
    0b11110_00001_01110_10000_11111,    // 2
    0b11111_00001_01110_00001_11111,    // 3
    0b10000_10000_10100_11111_00100,    // 4
    0b11111_10000_11110_00001_11110,    // 5
    0b11111_10000_11111_10001_11111,    // 6
    0b11111_00001_00010_00100_00100,    // 7
    0b11111_10001_11111_10001_11111,    // 8
    0b11111_10001_11111_00001_11111,    // 9
];

pub struct DotsDisplay<'a, L: Led> {
    leds: &'a [&'a L; 25],
    digit_displayed: Cell<char>,
}

impl<'a, L: Led> DotsDisplay<'a, L> {
    pub fn new(leds: &'a [&'a L; 25]) -> Self {
        Self { 
            leds, 
            digit_displayed: Cell::new('0') 
        }
    }

    fn display(&self, digit: char) {
        let digit_index = digit as usize - '0' as usize;
        let current_digit = DIGITS[digit_index];
        for idx in 0..25 {
            let bit = (current_digit >> (24 - idx)) & 0x1;
            match bit {
                1 => self.leds[idx].on(),
                _ => self.leds[idx].off(),
            }
        }
    }
}

impl<'a, L: Led> SyscallDriver for DotsDisplay<'a, L> {
    fn command(
        &self,
        command_num: usize,
        r2: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> CommandReturn {
        // CommandReturn::failure(ErrorCode::NOSUPPORT)
        match command_num {
            // driver presence verification
            0 => CommandReturn::success(),
            // print digit in arg r2
            1 => match char::from_u32(r2 as u32) {
                Some(digit) => {
                    if digit >= '0' && digit <= '9' {
                        self.display(digit);
                        // set new digit displayed
                        self.digit_displayed.set(digit);
                        CommandReturn::success()
                    } else {
                        CommandReturn::failure(ErrorCode::INVAL)
                    }
                },
                None => CommandReturn::failure(ErrorCode::INVAL),
            },
            // print next or previous digit stored in `digit_displayed`
            // previous is r2 == 0
            // next is r2 == 1
            2 => match r2 {
                0 => {
                    if self.digit_displayed.get() > '0' {
                        self.digit_displayed.set(
                            (self.digit_displayed.get() as u8 - 1) as char
                        );
                        self.display(self.digit_displayed.get());
                    }
                    CommandReturn::success()
                }
                1 => {
                    if self.digit_displayed.get() < '9' {
                        self.digit_displayed.set(
                            (self.digit_displayed.get() as u8 + 1) as char
                        );
                        self.display(self.digit_displayed.get());
                    }
                    CommandReturn::success()
                }, 
                _ => CommandReturn::failure(ErrorCode::INVAL),
            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(
        &self, 
        _: ProcessId
    ) -> core::result::Result<(), Error> {
        Ok(()) 
    }
}