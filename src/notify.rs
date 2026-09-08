use std::io::Write;
#[allow(unused_imports)]
use std::process::{Command, Stdio};

pub fn bell() {
    print!("\x07");
    let _ = std::io::stdout().flush();
}

/// Best-effort desktop notification. Never fails loudly: if the platform
/// tool isn't installed, this just does nothing.
#[allow(unused_variables)]
pub fn system_notify(title: &str, body: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send")
            .arg(title)
            .arg(body)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!("display notification {:?} with title {:?}", body, title);
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
