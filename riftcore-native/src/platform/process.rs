use std::path::{Path, PathBuf};

/// Decodes a NUL-terminated UTF-16 slice, stopping strictly at the first NUL (`0`)
/// unit so stale trailing buffer data after the terminator is never included.
pub fn utf16_z_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&unit| unit == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LcuLockfile {
    pub process_name: String,
    pub pid: u32,
    pub port: u16,
    pub token: String,
    pub protocol: String,
    pub install_dir: Option<PathBuf>,
}

#[cfg(windows)]
#[repr(C)]
pub struct ProcessEntry32W {
    pub dw_size: u32,
    pub cnt_usage: u32,
    pub th32_process_id: u32,
    pub th32_default_heap_id: usize,
    pub th32_module_id: u32,
    pub cnt_threads: u32,
    pub th32_parent_process_id: u32,
    pub pc_pri_class_base: i32,
    pub dw_flags: u32,
    pub sz_exe_file: [u16; 260],
}

#[cfg(windows)]
impl Default for ProcessEntry32W {
    fn default() -> Self {
        Self {
            dw_size: std::mem::size_of::<Self>() as u32,
            cnt_usage: 0,
            th32_process_id: 0,
            th32_default_heap_id: 0,
            th32_module_id: 0,
            cnt_threads: 0,
            th32_parent_process_id: 0,
            pc_pri_class_base: 0,
            dw_flags: 0,
            sz_exe_file: [0u16; 260],
        }
    }
}

#[cfg(windows)]
pub fn is_process_running(pid: u32) -> bool {
    use std::ffi::c_void;

    if pid == 0 {
        return false;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
        fn GetExitCodeProcess(process: *mut c_void, code: *mut u32) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;

    // SAFETY: `OpenProcess` is called with valid flags and `pid != 0`. Any non-null
    // returned handle is passed to `GetExitCodeProcess` with a valid stack `u32`
    // pointer and unconditionally closed with `CloseHandle`.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return false;
        }
        let mut exit_code = 0u32;
        let active = GetExitCodeProcess(process, &mut exit_code) != 0 && exit_code == STILL_ACTIVE;
        let _ = CloseHandle(process);
        active
    }
}

#[cfg(not(windows))]
pub fn is_process_running(_pid: u32) -> bool {
    false
}

/// Discovers the full image path of the first running process matching `exe_name`
/// (case-insensitive) via `CreateToolhelp32Snapshot`.
#[cfg(windows)]
pub fn find_process_exe_path(exe_name: &str) -> Option<PathBuf> {
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> *mut c_void;
        fn Process32FirstW(snapshot: *mut c_void, entry: *mut ProcessEntry32W) -> i32;
        fn Process32NextW(snapshot: *mut c_void, entry: *mut ProcessEntry32W) -> i32;
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
        fn QueryFullProcessImageNameW(
            process: *mut c_void,
            flags: u32,
            path: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    // SAFETY: Toolhelp snapshot handle is validated against both `null` and
    // `INVALID_HANDLE_VALUE (-1)` before iterating `ProcessEntry32W` (with `dw_size` set).
    // `QueryFullProcessImageNameW` receives a 1024-unit stack buffer and its written
    // length is clamped via `.get(..)` before slicing; all handles are closed before return.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() || snapshot as isize == -1 {
            return None;
        }

        let mut entry = ProcessEntry32W::default();
        let mut found = None;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let candidate = utf16_z_to_string(&entry.sz_exe_file);
                if candidate.eq_ignore_ascii_case(exe_name) {
                    let process =
                        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32_process_id);
                    if !process.is_null() {
                        let mut buffer = [0u16; 1024];
                        let mut size = buffer.len() as u32;
                        if QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size)
                            != 0
                        {
                            if let Some(slice) = buffer.get(..size as usize) {
                                found = Some(PathBuf::from(String::from_utf16_lossy(slice)));
                            }
                        }
                        let _ = CloseHandle(process);
                    }
                    if found.is_some() {
                        break;
                    }
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
pub fn find_process_exe_path(_exe_name: &str) -> Option<PathBuf> {
    None
}

/// Discovers the running `LeagueClientUx.exe` executable path, if present.
pub fn find_league_client_ux_path() -> Option<PathBuf> {
    find_process_exe_path("LeagueClientUx.exe")
}

/// Builds the ordered list of LCU lockfile candidate paths.
/// `RIFTCORE_LOCKFILE` (when set and non-empty) always takes highest priority,
/// followed by the directory of running `LeagueClientUx.exe`, then standard drives.
pub fn lcu_lockfile_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(override_path) = std::env::var("RIFTCORE_LOCKFILE") {
        let trimmed = override_path.trim();
        if !trimmed.is_empty() {
            candidates.push(PathBuf::from(trimmed));
        }
    }
    if let Some(ux_path) = find_league_client_ux_path() {
        if let Some(parent) = ux_path.parent() {
            candidates.push(parent.join("lockfile"));
        }
    }
    for drive in ['C', 'D', 'E', 'F'] {
        candidates.push(PathBuf::from(format!(
            "{drive}:\\Riot Games\\League of Legends\\lockfile"
        )));
        candidates.push(PathBuf::from(format!(
            "{drive}:\\Riot Games\\League of Legends (PBE)\\lockfile"
        )));
        candidates.push(PathBuf::from(format!(
            "{drive}:\\Program Files\\Riot Games\\League of Legends\\lockfile"
        )));
    }
    candidates
}

/// Parses and validates LCU lockfile contents (`name:pid:port:token:protocol`) without
/// querying OS process liveness (useful for unit tests and override inspection).
pub fn parse_lockfile_str(
    content: &str,
    lockfile_path: &Path,
    require_live_pid: bool,
) -> Option<LcuLockfile> {
    let fields: Vec<&str> = content.trim().split(':').collect();
    if fields.len() < 5 {
        return None;
    }
    let process_name = fields[0].trim();
    let pid = fields[1].trim().parse::<u32>().ok().filter(|&p| p > 0)?;
    let port = fields[2].trim().parse::<u16>().ok().filter(|&p| p > 0)?;
    let token = fields[3].trim();
    let protocol = fields[4].trim();

    if process_name.is_empty()
        || token.is_empty()
        || protocol.is_empty()
        || !token
            .bytes()
            .all(|b| b.is_ascii_graphic() && !matches!(b, b':' | b'@' | b'/' | b'\\'))
    {
        return None;
    }
    if require_live_pid && !is_process_running(pid) {
        return None;
    }

    Some(LcuLockfile {
        process_name: process_name.to_string(),
        pid,
        port,
        token: token.to_string(),
        protocol: protocol.to_string(),
        install_dir: lockfile_path.parent().map(Path::to_path_buf),
    })
}

/// Reads and validates an LCU lockfile on disk.
pub fn read_lcu_lockfile(path: &Path, require_live_pid: bool) -> Option<LcuLockfile> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_lockfile_str(&content, path, require_live_pid)
}

/// Discovers and validates the active League Client lockfile.
pub fn discover_lcu_lockfile() -> Option<LcuLockfile> {
    let env_override = std::env::var("RIFTCORE_LOCKFILE")
        .ok()
        .map(|v| PathBuf::from(v.trim().to_string()))
        .filter(|p| !p.as_os_str().is_empty());

    for candidate in lcu_lockfile_candidates() {
        let is_env_override = env_override.as_ref().is_some_and(|ov| ov == &candidate);
        if let Some(parsed) = read_lcu_lockfile(&candidate, !is_env_override) {
            return Some(parsed);
        }
    }
    None
}

/// Checks if Riot Client is currently running using its local lockfile PID.
pub fn is_riot_client_open() -> bool {
    let Ok(local_app_data) = std::env::var("LOCALAPPDATA") else {
        return false;
    };
    let path = PathBuf::from(local_app_data)
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| text.split(':').nth(1)?.trim().parse::<u32>().ok())
        .is_some_and(is_process_running)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_z_stops_at_first_nul() {
        let buf: Vec<u16> = "LeagueClientUx.exe\0garbageAfterNul.exe\0\0"
            .encode_utf16()
            .collect();
        assert_eq!(utf16_z_to_string(&buf), "LeagueClientUx.exe");
    }

    #[test]
    fn parse_lockfile_rejects_port_injection_and_bad_pid() {
        let path = Path::new("C:\\Riot Games\\League of Legends\\lockfile");
        assert!(
            parse_lockfile_str("LeagueClient:1234:80@evil.com:tok:https", path, false).is_none()
        );
        assert!(parse_lockfile_str("LeagueClient:abc:51234:tok:https", path, false).is_none());
        assert!(parse_lockfile_str("LeagueClient:1234:0:tok:https", path, false).is_none());
        assert!(parse_lockfile_str("LeagueClient:1234:70000:tok:https", path, false).is_none());

        let valid =
            parse_lockfile_str("LeagueClient:1234:51234:secretToken_123:https", path, false)
                .expect("valid lockfile should parse");
        assert_eq!(valid.pid, 1234);
        assert_eq!(valid.port, 51234);
        assert_eq!(valid.token, "secretToken_123");
    }

    #[test]
    fn riftcore_lockfile_env_takes_priority() {
        let prev = std::env::var("RIFTCORE_LOCKFILE").ok();
        std::env::set_var("RIFTCORE_LOCKFILE", "D:\\custom\\test_lockfile");
        let candidates = lcu_lockfile_candidates();
        assert_eq!(
            candidates.first(),
            Some(&PathBuf::from("D:\\custom\\test_lockfile"))
        );
        match prev {
            Some(v) => std::env::set_var("RIFTCORE_LOCKFILE", v),
            None => std::env::remove_var("RIFTCORE_LOCKFILE"),
        }
    }
}
