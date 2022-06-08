
pub mod pipeline;
pub mod cpu;
pub mod rgba;

extern crate lazy_static;
extern crate rand;
extern crate num_traits;
extern crate probability;
extern crate core;

pub enum Filter<Image> {
    Convoluted(Image),
    Median(usize),
}

trait Map2D {
    type Item;
    type Output;
    fn map2d<T>(&self, p: impl Fn(Self::Item, Self::Item) -> T) -> Self::Output;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
