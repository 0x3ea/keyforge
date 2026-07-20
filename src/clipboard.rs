use crate::sensitive::SecretVec;
use arboard::Clipboard;
use std::thread;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn write_to_clipboard(password: &SecretVec, timeout: u32) -> Result<(), String> {
    let password_text = std::str::from_utf8(password.as_bytes())
        .map_err(|e| format!("generated password is not valid UTF-8: {e}"))?
        .to_owned();

    let mut clipboard = Clipboard::new().map_err(|e| format!("failed to access clipboard: {e}"))?;

    clipboard
        .set_text(password_text.clone())
        .map_err(|e| format!("failed to write password to clipboard: {e}"))?;

    install_interrupt_handler()?;

    println!("Password copied to clipboard. It will be cleared in {timeout} seconds.");

    wait_for_timeout_or_interrupt(timeout);
    clear_if_unchanged(&mut clipboard, &password_text)?;

    if INTERRUPTED.load(Ordering::SeqCst) {
        return Err("interrupted".to_string());
    }
    Ok(())
}

fn install_interrupt_handler() -> Result<(), String> {
    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    })
    .map_err(|e| format!("failed to install interrupt handler: {e}"))
}

fn wait_for_timeout_or_interrupt(timeout: u32) {
    let deadline = Instant::now() + Duration::from_secs(timeout as u64);

    while Instant::now() < deadline {
        if INTERRUPTED.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn clear_if_unchanged(clipboard: &mut Clipboard, expected: &str) -> Result<(), String> {
    let current = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => return Ok(()),
    };

    if current == expected {
        clipboard
            .set_text("")
            .map_err(|e| format!("failed to clear clipboard: {e}"))?;
    }
    Ok(())
}
