use kernel::debug;
use kernel::grant::Grant;
use kernel::hil::led::Led;
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::process::{Error, ProcessId};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;
use kernel::errorcode::into_statuscode;

use core::cell::Cell;

pub const DRIVER_NUM: usize = 0xa0001;

#[derive(Default)]
pub struct AppData {
    buffer: ReadOnlyProcessBuffer,
}

// https://www.dafont.com/5x5.font
const DIGITS: [u32; 10] = [
    0b11111_10011_10101_11001_11111, // 0
    0b00100_01100_00100_00100_01110, // 1
    0b11110_00001_01110_10000_11111, // 2
    0b11111_00001_01110_00001_11111, // 3
    0b10000_10000_10100_11111_00100, // 4
    0b11111_10000_11110_00001_11110, // 5
    0b11111_10000_11111_10001_11111, // 6
    0b11111_00001_00010_00100_00100, // 7
    0b11111_10001_11111_10001_11111, // 8
    0b11111_10001_11111_00001_11111, // 9
];

#[derive(Clone,Copy,PartialEq)]
enum State {
    Idle,
    Printing{
        process_id: ProcessId,
        ms: usize,
        position: usize,
        len: usize
    },
}

pub struct DotsTextDisplay<'a, L: Led, A: Alarm<'a>> {
    leds: &'a [&'a L; 25],
    digit_displayed: Cell<char>,
    alarm: &'a A,
    app_data: Grant<AppData, 1>,
    state: Cell<State>,
}

impl<'a, L: Led, A: Alarm<'a>> DotsTextDisplay<'a, L, A> {
    pub fn new(leds: &'a [&'a L; 25], alarm: &'a A, app_data: Grant<AppData, 1>) -> Self {
        Self {
            leds,
            digit_displayed: Cell::new('0'),
            alarm,
            app_data,
            state: Cell::new(State::Idle),
        }
    }

    // pub fn set_timeout(&self) {
    //     self.alarm
    //         .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(1000));
    // }

    fn display_next_digit(&self) -> Result<(), ErrorCode> {
        match self.state.get () {
            State::Printing{
                process_id,
                ms,
                position,
                len
            } => {
                let res = self.app_data.enter(process_id, |data, upcalls| {
                    let res = data.buffer.enter (|buffer| {
                        if position < buffer.len() && position < len {
                            self.display(buffer[position].get() as char);
                            
                            self.state.set (State::Printing{
                                process_id,
                                ms,
                                position: position+1,
                                len
                            });
                            true
                        }
                        else 
                        {
                            upcalls.schedule_upcall (0, (0,0,0)).ok();
                            self.state.set(State::Idle);
                            false
                        }
                    });
                    if let Err(error) = res {
                        upcalls.schedule_upcall (0, (into_statuscode (error.into()),0,0)).ok();
                    };
                    res
                });
                match res {
                    Ok(Ok(next_print)) => {
                        if next_print {
                            self.alarm.set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(ms as u32));
                        }
                        Ok(())
                    },
                    Ok(Err(error)) => {
                        self.state.set(State::Idle);
                        Err(error.into())
                    }
                    Err(error) => 
                    {
                        self.state.set(State::Idle);
                        Err(error.into())
                    }
                }
            }
            State::Idle => {
                Err(ErrorCode::FAIL)
                // unreachable! ()
            }
        }
    }

    fn display(&self, digit: char) {
        let digit_index = digit as usize - '0' as usize;
        let current_digit = DIGITS[digit_index];
        for index in 0..25 {
            let bit = (current_digit >> (24 - index)) & 0x1;
            if bit == 1 {
                self.leds[index].on();
            } else {
                self.leds[index].off();
            }
        }
    }
}

impl<'a, L: Led, A: Alarm<'a>> SyscallDriver for DotsTextDisplay<'a, L, A> {
    fn command(
        &self,
        command_num: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        // CommandReturn::failure(ErrorCode::NOSUPPORT)
        match command_num {
            // driver presence verification
            0 => CommandReturn::success(),
            1 => {
                if self.state.get() == State::Idle {
                    self.state.set(State::Printing{
                        process_id,
                        ms: r3,
                        position: 0,
                        len: r2
                    });
                    if let Err(error) = self.display_next_digit() {
                        CommandReturn::failure(error)
                    }
                    else
                    {
                        CommandReturn::success()
                    }
                    /*
                    match self.display_next_digit() {
                        Err(error) => ...
                        _ => ...
                    }
                    */
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            },
            // print next or previous digit stored in `digit_displayed`
            // previous is r2 == 0
            // next is r2 == 1
            2 => match r2 {
                0 => {
                    if self.digit_displayed.get() > '0' {
                        self.digit_displayed
                            .set((self.digit_displayed.get() as u8 - 1) as char);
                        self.display(self.digit_displayed.get());
                    }
                    CommandReturn::success()
                }
                1 => {
                    if self.digit_displayed.get() < '9' {
                        self.digit_displayed
                            .set((self.digit_displayed.get() as u8 + 1) as char);
                        self.display(self.digit_displayed.get());
                    }
                    CommandReturn::success()
                }
                _ => CommandReturn::failure(ErrorCode::INVAL),
            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), Error> {
        self.app_data.enter(process_id, |_, _| {})
    }

    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_num: usize,
        mut buffer: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        match allow_num {
            0 => {
                let res = self.app_data.enter(process_id, |data, _| {
                    core::mem::swap(&mut data.buffer, &mut buffer);
                });
                match res {
                    Ok(()) => Ok(buffer),
                    Err(error) => Err((buffer, error.into())),
                }
            }
            _ => Err((buffer, ErrorCode::NOSUPPORT)),
        }
    }
}

impl<'a, L: Led, A: Alarm<'a>> AlarmClient for DotsTextDisplay<'a, L, A> {
    fn alarm(&self) {
        debug!("fired");
        let _ = self.display_next_digit();
    }
}
