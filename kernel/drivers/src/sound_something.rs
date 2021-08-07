use kernel::hil::sensors::SoundPressure;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;
use kernel::process::{Error, ProcessId};

pub const DRIVER_NUM: usize = 0xa000F;

pub struct SoundSomething<'a> {
    sound: &'a dyn SoundPressure<'a>
}

impl<'a> SoundSomething<'a> {
    pub fn new(sound: &'a dyn SoundPressure<'a>) -> Self {
        Self { sound }
    }

    pub fn read(&self) -> u8 {
        // self.sound.enable();
        // self.sound.disable();
        unimplemented!()

    }
}

impl<'a> SyscallDriver for SoundSomething<'a> {
    fn allocate_grant(
        &self, 
        _: kernel::ProcessId
    ) -> core::result::Result<(), kernel::process::Error> {
        Ok(())
    }
}
