#![allow(dead_code)]

mod consts;
mod rayt;

use std::{fs, fs::File, io::prelude::*, path::Path};

struct Color([f64; 3]);

const IMAGE_WIDTH: u32 = 200;
const IMAGE_HEIGHT: u32 = 100;

const OUTPUT_FILENAME: &str = "render.ppm";
const BUCKUP_FILENAME: &str = "render_bak.ppm";

fn main() {
    backup();

    let mut pixels: Vec<Color> = Vec::with_capacity((IMAGE_WIDTH * IMAGE_HEIGHT) as usize);

    for j in 0..IMAGE_HEIGHT {
        let par_iter = (0..IMAGE_WIDTH).into_iter().map(|i| {
            Color([
                i as f64 / IMAGE_WIDTH as f64,
                j as f64 / IMAGE_HEIGHT as f64,
                0.5,
            ])
        });

        let mut line_pixels: Vec<_> = par_iter.collect();
        pixels.append(&mut line_pixels);
    }

    save_ppm(String::from("render.ppm"), &pixels).unwrap();
}

fn save_ppm(filename: String, pixels: &[Color]) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "P3")?;
    writeln!(file, "{} {}", IMAGE_WIDTH, IMAGE_HEIGHT)?;
    writeln!(file, "255")?;

    for Color([r, g, b]) in pixels {
        let to255 = |x| (x * 255.99) as u8;
        writeln!(file, "{} {} {}", to255(r), to255(g), to255(b))?;
    }
    file.flush()?;

    Ok(())
}

fn backup() {
    let output_path = Path::new(OUTPUT_FILENAME);
    if output_path.exists() {
        println!("backup {:?} -> {:?}", OUTPUT_FILENAME, BUCKUP_FILENAME);
        fs::rename(OUTPUT_FILENAME, BUCKUP_FILENAME).unwrap();
    }
}
