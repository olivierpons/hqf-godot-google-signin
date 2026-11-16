#![no_main]

use hqf_godot_google_signin::IdTokenClaims;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(token) = std::str::from_utf8(data) {
        let _ = IdTokenClaims::from_jwt(token);
    }
});
