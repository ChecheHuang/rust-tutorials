use crate::args::Args;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::path::PathBuf;

pub fn walk<F: FnMut(PathBuf)>(args: &Args, mut sink: F) {
    let mut paths = args.paths.iter();
    let first = paths.next().expect("at least one path");
    let mut builder = WalkBuilder::new(first);
    for p in paths {
        builder.add(p);
    }

    builder
        .hidden(!args.hidden)
        .git_ignore(true)
        .git_exclude(true);

    if !args.globs.is_empty() {
        let mut ov = OverrideBuilder::new(first);
        for g in &args.globs {
            // 忽略 add 錯誤 — 讓 main 印給 user
            let _ = ov.add(g);
        }
        if let Ok(o) = ov.build() {
            builder.overrides(o);
        }
    }

    let walker = builder.build();
    for entry in walker.flatten() {
        let p = entry.path();
        if p.is_file() {
            sink(p.to_path_buf());
        }
    }
}
