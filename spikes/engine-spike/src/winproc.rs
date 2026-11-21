//! Per-process measurement and control: private working set, CPU time,
//! Job Objects and EcoQoS. Win32 only.

use std::mem::size_of;

use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob, JOB_OBJECT_CPU_RATE_CONTROL_ENABLE,
    JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_CPU_RATE_CONTROL_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectCpuRateControlInformation, JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX2,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetProcessInformation, GetProcessTimes, OpenProcess,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
    ProcessPowerThrottling, SetProcessInformation,
};
use windows::core::BOOL;

/// Owned process handle, closed on drop.
struct Proc(HANDLE);
impl Drop for Proc {
    fn drop(&mut self) {
        // SAFETY: the handle came from OpenProcess and is closed exactly once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn open(
    pid: u32,
    access: windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS,
) -> Option<Proc> {
    // SAFETY: plain Win32 call; the returned handle is owned by `Proc`.
    unsafe { OpenProcess(access, false, pid).ok().map(Proc) }
}

/// Private working set in bytes (what Task Manager's "Memory" column shows),
/// via PROCESS_MEMORY_COUNTERS_EX2 (Windows 10/11 22H2 + Sept 2023 update).
pub fn private_ws(pid: u32) -> Option<u64> {
    counters(pid).map(|c| c.0)
}

/// (private working set, private commit "PrivateUsage") in bytes.
pub fn counters(pid: u32) -> Option<(u64, u64)> {
    let p = open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let mut c = PROCESS_MEMORY_COUNTERS_EX2 {
        cb: size_of::<PROCESS_MEMORY_COUNTERS_EX2>() as u32,
        ..Default::default()
    };
    // SAFETY: `c` is a valid, correctly sized EX2 struct; the API takes the
    // base-struct pointer and uses `cb` to know the real size.
    unsafe {
        GetProcessMemoryInfo(p.0, &mut c as *mut _ as *mut PROCESS_MEMORY_COUNTERS, c.cb).ok()?;
    }
    Some((c.PrivateWorkingSetSize as u64, c.PrivateUsage as u64))
}

/// Total CPU time (user + kernel) in milliseconds.
pub fn cpu_ms(pid: u32) -> Option<u64> {
    let p = open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let (mut c, mut e, mut k, mut u) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    // SAFETY: all out-pointers are valid FILETIMEs on the stack.
    unsafe { GetProcessTimes(p.0, &mut c, &mut e, &mut k, &mut u).ok()? };
    let ft = |f: FILETIME| ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64;
    Some((ft(k) + ft(u)) / 10_000)
}

pub struct Job(HANDLE);

/// Create a job with KILL_ON_JOB_CLOSE and put our own process in it. Must run
/// before the WebView2 environment is created, so the browser process (our
/// child) and everything it launches inherits the job.
pub fn put_self_in_job() -> windows::core::Result<Job> {
    // SAFETY: standard job-object setup; `info` is a valid, correctly sized
    // struct and the handle is kept for the process lifetime.
    unsafe {
        let job = CreateJobObjectW(None, None)?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )?;
        AssignProcessToJobObject(job, GetCurrentProcess())?;
        Ok(Job(job))
    }
}

impl Job {
    /// Hard CPU cap for the whole job, in percent of total machine CPU.
    pub fn set_cpu_cap(&self, percent: u32) -> windows::core::Result<()> {
        let mut info = JOBOBJECT_CPU_RATE_CONTROL_INFORMATION {
            ControlFlags: JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP,
            ..Default::default()
        };
        // CpuRate is in 1/100 of a percent.
        info.Anonymous.CpuRate = percent * 100;
        // SAFETY: valid struct of the size we pass; job handle is live.
        unsafe {
            SetInformationJobObject(
                self.0,
                JobObjectCpuRateControlInformation,
                &info as *const _ as *const _,
                size_of::<JOBOBJECT_CPU_RATE_CONTROL_INFORMATION>() as u32,
            )
        }
    }

    pub fn contains(&self, pid: u32) -> Option<bool> {
        let p = open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
        let mut r = BOOL(0);
        // SAFETY: both handles are live; `r` is a valid out-pointer.
        unsafe { IsProcessInJob(p.0, Some(self.0), &mut r).ok()? };
        Some(r.as_bool())
    }
}

/// Turn EcoQoS (execution-speed throttling) on or off for a process.
pub fn set_ecoqos(pid: u32, on: bool) -> windows::core::Result<()> {
    let p = open(pid, PROCESS_SET_INFORMATION).ok_or_else(windows::core::Error::from_thread)?;
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: if on {
            PROCESS_POWER_THROTTLING_EXECUTION_SPEED
        } else {
            0
        },
    };
    // SAFETY: valid struct and size for ProcessPowerThrottling.
    unsafe {
        SetProcessInformation(
            p.0,
            ProcessPowerThrottling,
            &state as *const _ as *const _,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    }
}

/// Read back (ControlMask, StateMask) of the power-throttling state.
pub fn get_ecoqos(pid: u32) -> Option<(u32, u32)> {
    let p = open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let mut state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ..Default::default()
    };
    // SAFETY: valid out-struct and size.
    unsafe {
        GetProcessInformation(
            p.0,
            ProcessPowerThrottling,
            &mut state as *mut _ as *mut _,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
        .ok()?;
    }
    Some((state.ControlMask, state.StateMask))
}
