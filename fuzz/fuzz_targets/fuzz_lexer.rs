#![no_main]

use libfuzzer_sys::fuzz_target;
use opal_lexer::lex;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // The lexer should never panic on any input
        let _ = lex(s);
    }
});
