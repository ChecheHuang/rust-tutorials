use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Match {
    pub line_no: u32,
    pub start: usize,
    pub end: usize,
    pub content: String,
}

pub enum Output {
    File { path: PathBuf, matches: Vec<Match>, files_only: bool },
    Skipped(PathBuf),
    Error { path: PathBuf, msg: String },
}

impl Output {
    pub fn file(path: &Path, matches: Vec<Match>, files_only: bool) -> Self {
        Output::File { path: path.to_path_buf(), matches, files_only }
    }
    pub fn skipped(path: &Path) -> Self { Output::Skipped(path.to_path_buf()) }
    pub fn error(path: PathBuf, msg: String) -> Self { Output::Error { path, msg } }
}

pub struct Printer {
    out: Mutex<StandardStream>,
}

impl Printer {
    pub fn new(no_color: bool) -> Self {
        let choice = if no_color || !atty::is(atty::Stream::Stdout) {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };
        Self { out: Mutex::new(StandardStream::stdout(choice)) }
    }

    pub fn print(&self, output: &Output) {
        let mut out = self.out.lock().unwrap();
        match output {
            Output::File { path, matches, files_only } => {
                if matches.is_empty() { return; }
                if *files_only {
                    let _ = out.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
                    let _ = writeln!(out, "{}", path.display());
                    let _ = out.reset();
                    return;
                }
                // 標題：紫色檔名
                let _ = out.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)));
                let _ = writeln!(out, "{}", path.display());
                let _ = out.reset();

                for m in matches {
                    // 行號（綠）
                    let _ = out.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
                    let _ = write!(out, "{}:", m.line_no);
                    let _ = out.reset();

                    let (pre, mid_and_rest) = m.content.split_at(m.start.min(m.content.len()));
                    let mid_end = (m.end - m.start).min(mid_and_rest.len());
                    let (mid, rest) = mid_and_rest.split_at(mid_end);

                    let _ = write!(out, "{pre}");
                    let _ = out.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true));
                    let _ = write!(out, "{mid}");
                    let _ = out.reset();
                    let _ = writeln!(out, "{rest}");
                }
                let _ = writeln!(out);
            }
            Output::Skipped(_) => { /* 安靜跳過 binary */ }
            Output::Error { path, msg } => {
                eprintln!("mini-rg: {}: {msg}", path.display());
            }
        }
    }
}
