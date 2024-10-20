use core::panic;
use image::{
    DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageReader, Rgba, RgbaImage,
};
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
impl Rect {
    fn coord_in_rect(&self, x: u32, y: u32) -> bool {
        return x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height;
    }
}
#[derive(Debug)]
struct FrameData {
    image: DynamicImage,
    offset: (u32, u32),
    frame_time: u32,
    cleanup_rect: Option<Rect>,
}
fn generate_frame(
    image1: &DynamicImage,
    image2: &DynamicImage,
) -> (Rect, DynamicImage, DynamicImage) {
    let mut new_pixel_buffer =
        DynamicImage::ImageRgba8(RgbaImage::new(image1.width(), image1.height()));
    let mut clear_pixel_buffer = new_pixel_buffer.clone();
    let mut same_pixels_buffer = new_pixel_buffer.clone();
    let mut drawn_frame_buffer = new_pixel_buffer.clone();
    let mut save_image_buffer = new_pixel_buffer.clone();
    for ((x, y, value_prev), (.., value_new)) in zip(image1.pixels(), image2.pixels()) {
        if value_prev != value_new && value_new[3] != 0 {
            new_pixel_buffer.put_pixel(x, y, value_new);
        } else if value_prev[3] > 0 && value_new[3] == 0 {
            clear_pixel_buffer.put_pixel(x, y, value_prev);
        } else if value_prev == value_new {
            same_pixels_buffer.put_pixel(x, y, value_new);
        }
    }
    let clean_rect = get_bounding_rect(&clear_pixel_buffer);
    for ((x, y, new_pixel), (.., same_pixel)) in
        zip(new_pixel_buffer.pixels(), same_pixels_buffer.pixels())
    {
        if clean_rect.coord_in_rect(x, y) && new_pixel[3] == 0 {
            drawn_frame_buffer.put_pixel(x, y, same_pixel);
            save_image_buffer.put_pixel(x, y, same_pixel);
        } else if new_pixel[3] == 0 && same_pixel[3] != 0 {
            drawn_frame_buffer.put_pixel(x, y, same_pixel);
        } else {
            drawn_frame_buffer.put_pixel(x, y, new_pixel);
            save_image_buffer.put_pixel(x, y, new_pixel);
        }
    }
    return (clean_rect, drawn_frame_buffer, save_image_buffer);
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
fn process_folder(path: std::path::PathBuf) -> Vec<FrameData> {
    let mut rendered_frames: DynamicImage = DynamicImage::ImageRgba8(RgbaImage::new(0, 0));
    let mut out: Vec<FrameData> = Vec::new();
    for images_entry in fs::read_dir(path).unwrap() {
        let image = DynamicImage::ImageRgba8(
            ImageReader::open(images_entry.unwrap().path())
                .unwrap()
                .decode()
                .unwrap()
                .to_rgba8(),
        );
        if out.len() == 0 {
            rendered_frames = image.clone();
            let rect = get_bounding_rect(&image);
            out.push(FrameData {
                image: image.crop_imm(rect.x, rect.y, rect.width, rect.height),
                offset: (rect.x, rect.y),
                frame_time: 1,
                cleanup_rect: None,
            });
        } else if image == rendered_frames {
            out.last_mut().unwrap().frame_time += 1
        } else {
            let (clear_rect, fr, save_img) = generate_frame(&rendered_frames, &image);
            let rect = get_bounding_rect(&save_img);
            out.push(FrameData {
                image: save_img.crop_imm(rect.x, rect.y, rect.width, rect.height),
                offset: (rect.x, rect.y),
                frame_time: 1,
                cleanup_rect: Some(clear_rect),
            });
            rendered_frames = fr;
        }
    }
    return out;
}
fn pack_animations() {}

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
