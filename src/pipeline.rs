use crate::rgba::Rgba;
use crate::Filter;

pub trait Image {
    fn black(width: usize, height: usize) -> Self;
}

pub trait Pipeline: Sized {
    type Image: Image;
    fn filter(self, needle: Filter<Self>) -> Self;
    fn gaussian_blur(self, size: usize, variance: f64) -> Self;
    fn offset(self, x: i64, y: i64) -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn ennoise(self, noise: Self) -> Self;
    fn dim(self, factor: Rgba) -> Self;
    fn grayscale(self) -> Self;
    fn gradient(self) -> Self;
    fn invert(self) -> Self;
    fn non_max_suppress(self) -> Self;
    fn quantize(self, thresholds: Vec<f64>) -> Self;
    fn canny(self, thresholds: Vec<f64>) -> Self {
        self.grayscale()
            .gaussian_blur(5, 0.6)
            .gradient()
            .non_max_suppress()
            .quantize(thresholds)
    }
    fn apply(self, image: &Self::Image) -> Self::Image;
    fn generate(self, width: usize, height: usize) -> Self::Image {
        self.apply(&Image::black(width, height))
    }
}

pub trait Generator {
    type Pipeline: Pipeline;
    fn gaussian_noise(&self, mean: f64, variance: f64, intensity: f64) -> Self::Pipeline;
    fn salt_and_pepper_noise(&self, variance: f64) -> Self::Pipeline;
    fn average_needle(&self) -> Filter<Self::Pipeline>;
    fn gaussian_needle(&self, variance: f64) -> Filter<Self::Pipeline>;
}