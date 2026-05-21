use std::{fs, path::{Path, PathBuf}, time::Duration};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar};

mod style;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path of directory or file to remove
    path: PathBuf,

    /// Print every deleted item in separate line
    #[arg(short, long)]
    verbose: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let p = PathBuf::from(cli.path);
    if p.is_dir() {
        let mpb = MultiProgress::new();
        let spinner = mpb.add(style::themed_spinner()).with_message("Analyzing...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let entry_count = count_dir(&p)? + 1;
        let pb = mpb.add(style::themed_progressbar_no_msg(entry_count as u64));
        full_remove_dir(&p, &pb, &spinner, cli.verbose, &mpb)?;
        pb.finish();
        spinner.finish_with_message("Finished");
    }
    else {
        fs::remove_file(&p)?;
        let pfname = p.file_name();
        match pfname {
            Some(n) => println!("Removed file {}", n.to_string_lossy()),
            None => println!("Removed file")
        }
    }
    Ok(())
}

fn count_dir(path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let readdir = fs::read_dir(path)?;
    let mut count: usize = 0;
    for e in readdir {
        let e = e?;
        if e.file_type()?.is_dir() {
            let n = count_dir(&e.path())?;
            count += n + 1;
            continue;
        }
        count += 1;
    }
    Ok(count)
}

fn full_remove_dir(path: &Path, pb: &ProgressBar, spinner: &ProgressBar, verbose: bool, mpb: &MultiProgress) -> Result<(), Box<dyn std::error::Error>> {
    let readdir = fs::read_dir(path)?;
    let mut dirs: Vec<fs::DirEntry> = Vec::new();
    for e in readdir {
        let e = e?;
        if e.file_type()?.is_dir() {
            full_remove_dir(&e.path(), pb, spinner, verbose, mpb)?;
            dirs.push(e);
            continue;
        }
        spinner.set_message(format!("Removing file {}", e.file_name().to_string_lossy()));
        fs::remove_file(e.path())?;
        pb.inc(1);
        if verbose { mpb.println(format!("Removed file {}", e.file_name().to_string_lossy()))?; }
    }
    let pfname = path.file_name();
    match pfname {
        Some(n) => {
            spinner.set_message(format!("Removing dir {}", n.to_string_lossy()));
            if verbose { mpb.println(format!("Removed dir {}", n.to_string_lossy()))?; }
        }
        None => spinner.set_message("Removing dir")
    }
    fs::remove_dir(path)?;
    pb.inc(1);
    Ok(())
}