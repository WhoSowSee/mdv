use std::io;
use std::process::Command;

#[cfg(unix)]
pub(super) struct InterruptGuard {
    previous: libc::sigaction,
}

#[cfg(unix)]
impl InterruptGuard {
    pub(super) fn install() -> io::Result<Self> {
        // SAFETY: An all-zero sigaction is a valid base before setting its handler and mask.
        let mut ignored = unsafe { std::mem::zeroed::<libc::sigaction>() };
        ignored.sa_sigaction = libc::SIG_IGN;
        // SAFETY: `ignored.sa_mask` points to initialized storage owned by this function.
        let mask_result = unsafe { libc::sigemptyset(&mut ignored.sa_mask) };
        if mask_result != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `previous` is writable storage and `ignored` remains valid for the call.
        let mut previous = unsafe { std::mem::zeroed::<libc::sigaction>() };
        let install_result = unsafe { libc::sigaction(libc::SIGINT, &ignored, &mut previous) };
        if install_result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { previous })
    }
}

#[cfg(unix)]
impl Drop for InterruptGuard {
    fn drop(&mut self) {
        // SAFETY: `previous` was initialized by sigaction for SIGINT in `install`.
        unsafe {
            let _ = libc::sigaction(libc::SIGINT, &self.previous, std::ptr::null_mut());
        }
    }
}

#[cfg(unix)]
pub(super) fn configure_child_interrupt(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    // SAFETY: The closure calls only libc signal functions before exec and captures no state.
    unsafe {
        command.pre_exec(|| {
            let mut default_action = std::mem::zeroed::<libc::sigaction>();
            default_action.sa_sigaction = libc::SIG_DFL;
            if libc::sigemptyset(&mut default_action.sa_mask) != 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::sigaction(libc::SIGINT, &default_action, std::ptr::null_mut()) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(windows)]
pub(super) struct InterruptGuard;

#[cfg(windows)]
const CTRL_C_EVENT: u32 = 0;
#[cfg(windows)]
const CTRL_BREAK_EVENT: u32 = 1;

#[cfg(windows)]
unsafe extern "system" fn ignore_console_interrupt(control_type: u32) -> i32 {
    match control_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT => 1,
        _ => 0,
    }
}

#[cfg(windows)]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(
        handler: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
}

#[cfg(windows)]
impl InterruptGuard {
    pub(super) fn install() -> io::Result<Self> {
        // SAFETY: The callback has the required Windows ABI and remains valid for this process.
        let installed = unsafe { SetConsoleCtrlHandler(Some(ignore_console_interrupt), 1) };
        if installed == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self)
    }
}

#[cfg(windows)]
impl Drop for InterruptGuard {
    fn drop(&mut self) {
        // SAFETY: This removes the same process-local callback registered in `install`.
        unsafe {
            let _ = SetConsoleCtrlHandler(Some(ignore_console_interrupt), 0);
        }
    }
}

#[cfg(windows)]
pub(super) fn configure_child_interrupt(_: &mut Command) {}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn ignores_only_interrupt_events() {
        assert_eq!(unsafe { ignore_console_interrupt(CTRL_C_EVENT) }, 1);
        assert_eq!(unsafe { ignore_console_interrupt(CTRL_BREAK_EVENT) }, 1);
        for event in [2, 5, 6] {
            assert_eq!(unsafe { ignore_console_interrupt(event) }, 0);
        }
    }
}
