#![allow(dead_code)]

mod consts;
mod rayt;

use image::{Rgb, RgbImage};
use rayon::prelude::*;
use rayt::camera::*;
use rayt::float3::*;
use rayt::quat::*;
use rayt::ray::*;
use rayt::window::*;
use std::{fs, path::Path};

const IMAGE_WIDTH: u32 = 200;
const IMAGE_HEIGHT: u32 = 100;

const OUTPUT_FILENAME: &str = "render.png";
const BUCKUP_FILENAME: &str = "render_bak.png";

fn color(ray: &Ray) -> Color {
    let d = ray.direction.normalize();
    let t = 0.5 * (d.y() * 1.0);
    Color::new(0.5, 0.7, 1.0).lerp(Color::one(), t)
}

fn main() {
    backup();

    // TODO quat を use しないと quat のテストでエラーになるため、適当に use している
    Quat::new(0.0, 0.0, 0.0, 0.0);

    let camera = Camera::new(
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(-2.0, -1.0, -1.0),
    );

    let mut img = RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let u = *x as f64 / (IMAGE_WIDTH - 1) as f64;
            let v = *y as f64 / (IMAGE_HEIGHT - 1) as f64;
            let ray = camera.ray(u, v);
            let rgb = color(&ray).to_rgb();

            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
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
