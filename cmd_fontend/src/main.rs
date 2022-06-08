extern crate computer_vision;
extern crate image;
extern crate core;

use image::load;
use computer_vision::cpu::{CpuGenerator, CpuPipeline, Image};
use computer_vision::Filter;
use computer_vision::pipeline::{Generator, Pipeline};

trait ParseArgs {
    fn parse(self, s: String, img: &Image) -> Self;
}

impl ParseArgs for CpuPipeline {
    fn parse(self, s: String, img: &Image) -> Self {
        let mut opt = s.split("=");
        let command = opt.next().unwrap();
        match command {

            "--gaussian-blur" => {
                let size: usize = opt.next()
                    .expect("Expected size of blur")
                    .parse()
                    .expect("Invalid gaussian blur size");
                let size = size + size % 2;
                self.filter(CpuGenerator::new(size)
                    .gaussian_needle(size as f64 / 10.0 + 0.1))
            },

            "--average-blur" => {
                let size: usize = opt.next()
                    .expect("Expected size of blur")
                    .parse()
                    .expect("Invalid gaussian blur size");
                let size = size + size % 2;
                self.filter(CpuGenerator::new(size)
                    .average_needle())
            },

            "--median" => {
                let size = opt.next()
                    .expect("Expected size of blur")
                    .parse()
                    .expect("Invalid gaussian blur size");
                self.filter(Filter::Median(size))
            },

            "--gaussian-noise" => {
                let variance: f64 = opt.next()
                    .expect("Expected variance of noise")
                    .parse()
                    .expect("Invalid noise variance");
                self.ennoise(CpuGenerator::new(img.width()
                        .max(img.height()))
                        .gaussian_noise(0.5, 1.0 / variance, 0.7))
            },

            "--impulse-noise" => {
                let variance = opt.next()
                    .expect("Expected variance of noise")
                    .parse()
                    .expect("Invalid noise variance");
                self.ennoise(CpuGenerator::new(img.width()
                        .max(img.height()))
                        .salt_and_pepper_noise(variance))
            },

            "--canny" => {
                let threshold: Vec<f64> = opt.next()
                    .unwrap_or("0.0")
                    .split(",")
                    .map(|x| x.parse().expect(&format!("Invalid threshold {x}")))
                    .collect();
                self.canny(threshold)
            },

            "--grayscale" => self.grayscale(),
            "--gradient" => self.gradient(),

            unknown => panic!("Unexpected option '{}'", unknown)
        }
    }
}

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();

    let src_uri = args.next()
        .expect("Expected source image");

    let dest_uri = args.next()
        .expect("Expected destination image");

    println!("Loading image {}", src_uri);

    let surface = image::io::Reader::open(&src_uri)
        .expect(&format!("Unable to load image '{}'", src_uri))
        .decode()
        .unwrap()
        .into_rgba8()
        .into();

    let pipeline = args.fold(
        CpuPipeline::default(),
        |pipeline, action| pipeline.parse(action, &surface)
    );

    println!("Calculating");
    let data = pipeline.apply(&surface);
    println!("Calculated: {}x{}", data.width(), data.height());

    let dir = std::env::current_dir().map(|mut dir| {
        dir.push(dest_uri);
        dir.as_path().to_owned()
    })
        .expect("Unable to open directory");

    data.save(dir.clone()).unwrap();
}
