#[cfg(target_os = "macos")]
mod macos {
    use std::mem;
    use std::os::raw::c_void;

    const PROC_PIDTASKINFO: i32 = 4;
    const PROC_PIDLISTFDS: i32 = 1;

    #[repr(C)]
    struct ProcTaskInfo {
        pti_virtual_size: u64,
        pti_resident_size: u64,
        pti_total_user: u64,
        pti_total_system: u64,
        pti_threads_user: u64,
        pti_threads_system: u64,
        pti_policy: i32,
        pti_faults: i32,
        pti_pageins: i32,
        pti_cow_faults: i32,
        pti_messages_sent: i32,
        pti_messages_received: i32,
        pti_syscalls_mach: i32,
        pti_syscalls_unix: i32,
        pti_csw: i32,
        pti_threadnum: i32,
        pti_numrunning: i32,
        pti_priority: i32,
    }

    #[repr(C)]
    struct ProcFdInfo {
        proc_fd: i32,
        proc_fdtype: u32,
    }

    extern "C" {
        fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut c_void,
            buffersize: i32,
        ) -> i32;
    }

    pub fn thread_count(pid: u32) -> u32 {
        unsafe {
            let mut info: ProcTaskInfo = mem::zeroed();
            let size = mem::size_of::<ProcTaskInfo>() as i32;
            let ret = proc_pidinfo(pid as i32, PROC_PIDTASKINFO, 0, &mut info as *mut _ as *mut c_void, size);
            if ret > 0 { info.pti_threadnum as u32 } else { 0 }
        }
    }

    pub fn open_fd_count(pid: u32) -> u32 {
        unsafe {
            let size = proc_pidinfo(pid as i32, PROC_PIDLISTFDS, 0, std::ptr::null_mut(), 0);
            if size > 0 { (size as usize / mem::size_of::<ProcFdInfo>()) as u32 } else { 0 }
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::{open_fd_count, thread_count};

#[cfg(not(target_os = "macos"))]
pub fn thread_count(_pid: u32) -> u32 { 0 }

#[cfg(not(target_os = "macos"))]
pub fn open_fd_count(_pid: u32) -> u32 { 0 }
