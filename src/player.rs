use std::{
    fmt::Display,
    process::{Command, Stdio},
    thread,
};

pub fn play<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: FnOnce(Result<(), std::io::Error>) + std::marker::Send + 'static,
>(
    url: S,
    callback: F,
) {
    log::debug!("Playing video with url: {}", url);
    let player_str = std::env::var("PLAYER").unwrap_or("mpv --ytdl".to_string());
    open_with(url, player_str, callback);
}

pub fn open_with<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: FnOnce(Result<(), std::io::Error>) + std::marker::Send + 'static,
>(
    url: S,
    command: String,
    callback: F,
) {
    thread::spawn(move || {
        let mut command_iter = command.split(' ');
        let program = command_iter
            .next()
            .expect("The command should have a program");
        let args: Vec<String> = command_iter.map(|s| s.to_string()).collect();

        let stdout = if log::log_enabled!(log::Level::Debug) {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let stderr = if log::log_enabled!(log::Level::Error) {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let command = Command::new(program)
            .args(args)
            .arg(url)
            .stdout(stdout)
            .stderr(stderr)
            .stdin(Stdio::null())
            .spawn();

        if let Ok(mut c) = command {
            callback(c.wait().map(|_| ()));
        } else {
            callback(command.map(|_| ()))
        }
    });
}
