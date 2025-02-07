use std::{fmt::Display, process::Command, thread};

const DOWNLOAD_MERGE: &str = "[Merger] Merging formats into ";
const DOWNLOAD_DESTINATION: &str = "[download] Destination: ";

pub fn download<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: FnOnce(Result<String, std::io::Error>) + std::marker::Send + 'static + std::marker::Sync,
>(
    url: S,
    callback: F,
) {
    log::debug!("Downloading video with url: {}", url);
    let download_dir = std::env::var("XDG_DOWNLOAD_DIR")
        .unwrap_or("$HOME/Downloads/%(title)s-%(id)s.%(ext)s".to_string());
    let downloader_str =
        std::env::var("DOWNLOADER").unwrap_or(format!("youtube-dl --output {}", download_dir));
    open_with_output(url, downloader_str, move |output| {
        if let Ok(output) = output {
            callback(Ok(output
                .lines()
                .rev()
                .find(|s| s.starts_with(DOWNLOAD_MERGE) || s.starts_with(DOWNLOAD_DESTINATION))
                .map(|s| s.strip_prefix(DOWNLOAD_MERGE).unwrap_or(s))
                .map(|s| s.strip_prefix(DOWNLOAD_DESTINATION).unwrap_or(s))
                .map(|s| s.trim_matches('"').to_owned())
                .unwrap_or_default()))
        } else {
            callback(output)
        }
    });
}

pub fn open_with_output<
    S: 'static + AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr> + std::marker::Send,
    F: FnOnce(Result<String, std::io::Error>) + std::marker::Send + 'static,
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

        let out = Command::new(program).args(args).arg(url).output();

        if let Ok(out) = out {
            callback(Ok(String::from_utf8_lossy(&out.stdout).to_string()));
        } else {
            callback(out.map(|_| "".to_owned()));
        }
    });
}
