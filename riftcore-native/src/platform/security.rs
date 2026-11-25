use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

const DPAPI_TOKEN_PREFIX: &str = "dpapi:v1:";

/// Verifies that `data` matches the expected 64-character hexadecimal SHA-256 digest.
/// Rejects malformed hex strings and performs constant-time digest comparison.
pub fn sha256_verify(data: &[u8], expected_hex: &str) -> bool {
    let Some(expected_bytes) = decode_sha256_hex(expected_hex) else {
        return false;
    };
    let actual = ring::digest::digest(&ring::digest::SHA256, data);
    constant_time_eq_32(actual.as_ref(), &expected_bytes)
}

/// Computes the lowercase hexadecimal SHA-256 digest of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, data);
    encode_hex(digest.as_ref())
}

/// Streams a file from `path` through `ring::digest::SHA256` and verifies it
/// against `expected_hex` without buffering the whole file in memory.
pub fn sha256_verify_file(path: &Path, expected_hex: &str) -> Result<bool, String> {
    use std::io::Read;

    let Some(expected_bytes) = decode_sha256_hex(expected_hex) else {
        return Ok(false);
    };
    let mut file =
        std::fs::File::open(path).map_err(|err| format!("Could not open file for hashing: {err}"))?;
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| format!("Failed reading file for SHA-256 verification: {err}"))?;
        if read == 0 {
            break;
        }
        ctx.update(&buffer[..read]);
    }
    let actual = ctx.finish();
    Ok(constant_time_eq_32(actual.as_ref(), &expected_bytes))
}

fn constant_time_eq_32(a: &[u8], b: &[u8; 32]) -> bool {
    if a.len() != 32 {
        return false;
    }
    let mut diff = 0u8;
    for (&lhs, &rhs) in a.iter().zip(b.iter()) {
        diff |= lhs ^ rhs;
    }
    diff == 0
}

fn decode_sha256_hex(hex: &str) -> Option<[u8; 32]> {
    let trimmed = hex.trim();
    if trimmed.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = trimmed.as_bytes();
    for (idx, chunk) in bytes.as_chunks::<2>().0.iter().enumerate() {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        out[idx] = (hi << 4) | lo;
    }
    Some(out)
}

fn decode_hex_vec(hex: &str) -> Result<Vec<u8>, String> {
    let trimmed = hex.trim();
    if !trimmed.len().is_multiple_of(2) {
        return Err("Invalid hexadecimal length".to_string());
    }
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    for chunk in trimmed.as_bytes().as_chunks::<2>().0 {
        let hi = hex_nibble(chunk[0]).ok_or_else(|| "Invalid hexadecimal digit".to_string())?;
        let lo = hex_nibble(chunk[1]).ok_or_else(|| "Invalid hexadecimal digit".to_string())?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn encode_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Returns `true` if `encoded` carries the DPAPI ciphertext marker.
pub fn is_dpapi_protected_value(encoded: &str) -> bool {
    encoded.trim().starts_with(DPAPI_TOKEN_PREFIX)
}

/// Protects a UTF-8 secret string using Windows DPAPI (`CurrentUser` scope) and
/// formats it as `"dpapi:v1:<hex>"`. Empty strings encode to `""`.
pub fn protect_secret_string(plaintext: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let protected = dpapi_protect(plaintext.as_bytes())?;
    Ok(format!("{DPAPI_TOKEN_PREFIX}{}", encode_hex(&protected)))
}

/// Unprotects a `"dpapi:v1:<hex>"` value previously returned by [`protect_secret_string`].
/// Empty strings decode to an empty zeroized string.
pub fn unprotect_secret_string(encoded: &str) -> Result<Zeroizing<String>, String> {
    let trimmed = encoded.trim();
    if trimmed.is_empty() {
        return Ok(Zeroizing::new(String::new()));
    }
    let hex_payload = trimmed
        .strip_prefix(DPAPI_TOKEN_PREFIX)
        .ok_or_else(|| "Secret does not have the expected DPAPI prefix".to_string())?;
    let ciphertext = decode_hex_vec(hex_payload)?;
    let raw = dpapi_unprotect(&ciphertext)?;
    let text = String::from_utf8(raw.to_vec())
        .map_err(|_| "Decrypted DPAPI secret is not valid UTF-8".to_string())?;
    Ok(Zeroizing::new(text))
}

#[cfg(windows)]
pub fn dpapi_protect(plaintext: &[u8]) -> Result<Vec<u8>, String> {
    use std::ffi::c_void;

    #[repr(C)]
    struct CryptDataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            p_data_in: *const CryptDataBlob,
            sz_data_descr: *const u16,
            p_optional_entropy: *const CryptDataBlob,
            pv_reserved: *mut c_void,
            p_prompt_struct: *mut c_void,
            dw_flags: u32,
            p_data_out: *mut CryptDataBlob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(hmem: *mut c_void) -> *mut c_void;
        fn GetLastError() -> u32;
    }

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x0000_0001;

    if plaintext.is_empty() {
        return Ok(Vec::new());
    }
    let len = u32::try_from(plaintext.len())
        .map_err(|_| "Secret exceeds maximum DPAPI buffer length".to_string())?;
    let mut input_copy = Zeroizing::new(plaintext.to_vec());
    let in_blob = CryptDataBlob {
        cb_data: len,
        pb_data: input_copy.as_mut_ptr(),
    };
    let mut out_blob = CryptDataBlob {
        cb_data: 0,
        pb_data: std::ptr::null_mut(),
    };
    let descr: [u16; 19] = [
        b'R' as u16, b'i' as u16, b'f' as u16, b't' as u16, b'C' as u16, b'o' as u16,
        b'r' as u16, b'e' as u16, b'.' as u16, b'R' as u16, b'i' as u16, b'o' as u16,
        b't' as u16, b'A' as u16, b'p' as u16, b'i' as u16, b'K' as u16, b'e' as u16,
        0,
    ];

    // SAFETY: `in_blob` points to the live `input_copy` buffer of `len` bytes, `descr` is
    // NUL-terminated UTF-16, and `out_blob` is a valid writable stack struct. On success,
    // `CryptProtectData` allocates `out_blob.pb_data` with `LocalAlloc`, which we always free
    // via `LocalFree` after copying out the ciphertext.
    unsafe {
        let ok = CryptProtectData(
            &in_blob,
            descr.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        );
        if ok == 0 {
            let code = GetLastError();
            if !out_blob.pb_data.is_null() {
                let _ = LocalFree(out_blob.pb_data.cast::<c_void>());
            }
            return Err(format!("CryptProtectData failed with Win32 error {code:#x}"));
        }
        if out_blob.pb_data.is_null() || out_blob.cb_data == 0 {
            if !out_blob.pb_data.is_null() {
                let _ = LocalFree(out_blob.pb_data.cast::<c_void>());
            }
            return Err("CryptProtectData returned an empty buffer".to_string());
        }
        let slice = std::slice::from_raw_parts(out_blob.pb_data, out_blob.cb_data as usize);
        let result = slice.to_vec();
        let _ = LocalFree(out_blob.pb_data.cast::<c_void>());
        Ok(result)
    }
}

#[cfg(not(windows))]
pub fn dpapi_protect(_plaintext: &[u8]) -> Result<Vec<u8>, String> {
    Err("DPAPI encryption is only supported on Windows".to_string())
}

#[cfg(windows)]
pub fn dpapi_unprotect(ciphertext: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
    use std::ffi::c_void;

    #[repr(C)]
    struct CryptDataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptUnprotectData(
            p_data_in: *const CryptDataBlob,
            ppsz_data_descr: *mut *mut u16,
            p_optional_entropy: *const CryptDataBlob,
            pv_reserved: *mut c_void,
            p_prompt_struct: *mut c_void,
            dw_flags: u32,
            p_data_out: *mut CryptDataBlob,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(hmem: *mut c_void) -> *mut c_void;
        fn GetLastError() -> u32;
    }

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x0000_0001;

    if ciphertext.is_empty() {
        return Ok(Zeroizing::new(Vec::new()));
    }
    let len = u32::try_from(ciphertext.len())
        .map_err(|_| "Ciphertext exceeds maximum DPAPI buffer length".to_string())?;
    let mut cipher_copy = ciphertext.to_vec();
    let in_blob = CryptDataBlob {
        cb_data: len,
        pb_data: cipher_copy.as_mut_ptr(),
    };
    let mut out_blob = CryptDataBlob {
        cb_data: 0,
        pb_data: std::ptr::null_mut(),
    };

    // SAFETY: `in_blob` points to `cipher_copy` (`len` valid bytes) and `out_blob` is a
    // writable stack `CryptDataBlob`. When `CryptUnprotectData` returns non-zero, `out_blob.pb_data`
    // owns `out_blob.cb_data` `LocalAlloc`-allocated plaintext bytes, which are copied into a
    // `Zeroizing<Vec<u8>>`, explicitly wiped in-place via `zeroize()`, and freed via `LocalFree`.
    unsafe {
        let ok = CryptUnprotectData(
            &in_blob,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        );
        if ok == 0 {
            let code = GetLastError();
            if !out_blob.pb_data.is_null() {
                let _ = LocalFree(out_blob.pb_data.cast::<c_void>());
            }
            return Err(format!("CryptUnprotectData failed with Win32 error {code:#x}"));
        }
        if out_blob.pb_data.is_null() {
            return Err("CryptUnprotectData returned a null plaintext buffer".to_string());
        }
        let plain_slice =
            std::slice::from_raw_parts_mut(out_blob.pb_data, out_blob.cb_data as usize);
        let result = Zeroizing::new(plain_slice.to_vec());
        plain_slice.zeroize();
        let _ = LocalFree(out_blob.pb_data.cast::<c_void>());
        Ok(result)
    }
}

#[cfg(not(windows))]
pub fn dpapi_unprotect(_ciphertext: &[u8]) -> Result<Zeroizing<Vec<u8>>, String> {
    Err("DPAPI decryption is only supported on Windows".to_string())
}

/// Verifies a Windows PE file's Authenticode signature using `WinVerifyTrust`.
/// Fail-closed: returns `Err` for unsigned files, untrusted publishers, revocation
/// failures, missing files, or any non-zero `WinVerifyTrust` status.
#[cfg(windows)]
pub fn verify_authenticode(path: &Path) -> Result<(), String> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    if !path.is_file() {
        return Err("Authenticode target path does not exist or is not a regular file".to_string());
    }
    let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide_path.contains(&0) {
        return Err("Authenticode target path contains an unexpected NUL character".to_string());
    }
    wide_path.push(0);

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    struct WinTrustFileInfo {
        cb_struct: u32,
        pcwsz_file_path: *const u16,
        h_file: *mut c_void,
        pg_known_subject: *mut Guid,
    }

    #[repr(C)]
    struct WinTrustData {
        cb_struct: u32,
        p_policy_callback_data: *mut c_void,
        p_sip_client_data: *mut c_void,
        dw_ui_choice: u32,
        fdw_revocation_checks: u32,
        dw_union_choice: u32,
        p_file: *mut WinTrustFileInfo,
        dw_state_action: u32,
        h_wvt_state_data: *mut c_void,
        pwsz_url_reference: *mut u16,
        dw_prov_flags: u32,
        dw_ui_context: u32,
        p_signature_settings: *mut c_void,
    }

    #[link(name = "wintrust")]
    extern "system" {
        fn WinVerifyTrust(
            hwnd: *mut c_void,
            pg_action_id: *mut Guid,
            p_wvt_data: *mut WinTrustData,
        ) -> i32;
    }

    const WTD_UI_NONE: u32 = 2;
    const WTD_REVOKE_WHOLECHAIN: u32 = 1;
    const WTD_CHOICE_FILE: u32 = 1;
    const WTD_STATEACTION_VERIFY: u32 = 1;
    const WTD_STATEACTION_CLOSE: u32 = 2;
    const WTD_REVOCATION_CHECK_CHAIN: u32 = 0x0000_0040;

    let mut action_guid = Guid {
        data1: 0x00AA_C56B,
        data2: 0xCD44,
        data3: 0x11D0,
        data4: [0x8C, 0xC2, 0x00, 0xC0, 0x4F, 0xC2, 0x95, 0xEE],
    };

    let mut file_info = WinTrustFileInfo {
        cb_struct: std::mem::size_of::<WinTrustFileInfo>() as u32,
        pcwsz_file_path: wide_path.as_ptr(),
        h_file: std::ptr::null_mut(),
        pg_known_subject: std::ptr::null_mut(),
    };

    let mut trust_data = WinTrustData {
        cb_struct: std::mem::size_of::<WinTrustData>() as u32,
        p_policy_callback_data: std::ptr::null_mut(),
        p_sip_client_data: std::ptr::null_mut(),
        dw_ui_choice: WTD_UI_NONE,
        fdw_revocation_checks: WTD_REVOKE_WHOLECHAIN,
        dw_union_choice: WTD_CHOICE_FILE,
        p_file: &mut file_info,
        dw_state_action: WTD_STATEACTION_VERIFY,
        h_wvt_state_data: std::ptr::null_mut(),
        pwsz_url_reference: std::ptr::null_mut(),
        dw_prov_flags: WTD_REVOCATION_CHECK_CHAIN,
        dw_ui_context: 0,
        p_signature_settings: std::ptr::null_mut(),
    };

    let headless_hwnd = (-1isize) as *mut c_void;

    // SAFETY: `wide_path` stays alive and NUL-terminated for the duration of both
    // `WinVerifyTrust` calls. `file_info` and `trust_data` use exact Win32 ABI layouts with
    // initialized `cb_struct` sizes. `WTD_STATEACTION_CLOSE` is always called afterwards to
    // release `h_wvt_state_data`.
    let status = unsafe {
        let verify_status = WinVerifyTrust(headless_hwnd, &mut action_guid, &mut trust_data);
        trust_data.dw_state_action = WTD_STATEACTION_CLOSE;
        let _ = WinVerifyTrust(headless_hwnd, &mut action_guid, &mut trust_data);
        verify_status
    };

    if status == 0 {
        Ok(())
    } else {
        Err(format!(
            "Authenticode verification failed (WinVerifyTrust status {:#010x})",
            status as u32
        ))
    }
}

#[cfg(not(windows))]
pub fn verify_authenticode(_path: &Path) -> Result<(), String> {
    Err("Authenticode verification is only supported on Windows".to_string())
}

/// Convenience boolean wrapper over [`verify_authenticode`] (fail-closed).
pub fn is_authenticode_valid(path: &Path) -> bool {
    verify_authenticode(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_verify_accepts_valid_case_insensitive_hex() {
        let payload = b"riftcore-update-binary";
        let expected = sha256_hex(payload);
        assert_eq!(expected.len(), 64);
        assert!(sha256_verify(payload, &expected));
        assert!(sha256_verify(payload, &expected.to_ascii_uppercase()));
    }

    #[test]
    fn sha256_verify_rejects_mismatch_or_malformed_hex() {
        let payload = b"riftcore-update-binary";
        let expected = sha256_hex(payload);
        assert!(!sha256_verify(b"tampered", &expected));
        assert!(!sha256_verify(payload, "deadbeef"));
        assert!(!sha256_verify(payload, &"z".repeat(64)));
    }

    #[cfg(windows)]
    #[test]
    fn dpapi_round_trips_secret_string() {
        let secret = "RGAPI-test-secret-0000-1111";
        let protected = protect_secret_string(secret).expect("dpapi protect should succeed");
        assert!(is_dpapi_protected_value(&protected));
        assert!(!protected.contains("RGAPI-"));
        let recovered = unprotect_secret_string(&protected).expect("dpapi unprotect should succeed");
        assert_eq!(recovered.as_str(), secret);
    }

    #[cfg(windows)]
    #[test]
    fn winverifytrust_fails_closed_on_missing_and_unsigned_files() {
        let missing = Path::new("D:\\definitely_missing_riftcore_probe.exe");
        assert!(!is_authenticode_valid(missing));

        let temp_path = std::env::temp_dir().join(format!(
            "riftcore-unsigned-test-{}.exe",
            std::process::id()
        ));
        std::fs::write(&temp_path, b"MZnot-a-signed-pe").expect("write temp probe");
        let valid = is_authenticode_valid(&temp_path);
        let _ = std::fs::remove_file(&temp_path);
        assert!(!valid);
    }
}
