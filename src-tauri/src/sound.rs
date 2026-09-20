//! The optional notification sound.
//!
//! The reminder is a custom window rather than a native toast, so nothing plays a sound
//! for us. Playing it from Rust rather than from the webview is deliberate: the reminder
//! window is created unfocused and never receives a user gesture, so Chromium's autoplay
//! policy would silently refuse to start a WebAudio context inside it.
//!
//! Every platform here plays the user's *configured* notification sound rather than an
//! asset of ours, and every failure is ignored — a missing sound daemon must never stop a
//! reminder from being shown.

/// Plays the system notification sound, if this platform has one we can reach.
pub fn play_notification() {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Diagnostics::Debug::MessageBeep;
        use windows_sys::Win32::UI::WindowsAndMessaging::MB_OK;

        // Asynchronous, and MB_OK maps to whatever the user chose for "Default Beep".
        unsafe { MessageBeep(MB_OK) };
    }

    #[cfg(target_os = "macos")]
    play_detached(std::process::Command::new("afplay").arg("/System/Library/Sounds/Ping.aiff"));

    // No daemon, no sound. Reminders still appear, which is the part that matters.
    #[cfg(all(unix, not(target_os = "macos")))]
    play_detached(std::process::Command::new("canberra-gtk-play").args(["-i", "message"]));
}

/// Spawns a player and reaps it on a throwaway thread. Left unreaped, one zombie per
/// reminder would accumulate for as long as the app runs, which for this app is months.
#[cfg(unix)]
fn play_detached(command: &mut std::process::Command) {
    if let Ok(mut player) = command.spawn() {
        std::thread::spawn(move || {
            let _ = player.wait();
        });
    }
}
