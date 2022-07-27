#![allow(dead_code)]

mod consts;
mod rayt;

use image::{Rgb, RgbImage};
use rayt::float3::*;
use rayt::window::*;
use std::{collections::HashMap, fs, path::Path};

const IMAGE_WIDTH: u32 = 200;
const IMAGE_HEIGHT: u32 = 100;

const OUTPUT_FILENAME: &str = "render.png";
const BUCKUP_FILENAME: &str = "render_bak.png";

fn main() {
    backup();

    let mut pixels = HashMap::<(u32, u32), Color>::new();

    for j in 0..IMAGE_HEIGHT {
        for i in 0..IMAGE_WIDTH {
            pixels.insert(
                (i, j),
                Color::new(
                    i as f64 / IMAGE_WIDTH as f64,
                    j as f64 / IMAGE_HEIGHT as f64,
                    0.5,
                ),
            );
        }
    }

    let mut img = RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .iter_mut()
        .for_each(|(x, y, pixel)| {
            let color = pixels.get(&(*x, *y)).unwrap();
            pixel[0] = color.r();
            pixel[1] = color.g();
            pixel[2] = color.b();
        });
    img.save(OUTPUT_FILENAME).unwrap();
    draw_in_window(BUCKUP_FILENAME, img).unwrap();
}

fn backup() {
    let output_path = Path::new(OUTPUT_FILENAME);
    if output_path.exists() {
        println!("backup {:?} -> {:?}", OUTPUT_FILENAME, BUCKUP_FILENAME);
        fs::rename(OUTPUT_FILENAME, BUCKUP_FILENAME).unwrap();
    }
}
