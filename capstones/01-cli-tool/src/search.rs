use crate::args::Args;
use crate::printer::{Match, Output};
use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone)]
pub struct SearchOpts {
    pub re: Regex,
    pub files_with_matches: bool,
}

impl SearchOpts {
    pub fn from_args(args: &Args) -> Result<Self> {
        let pat = if args.fixed_strings {
            regex::escape(&args.pattern)
        } else {
            args.pattern.clone()
        };
        let re = RegexBuilder::new(&pat)
            .case_insensitive(args.ignore_case)
            .build()?;
        Ok(Self {
            re,
            files_with_matches: args.files_with_matches,
        })
    }
}

pub fn search_path(path: &Path, opts: &SearchOpts) -> Result<Output> {
    let f = fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, f);

    // 簡單偵測 binary：前 8K 含 NUL byte 就跳過
    {
        let buf = reader.fill_buf()?;
        if buf.iter().take(8192).any(|&b| b == 0) {
            return Ok(Output::skipped(path));
        }
    }

    let mut matches = Vec::new();
    let mut line_no: u32 = 0;

    let mut buf = Vec::with_capacity(4096);
    while {
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf)?;
        n > 0
    } {
        line_no += 1;
        // 去掉 trailing \n
        let line = if buf.ends_with(b"\n") { &buf[..buf.len() - 1] } else { &buf[..] };

        if let Some(m) = opts.re.find(line) {
            let display = String::from_utf8_lossy(line).into_owned();
            matches.push(Match {
                line_no,
                start: m.start(),
                end: m.end(),
                content: display,
            });
            if opts.files_with_matches {
                break;
            }
        }
    }
    Ok(Output::file(path, matches, opts.files_with_matches))
}
