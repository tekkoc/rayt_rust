use crate::rayt::camera::*;
use crate::rayt::float3::*;
use crate::rayt::ray::*;
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::{fs, path::Path};

const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = 200;

const OUTPUT_FILENAME: &str = "render.png";
const BUCKUP_FILENAME: &str = "render_bak.png";

fn backup() {
    let output_path = Path::new(OUTPUT_FILENAME);
    if output_path.exists() {
        println!("backup {:?} -> {:?}", OUTPUT_FILENAME, BUCKUP_FILENAME);
        fs::rename(OUTPUT_FILENAME, BUCKUP_FILENAME).unwrap();
    }
}

pub trait Scene {
    fn camera(&self) -> Camera;
    fn trace(&self, ray: Ray) -> Color;
    fn width(&self) -> u32 {
        IMAGE_WIDTH
    }
    fn height(&self) -> u32 {
        IMAGE_HEIGHT
    }
    fn aspect(&self) -> f64 {
        self.width() as f64 / self.height() as f64
    }
}

pub fn render(scene: impl Scene + Sync) {
    // scene は複数スレッドから参照されるため、Syncマーカートレイトが必要

    backup();

    let camera = scene.camera();

    let mut img = RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    img.enumerate_pixels_mut()
        .collect::<Vec<(u32, u32, &mut Rgb<u8>)>>()
        .par_iter_mut()
        .for_each(|(x, y, pixel)| {
            let u = *x as f64 / (IMAGE_WIDTH - 1) as f64;
            let v = (IMAGE_HEIGHT - *y - 1) as f64 / (IMAGE_HEIGHT - 1) as f64;
            let ray = camera.ray(u, v);
            let rgb = scene.trace(ray).to_rgb();

            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
        });
    img.save(OUTPUT_FILENAME).unwrap();
}