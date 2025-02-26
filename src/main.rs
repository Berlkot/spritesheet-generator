use clap::Parser;
use core::panic;
use std::fs::DirEntry;
use image::{DynamicImage, GenericImage, GenericImageView, ImageReader, RgbaImage};
use itertools::Itertools;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
use std::usize;
use std::{fs, iter::zip};
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};

#[derive(Parser)]
struct Cli {
    folder_path: std::path::PathBuf,
    #[arg(default_value = "./")]
    output_path: std::path::PathBuf,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"x\":{},\"y\":{},\"width\":{},\"height\":{}}}",
            self.x, self.y, self.width, self.height
        )
    }
}
#[derive(Debug)]
struct FrameData {
    image: DynamicImage,
    offset: (u32, u32),
    frame_time: u32,
    cleanup_rect: Option<Rect>,
}
#[derive(Debug)]
struct EncodedFrameData {
    location: Rect,
    offset: (u32, u32),
    frame_time: u32,
    cleanup_rect: Option<Rect>,
}
impl Display for EncodedFrameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = self.offset;
        let ster = match self.cleanup_rect {
            Some(rect) => rect.to_string(),
            None => "null".to_string(),
        };
        write!(f, "{{\"location\":{},\"position\":{{\"x\":{},\"y\":{}}},\"duration\":{},\"clear_rect\":{}}}", self.location, x, y, self.frame_time, ster)
    }
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
        width: 0,
        height: 0,
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
    let mut entries: Vec<DirEntry> = fs::read_dir(path).unwrap().map(|r| r.unwrap()).collect();
    entries.sort_by_key(|dir| dir.path());
    for images_entry in entries {
        let image = DynamicImage::ImageRgba8(
            ImageReader::open(images_entry.path())
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
                cleanup_rect: if clear_rect
                    == (Rect {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    }) {
                    None
                } else {
                    Some(clear_rect)
                },
            });
            rendered_frames = fr;
        }
    }
    return out;
}

fn pack_animations(
    animations: HashMap<String, Vec<FrameData>>,
) -> (DynamicImage, HashMap<String, Vec<EncodedFrameData>>) {
    let mut unique_images: Vec<(usize, &String)> = Vec::new();
    let mut skipped_frames: Vec<(usize, usize, &String)> = Vec::new();
    let mut out: HashMap<String, Vec<EncodedFrameData>> = HashMap::new();
    let mut dimensions: u32 = 0;
    for (animation_name, frames) in animations.iter() {
        'outer: for (num, frame) in frames.iter().enumerate() {
            for (index, (i, r)) in unique_images.iter().enumerate() {
                if frame.image == animations.get_key_value(*r).unwrap().1[*i].image {
                    skipped_frames.push((num, index, animation_name));
                    continue 'outer;
                }
            }
            let (x, y) = frame.image.dimensions();
            dimensions += x * y + x * y;
            unique_images.push((num, animation_name));
        }
    }
    let side = f64::from(dimensions).sqrt().round() as u32;
    let config = TexturePackerConfig {
        allow_rotation: false,
        //texture_outlines: true,
        max_width: side,
        max_height: side,
        texture_padding: 0,
        force_max_dimensions: false,
        trim: false,
        ..Default::default()
    };
    let mut packer = TexturePacker::new_skyline(config);
    for (i, (fr_num, a_name)) in unique_images.iter().enumerate() {
        // here we killing image
        packer
            .pack_ref(
                format!("{:0>8}", i),
                animations.get_key_value(*a_name).unwrap().1[*fr_num]
                    .image
                    .borrow(),
            )
            .unwrap();
    }
    let mut unique_images_positions = Vec::new();
    let frames = packer.get_frames();
    for name in frames.keys().sorted() {
        let b_rect = frames[name].frame;
        unique_images_positions.push(Rect {
            x: b_rect.x,
            y: b_rect.y,
            width: b_rect.w,
            height: b_rect.h,
        });
    }
    for (animation_name, frames) in animations.iter() {
        out.insert(animation_name.clone(), Vec::new());
        out.get_mut(animation_name)
            .unwrap()
            .resize_with(frames.len(), || EncodedFrameData {
                location: Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                offset: (0, 0),
                frame_time: 0,
                cleanup_rect: Some(Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                }),
            });
        for (num, frame) in frames.iter().enumerate() {
            for i in skipped_frames.iter() {
                if i.0 == num && i.2 == animation_name {
                    out.get_mut(animation_name).unwrap()[num] = EncodedFrameData {
                        location: unique_images_positions[i.1],
                        offset: frame.offset,
                        frame_time: frame.frame_time,
                        cleanup_rect: frame.cleanup_rect,
                    };
                }
            }
            for (index, i) in unique_images.iter().enumerate() {
                if i.0 == num && i.1 == animation_name {
                    out.get_mut(animation_name).unwrap()[num] = EncodedFrameData {
                        location: unique_images_positions[index],
                        offset: frame.offset,
                        frame_time: frame.frame_time,
                        cleanup_rect: frame.cleanup_rect,
                    };
                }
            }
        }
    }

    return (ImageExporter::export(&packer, None).unwrap(), out);
}

fn main() {
    let args = Cli::parse();
    let folder_path = &args.folder_path;
    let output_path = &args.output_path;
    if !folder_path.is_dir() {
        panic!("Folder path is not a directory!")
    }
    let mut animations: HashMap<String, Vec<FrameData>> = HashMap::new();
    let mut animations_sizes: HashMap<String, (u32, u32)> = HashMap::new();
    for entry in fs::read_dir(folder_path).unwrap() {
        let anim_path = entry.unwrap().path();

        println!("Working on {}", anim_path.display());
        if !anim_path.is_dir() {
            println!("Path is not a forder. Skipping...");
            continue;
        }
        animations.insert(
            anim_path
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned(),
            process_folder(anim_path.clone()),
        );
        let image_saple = ImageReader::open(
            fs::read_dir(&anim_path)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path(),
        )
        .unwrap()
        .decode()
        .unwrap();
        animations_sizes.insert(
            anim_path
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned(),
            (image_saple.width(), image_saple.height()),
        );
    }
    println!("Packing animations...");
    let (output_image, frame_data) = pack_animations(animations);
    println!("Writing animations...");
    output_image
        .save(format!("{}output.png", output_path.to_str().unwrap()))
        .unwrap();
    println!("Writing metadata...");
    let mut file =
        fs::File::create(format!("{}metadata.json", output_path.to_str().unwrap())).unwrap();
    file.write(b"{\"animations\":{").unwrap();
    let mut c = false;
    for (anim_name, data) in frame_data {
        if c {
            file.write(b",").unwrap();
        }
        file.write(format!("\"{}\":{{\"frames\":[", anim_name).as_bytes())
            .unwrap();
        let mut c2 = false;
        for i in data {
            if c2 {
                file.write(b",").unwrap();
            }
            file.write(format!("{}", i).as_bytes()).unwrap();
            c2 = true
        }
        file.write(
            format!(
                "],\"frame_rate\":24,\"width\":{},\"height\":{}}}}}",
                animations_sizes[&anim_name].0, animations_sizes[&anim_name].1
            )
            .as_bytes(),
        )
        .unwrap();
        c = true
    }
    file.write(b"}").unwrap();
}
