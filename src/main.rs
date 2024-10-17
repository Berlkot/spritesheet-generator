use core::panic;
use std::fs;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    folder_path: std::path::PathBuf,
}
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16
}

fn clear_rect_optimize(){}
fn get_bounding_rect(){}

fn main() {
    let args = Cli::parse();
    let folder_path = &args.folder_path;
    if !folder_path.is_dir() {
        panic!("Folder path is not a directory!")
    }
    for entry in fs::read_dir(folder_path).unwrap() {
        let anim_path = entry.unwrap().path();
        println!("Working on {}", anim_path.display());
        if !anim_path.is_dir() {
            println!("Path is not a forder. Skipping...");
            continue;
        }
        
    }

    //println!("{:?}", &character_forder);
}
