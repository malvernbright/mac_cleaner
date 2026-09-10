use dirs::home_dir;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    let home = home_dir().expect("Could not locate home directory");

    let targets = vec![
        home.join("Library/Caches"),
        home.join("Library/Logs"),
        home.join(".Trash"),
    ];

    // Set up an animated spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} Scanning macOS clutter... ({elapsed})")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut total_bytes = 0;
    let mut target_sizes = Vec::new();

    for target in &targets {
        if target.exists() {
            let size = calculate_size_parallel(target);
            total_bytes += size;
            target_sizes.push((target, size));
        }
    }

    // Stop and clear the spinner once scanning is complete
    pb.finish_and_clear();

    for (target, size) in &target_sizes {
        println!("- {:?}: {}", target.file_name().unwrap(), format_size(*size));
    }

    if total_bytes == 0 {
        println!("✨ Your system is already clean!");
        return;
    }

    println!("\n🗑️  Total space to reclaim: {}", format_size(total_bytes));
    
    if confirm_deletion() {
        for (target, _) in &target_sizes {
            clean_contents(target);
        }
        println!("✅ Cleanup complete! Reclaimed {}.", format_size(total_bytes));
    } else {
        println!("🛑 Aborted. No files were deleted.");
    }
}

/// Recursively calculates directory size using Rayon for multi-threading
fn calculate_size_parallel(path: &Path) -> u64 {
    let entries: Vec<_> = match fs::read_dir(path) {
        Ok(rd) => rd.flatten().collect(),
        Err(_) => return 0,
    };

    entries
        .par_iter()
        .map(|entry| {
            let entry_path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    calculate_size_parallel(&entry_path)
                } else {
                    metadata.len()
                }
            } else {
                0
            }
        })
        .sum()
}

fn clean_contents(path: &PathBuf) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let _ = fs::remove_dir_all(&entry_path);
            } else {
                let _ = fs::remove_file(&entry_path);
            }
        }
    }
}

fn format_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1_048_576.0;
    if mb > 1024.0 {
        let gb = mb / 1024.0;
        format!("{:.2} GB", gb)
    } else {
        format!("{:.2} MB", mb)
    }
}

fn confirm_deletion() -> bool {
    print!("Do you want to delete these files? (y/N): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let choice = input.trim().to_lowercase();
    choice == "y" || choice == "yes"
}