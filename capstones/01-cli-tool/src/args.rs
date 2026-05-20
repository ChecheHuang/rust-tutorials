use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "mini-rg", about = "簡易 ripgrep 實作")]
pub struct Args {
    /// 搜尋的 pattern（regex）
    pub pattern: String,

    /// 路徑（檔案或目錄），預設當前目錄
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// 不區分大小寫
    #[arg(short, long)]
    pub ignore_case: bool,

    /// fixed string（不作為 regex）
    #[arg(short = 'F', long)]
    pub fixed_strings: bool,

    /// 包含隱藏檔
    #[arg(long)]
    pub hidden: bool,

    /// glob 過濾，可多次：-g "*.rs" -g "!target/*"
    #[arg(short = 'g', long = "glob")]
    pub globs: Vec<String>,

    /// thread 數，預設 = 邏輯核心
    #[arg(short = 'j', long)]
    pub threads: Option<usize>,

    /// 強制關閉顏色
    #[arg(long)]
    pub no_color: bool,

    /// 只列檔名（有 match）
    #[arg(short = 'l', long = "files-with-matches")]
    pub files_with_matches: bool,
}
