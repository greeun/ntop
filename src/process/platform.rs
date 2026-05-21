#[cfg(target_os = "macos")]
mod macos {
    use std::mem;
    use std::os::raw::c_void;

    const PROC_PIDTASKINFO: i32 = 4;
    const PROC_PIDLISTFDS: i32 = 1;
    const RUSAGE_INFO_V2: i32 = 2;

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

    // Matches macOS <sys/resource.h> struct rusage_info_v2.
    // Field order/size must stay aligned with the kernel definition.
    #[repr(C)]
    struct RUsageInfoV2 {
        ri_uuid: [u8; 16],
        ri_user_time: u64,
        ri_system_time: u64,
        ri_pkg_idle_wkups: u64,
        ri_interrupt_wkups: u64,
        ri_pageins: u64,
        ri_wired_size: u64,
        ri_resident_size: u64,
        ri_phys_footprint: u64,
        ri_proc_start_abstime: u64,
        ri_proc_exit_abstime: u64,
        ri_child_user_time: u64,
        ri_child_system_time: u64,
        ri_child_pkg_idle_wkups: u64,
        ri_child_interrupt_wkups: u64,
        ri_child_pageins: u64,
        ri_child_elapsed_abstime: u64,
        ri_diskio_bytesread: u64,
        ri_diskio_byteswritten: u64,
    }

    extern "C" {
        fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut c_void,
            buffersize: i32,
        ) -> i32;

        fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut c_void) -> i32;
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

    /// Returns the process's `phys_footprint` in bytes — the same value macOS
    /// Activity Monitor shows in its "Memory" column. Falls back to `None`
    /// when the call fails (e.g. process has exited or insufficient permissions).
    pub fn phys_footprint(pid: u32) -> Option<u64> {
        unsafe {
            let mut info: RUsageInfoV2 = mem::zeroed();
            let ret = proc_pid_rusage(
                pid as i32,
                RUSAGE_INFO_V2,
                &mut info as *mut _ as *mut c_void,
            );
            if ret == 0 {
                Some(info.ri_phys_footprint)
            } else {
                None
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::{open_fd_count, phys_footprint, thread_count};

#[cfg(windows)]
mod windows {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next,
        TH32CS_SNAPTHREAD, THREADENTRY32,
    };

    pub fn thread_count(pid: u32) -> u32 {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot.is_null() {
                return 0;
            }

            let mut entry: THREADENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;

            let mut count = 0u32;
            if Thread32First(snapshot, &mut entry) != 0 {
                loop {
                    if entry.th32OwnerProcessID == pid {
                        count += 1;
                    }
                    if Thread32Next(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            count
        }
    }

    pub fn open_fd_count(_pid: u32) -> u32 {
        0
    }
}

#[cfg(windows)]
pub use windows::{open_fd_count, thread_count};

#[cfg(not(any(target_os = "macos", windows)))]
pub fn thread_count(_pid: u32) -> u32 { 0 }

#[cfg(not(any(target_os = "macos", windows)))]
pub fn open_fd_count(_pid: u32) -> u32 { 0 }

#[cfg(not(target_os = "macos"))]
pub fn phys_footprint(_pid: u32) -> Option<u64> { None }
