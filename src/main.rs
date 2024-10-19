use core::panic;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageReader, Rgba, RgbaImage};
use std::{fs, iter::zip};

use clap::Parser;

#[derive(Parser)]
struct Cli {
    folder_path: std::path::PathBuf,
}
#[derive(Debug)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
#[derive(Debug)]
struct FrameData {
    image: DynamicImage,
    offset: (u32, u32),
    frame_time: u32,
    cleanup_rect: Rect,
}

fn is_equal(image1: &DynamicImage, image2: &DynamicImage) -> bool {
    for ((.., value1), (.., value2)) in zip(image1.pixels(), image2.pixels()) {
        if value1 != value2 {
            return false;
        }
    }
    return true;
}

fn get_bounding_rect(image: &DynamicImage) -> Rect {
    let mut out_rect: Rect = Rect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
    };
    // left
    'outer: for x in 0..image.width() {
        for y in 0..image.height() {
            if image.get_pixel(x, y)[3] != 0 {
                out_rect.x = x;
                break 'outer;
            }
        }
    }
    // top
    'outer: for y in 0..image.height() {
        for x in 0..image.width() {
            if image.get_pixel(x, y)[3] != 0 {
                out_rect.y = y;
                break 'outer;
            }
        }
    }
    // right
    'outer: for x in (0..image.width()).rev() {
        for y in 0..image.height() {
            if image.get_pixel(x, y)[3] != 0 {
                out_rect.width = x + 1 - out_rect.x;
                break 'outer;
            }
        }
    }
    // bottom
    'outer: for y in (0..image.height()).rev() {
        for x in 0..image.width() {
            if image.get_pixel(x, y)[3] != 0 {
                out_rect.height = y + 1 - out_rect.y;
                break 'outer;
            }
        }
    }
    return out_rect;
}
fn process_folder(path: std::path::PathBuf) {
    let mut out: Vec<DynamicImage> = Vec::new();
    for images_entry in fs::read_dir(path).unwrap() {
        let image = DynamicImage::ImageRgba8(
            ImageReader::open(images_entry.unwrap().path())
                .unwrap()
                .decode()
                .unwrap()
                .to_rgba8(),
        );
        if out.len() > 0 {
            is_equal(&out[out.len()], &image);
        }
        out.push(image);
    }
    for mut image in out {
        let rect = get_bounding_rect(&image);
        image = image.crop_imm(rect.x, rect.y, rect.width, rect.height);
    }
}

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
        let out = process_folder(anim_path);
    }

    //println!("{:?}", &character_forder);
}
