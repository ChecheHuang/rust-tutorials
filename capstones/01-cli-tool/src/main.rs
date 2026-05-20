// mini-rg：一個簡化版 ripgrep。
//
// 用到的章節技術：
//   ch04 closure | ch10 ? | ch15 iterators | ch20 module | ch21 testing
//   ch26-30 async（這裡走 sync threading）| ch04 fn pointer
//
// 範例用法：
//   mini-rg "fn main" .
//   mini-rg -i "TODO|FIXME" src/
//   mini-rg --hidden -g "*.rs" "unsafe" .

mod args;
mod printer;
mod search;
mod walker;

use anyhow::Result;
use args::Args;
use clap::Parser;
use crossbeam_channel as channel;
use printer::Printer;
use search::SearchOpts;
use std::sync::Arc;
use std::thread;

fn main() -> Result<()> {
    let args = Args::parse();
    let opts = SearchOpts::from_args(&args)?;

    let printer = Arc::new(Printer::new(args.no_color));
    let (job_tx, job_rx) = channel::bounded::<std::path::PathBuf>(1024);
    let (out_tx, out_rx) = channel::bounded::<printer::Output>(1024);

    // workers
    let n = args.threads.unwrap_or_else(num_cpus::get).max(1);
    let mut workers = Vec::with_capacity(n);
    for _ in 0..n {
        let job_rx = job_rx.clone();
        let out_tx = out_tx.clone();
        let opts = opts.clone();
        workers.push(thread::spawn(move || {
            while let Ok(path) = job_rx.recv() {
                match search::search_path(&path, &opts) {
                    Ok(out) => { let _ = out_tx.send(out); }
                    Err(e) => { let _ = out_tx.send(printer::Output::error(path, e.to_string())); }
                }
            }
        }));
    }
    drop(out_tx); // 等 worker 全部 drop 才會關 out_rx

    // 印 thread
    let printer_clone = printer.clone();
    let printer_thread = thread::spawn(move || {
        for out in out_rx {
            printer_clone.print(&out);
        }
    });

    // 主執行緒 walk
    walker::walk(&args, |p| {
        let _ = job_tx.send(p);
    });
    drop(job_tx);

    for w in workers { let _ = w.join(); }
    let _ = printer_thread.join();
    Ok(())
}
