use std::collections::VecDeque;
use std::f64::consts::{E, PI};
use std::fmt::Alignment::Left;
use std::fmt::Debug;
use std::fs::DirEntry;
use std::iter::from_fn;
use std::num::Wrapping;
use std::ops::Range;
use std::path::Path;
use std::slice::SliceIndex;
use std::vec::IntoIter;
use image::{ImageResult, RgbaImage};
use probability::distribution::{Continuous, Gaussian};
use rand::{Rng, thread_rng};
use crate::Filter;
use crate::pipeline::{Generator, Pipeline};
use crate::rgba::Rgba;

#[derive(Clone)]
pub struct Image(Vec<Vec<Rgba>>);

impl std::ops::Index<(usize, usize)> for Image {
    type Output = Rgba;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.0[x][y]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.0[x][y]
    }
}

impl Image {
    fn construct(width: usize, height: usize, f: impl Fn(usize, usize) -> Rgba) -> Image {
        let data = (0..width)
            .map(|x| (0..height)
                .map(|y| f(x, y))
                .collect())
            .collect();
        Image(data)
    }

    fn from_pixel(width: usize, height: usize, pixel: Rgba) -> Image {
        Self::construct(width, height, move |_, _| pixel)
    }

    pub fn empty(width: usize, height: usize) -> Image {
        Self::from_pixel(width, height, Rgba::BLACK)
    }

    pub fn width(&self) -> usize {
        self.0.len()
    }

    pub fn height(&self) -> usize {
        self.0.first()
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn similar(&self, f: impl Fn(usize, usize) -> Rgba) -> Image {
        Image::construct(self.width(), self.height(), f)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> ImageResult<()> {
        Into::<RgbaImage>::into(self.clone())
            .save(path)
    }

    pub fn into_rgba8(self) -> Vec<u8> {
        self.0.into_iter()
            .flatten()
            .flat_map(Into::<[u8; 4]>::into)
            .collect()
    }
}

impl From<RgbaImage> for Image {
    fn from(i: RgbaImage) -> Self {
        Self::construct(i.width() as usize,
                        i.height() as usize,
                        |x, y| i
            .get_pixel(x as u32, y as u32)
            .into())
    }
}

impl Into<RgbaImage> for Image {
    fn into(self) -> RgbaImage {
        RgbaImage::from_fn(
            self.width() as u32,
            self.height() as u32,
            |x, y| {
                self[(x as usize, y as usize)].into()
            })
    }
}

#[derive(Default)]
pub struct CpuPipeline {
    actions: Vec<Box<dyn FnOnce(Image) -> Image>>
}

impl CpuPipeline {
    fn commit(mut self, action: impl FnOnce(Image) -> Image + 'static) -> Self {
        self.actions.push(Box::new(action));
        self
    }

    fn dbg(self, loc: impl AsRef<Path> + 'static) -> Self {
        self.commit(move |image| {
            image.save(loc).unwrap();
            image
        })
    }

    fn convolve(self,
                needle_width: usize,
                needle_height: usize,
                needle: impl Fn(usize, usize) -> Rgba + 'static,
                f: impl Fn(Self, Self) -> Self + 'static) -> Self {
        self.commit(move |image| {
            let out = (0..needle_width)
                .flat_map(|x| (0..needle_height)
                    .map(move |y| (x, y)))
                .map(|(x, y)| {
                    let needle_pixel = needle (x, y);
                    let image = image.clone();
                    CpuPipeline::default()
                        .commit(|_| image)
                        .offset(x as i64 - needle_width as i64 >> 1,
                                y as i64 - needle_height as i64 >> 1)
                        .dim(needle_pixel)
                })
                .fold(CpuPipeline::default(), f);
            out.generate(image.width(), image.height())
        })
    }

    fn convolve_by(self, needle: Image, f: impl Fn(Self, Self) -> Self + 'static) -> Self {
        self.convolve(needle.width(),
                      needle.height(),
                      move |x, y| needle[(x, y)],
                      f
        )
    }
}

pub struct CpuGenerator {
    pub size: usize,
}

impl CpuGenerator {
    pub fn new(size: usize) -> Self {
        CpuGenerator {
            size,
        }
    }
}

impl Generator for CpuGenerator {
    type Pipeline = CpuPipeline;

    fn gaussian_noise(&self, mean: f64, variance: f64, intensity: f64) -> Self::Pipeline {
        let pdf = Gaussian::new(mean, variance);
        CpuPipeline::default()
            .commit(move |image| {
                image.similar(|_, _| {
                    let rand = thread_rng().gen_range(0..255u8) as f64 / 256.0;
                    let res = pdf.density(rand) * intensity;
                    Rgba::gray(0.5 + if thread_rng().gen() {
                        res
                    } else {
                        -res
                    }).into()
                })
            })
    }

    fn salt_and_pepper_noise(&self, variance: f64) -> Self::Pipeline {
        let pdf = Gaussian::new(0.5, variance);
        CpuPipeline::default()
            .commit(move |image| image.similar(|_, _| {
                let rand = thread_rng().gen_range(0..255u8) as f64 / 256.0;
                Rgba::gray(if pdf.density(rand) > 0.6 {
                    if thread_rng().gen() {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.5
                })
            }))
    }

    fn average_needle(&self) -> Filter<Self::Pipeline> {
        let npixels = self.size * self.size;
        let value = 1.0 / npixels as f64;
        let pixel = Rgba::gray(value);
        let size = self.size;
        Filter::Convoluted(CpuPipeline::default().commit(move |_| Image::from_pixel(
            size,
            size,
            pixel.into())))
    }

    fn gaussian_needle(&self, variance: f64) -> Filter<Self::Pipeline> {
        let size = self.size;
        Filter::Convoluted(CpuPipeline::default()
            .commit(move |_| Image::construct(size, size, |i, j| {
                let center = (size >> 1) as i64;
                let i = (i as i64 - center).abs();
                let j = (j as i64 - center).abs();
                let offset = (i * i + j * j) as f64;
                let gauss = Gaussian::new(0f64, variance)
                    .density(offset.sqrt());

                let rgba = Rgba::gray(gauss);
                let out = rgba.into();
                out
            })))
    }
}

impl super::pipeline::Image for Image {
    fn black(width: usize, height: usize) -> Self {
        Image::from_pixel(width, height, Rgba::BLACK.into())
    }
}

fn image_by(op: &'static (dyn Fn(Rgba, Rgba) -> Rgba + 'static)) -> impl Fn(CpuPipeline, CpuPipeline) -> CpuPipeline {
    move |this: CpuPipeline, other: CpuPipeline| {
        this.commit(move |this| {
            let other = other.generate(this.width(), this.height());
            this.similar(|x, y| {
                let this = this[(x, y)];
                let other = other[(x, y)];
                op(this, other)
            })
        })
    }
}

impl Pipeline for CpuPipeline {
    type Image = Image;

    fn filter(self, needle: Filter<Self>) -> Self {
        match needle {
            Filter::Convoluted(n) => {
                self.convolve_by(n.generate(0, 0), Self::add)
            }
            Filter::Median(size) => {
                self.commit(move |image| {
                    let needle = Image::from_pixel(size, size,
                                                   Rgba::WHITE.into());
                    let min = CpuPipeline::default()
                        .convolve_by(needle.clone(), image_by(&Rgba::min))
                        .apply(&image);
                    let max = CpuPipeline::default()
                        .convolve_by(needle, image_by(&Rgba::max))
                        .apply(&image);
                    image.similar(|x, y| {
                        let min = Rgba::from(min[(x, y)]);
                        let max = Rgba::from(max[(x, y)]);

                        (min + max).into()
                    })
                })
            }
        }
    }

    fn offset(self, x: i64, y: i64) -> Self {
        self.commit(move |image| image.similar(|i, j| image[(
                (i as i64 + x).min(image.width()  as i64 - 1).max(0) as usize,
                (j as i64 + y).min(image.height() as i64 - 1).max(0) as usize,
            )].clone()))
    }

    fn add(self, other: Self) -> Self {
        self.commit(move |image| {
            let other = other.apply(&image);
            image.similar(|x, y| {
                let other = Rgba::from(other[(x, y)]);
                let this = Rgba::from(image[(x, y)]);
                (this + other).into()
            })
        })
    }

    fn ennoise(self, noise: Self) -> Self {
        self.commit(move |image| {
            let other = noise.apply(&image);
            image.similar(|x, y| {
                let noise = Rgba::from(other[(x, y)]);
                let this = Rgba::from(image[(x, y)]);

                let noise = (noise - Rgba::gray(0.5)) * Rgba::gray(2.0);

                (this + noise).into()
            })
        })
    }

    fn dim(self, factor: Rgba) -> Self {
        self.commit(move |image| image.similar(|x, y| {
            let this = image[(x, y)];
            let out = this * factor;
            out
        }))
    }

    fn grayscale(self) -> Self {
        self.dim(Rgba::GRAYSCALE_FACTOR)
            .commit(move |image| image.similar(|x, y|
                image[(x, y)].grayscale()
            ))
    }

    fn invert(self) -> Self {
        self.commit(|image| image.similar(|x, y| {
            Rgba::gray(1.0) - image[(x, y)]
        }))
    }

    fn gradient(self) -> Self {
        self.convolve(3, 3,
            |x, y| {
                let out = match (x, y) {
                    (0, 0) | (1, 1) | (2, 2) | (0, 2) | (2, 0) => Rgba::BLACK,
                    (1, 0) | (0, 1) => Rgba::WHITE.map(|x| -x),
                    (1, 2) | (2, 1) => Rgba::WHITE,
                    (x, y) => panic!("Got invalid index (x = {x}, y = {y})")
                };
                out
            },
            image_by(&std::ops::Add::add)
        ).commit(|image| image.similar(|x, y| {
            let pixel = image[(x, y)].map(f64::abs);
            pixel
        }))
    }

    fn apply(self, image: &Self::Image) -> Self::Image {
        self.actions.into_iter()
            .fold(image.clone(), |image, f| f(image))
    }

    fn sub(self, other: Self) -> Self {
        self.commit(move |image| {
            let other = other.apply(&image);
            image.similar(|x, y| {
                let other = Rgba::from(other[(x, y)]);
                let this = Rgba::from(image[(x, y)]);
                (this - other)
                    .with_alpha(this.alpha())
                    .into()
            })
        })
    }

    fn non_max_suppress(self) -> Self {
        self.commit(|mut image| image.similar(|x, y| {
            let suppress = |slice: [(usize, usize); 3]| -> bool {
                let values: [Rgba ;3] = slice.into_iter()
                    .map(|x| image[x])
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                values[0] < values[1] && values[2] < values[1]
            };

            let xp = x.checked_sub(1).unwrap_or(0);
            let xn = (x + 1).min(image.width() - 1);
            let yp = y.checked_sub(1).unwrap_or(0);
            let yn = (y + 1).min(image.height() - 1);

            if  suppress([(xp, yp), (x, y), (xn, yn)]) ||
                suppress([(xp, y ), (x, y), (xn, y )]) ||
                suppress([(x,  yp), (x, y), (x,  yn)]) ||
                suppress([(xp, yn), (x, y), (xn, yp)])
            {
                image[(x, y)]
            } else {
                Rgba::BLACK
            }
        }))
    }

    fn quantize(self, thresholds: Vec<f64>) -> Self {
        let len = thresholds.len();
        let steps = thresholds.into_iter()
            .rev()
            .chain([0.0])
            .enumerate()
            .map(move |(n, threshold)| (Rgba::gray(n as f64 / len as f64), threshold))
            .collect::<Vec<_>>();
        self.commit(move |image| image.similar(|x, y| {
            let pixel: f64 = Into::<[f64; 4]>::into(image[(x, y)])
                .into_iter()
                .rev()
                .skip(1)
                .sum::<f64>() / 3.0;
            let (intensity, _) = steps.iter()
                .filter(|(_, threshold)| &pixel >= threshold)
                .next()
                .unwrap()
                .clone();
            intensity
        }))
    }

    fn gaussian_blur(self, size: usize, variance: f64) -> Self {
        self.filter(CpuGenerator::new(5)
            .gaussian_needle(0.6))
    }

}