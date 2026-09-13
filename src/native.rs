/// Only touches top-level windows belonging to the emulator process we launched.
/// GNU Make's Windows distribution uses the active ANSI code page for its CWD.
/// Resolve the existing directory to its Windows short alias without moving assets.
pub fn build_directory(path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    if path.to_string_lossy().is_ascii() {
        return Ok(path.into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetShortPathNameW(long: *const u16, short: *mut u16, size: u32) -> u32;
        }
        let input = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let size = unsafe { GetShortPathNameW(input.as_ptr(), std::ptr::null_mut(), 0) };
        if size != 0 {
            let mut output = vec![0; size as usize];
            let used = unsafe { GetShortPathNameW(input.as_ptr(), output.as_mut_ptr(), size) };
            if used > 0 && used < size {
                let short = std::path::PathBuf::from(std::ffi::OsString::from_wide(
                    &output[..used as usize],
                ));
                if short.to_string_lossy().is_ascii() {
                    return Ok(short);
                }
            }
        }
        Err("The MIPS Make tool needs an ASCII working-directory alias. Windows short names are unavailable on this volume; use an ASCII project folder for native builds.".into())
    }
    #[cfg(not(windows))]
    {
        Ok(path.into())
    }
}

#[cfg(windows)]
pub fn emulator_window(pid: u32, visible: bool) {
    #[repr(C)]
    struct Request {
        pid: u32,
        visible: bool,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(isize, isize) -> i32,
            data: isize,
        ) -> i32;
        fn GetWindowThreadProcessId(window: isize, pid: *mut u32) -> u32;
        fn ShowWindow(window: isize, command: i32) -> i32;
        fn GetWindow(window: isize, command: u32) -> isize;
    }
    unsafe extern "system" fn visit(window: isize, data: isize) -> i32 {
        let request = unsafe { &*(data as *const Request) };
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(window, &mut pid);
        }
        if pid == request.pid && unsafe { GetWindow(window, 4) } == 0 {
            unsafe {
                ShowWindow(window, if request.visible { 8 } else { 0 });
            }
        }
        1
    }
    let request = Request { pid, visible };
    unsafe {
        EnumWindows(visit, &request as *const Request as isize);
    }
}
#[cfg(not(windows))]
pub fn emulator_window(_pid: u32, _visible: bool) {}
