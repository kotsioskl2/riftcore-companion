// RiftCore Native — services::updater (Wave 0 stub)
// Will host: SHA-256 (ring::digest::SHA256) + WinVerifyTrust Authenticode validation,
//            staging in %LOCALAPPDATA%\RiftCore\updates, browser fallback on failure.
// No hardcoded secrets; API key sourced from DPAPI-protected config (platform::security).
