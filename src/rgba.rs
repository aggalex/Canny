use num_traits::One;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Rgba {
    r: f64,
    b: f64,
    g: f64,
    a: f64,
}

impl Rgba {
    pub const BLACK: Rgba = Rgba {
        r: 0.0,
        b: 0.0,
        g: 0.0,
        a: 1.0
    };

    pub const RED: Rgba = Rgba {
        r: 1.0,
        b: 0.0,
        g: 0.0,
        a: 1.0
    };

    pub const VIOLET: Rgba = Rgba {
        r: 1.0,
        b: 1.0,
        g: 0.0,
        a: 1.0
    };

    pub const BLUE: Rgba = Rgba {
        r: 0.0,
        b: 1.0,
        g: 0.0,
        a: 1.0
    };

    pub const CYAN: Rgba = Rgba {
        r: 0.0,
        b: 1.0,
        g: 1.0,
        a: 1.0
    };

    pub const GREEN: Rgba = Rgba {
        r: 0.0,
        b: 0.0,
        g: 1.0,
        a: 1.0
    };

    pub const YELLOW: Rgba = Rgba {
        r: 1.0,
        b: 0.0,
        g: 1.0,
        a: 1.0
    };

    pub const WHITE: Rgba = Rgba {
        r: 1.0,
        b: 1.0,
        g: 1.0,
        a: 1.0
    };

    pub const COLOURS: [Rgba; 7] = [
        Self::BLACK,
        Self::RED,
        Self::VIOLET,
        Self::BLUE,
        Self::CYAN,
        Self::GREEN,
        Self::YELLOW,
    ];

    pub const GRAYSCALE_FACTOR: Rgba = Rgba {
        r: 0.3,
        g: 0.59,
        b: 0.11,
        a: 1.0
    };

    pub fn gray(value: f64) -> Self {
        Self {
            r: value,
            b: value,
            g: value,
            a: 1.0
        }
    }
}

impl Rgba {
    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.a = alpha;
        self
    }

    pub fn a(&self) -> &f64 {
        &self.a
    }
}

impl Rgba {
    pub fn map(self, f: impl Fn(f64) -> f64) -> Rgba {
        Rgba {
            r: f(self.r),
            b: f(self.b),
            g: f(self.g),
            a: f(self.a)
        }
    }

    pub fn min(self, other: Self) -> Self {
        self.into_iter()
            .zip(other)
            .map(|(a, b)| a.min(b))
            .collect()
    }

    pub fn max(self, other: Self) -> Self {
        self.into_iter()
            .zip(other)
            .map(|(a, b)| a.max(b))
            .collect()
    }

    pub fn grayscale(self) -> Self {
        let Rgba {r, g, b, a} = self * Self::GRAYSCALE_FACTOR;
        Rgba::gray((r + g + b) / 3.0)
            .with_alpha(a)
    }

    pub fn alpha(&self) -> f64 {
        self.a
    }
}

impl std::ops::Mul for Rgba {
    type Output = Rgba;

    fn mul(self, rhs: Self) -> Self::Output {
        self.into_iter()
            .zip(rhs)
            .map(|(a, b)| a * b)
            .collect()
    }
}

impl std::ops::Add for Rgba {
    type Output = Rgba;

    fn add(self, rhs: Self) -> Self::Output {
        let out = self.into_iter()
            .zip(rhs)
            .map(|(a, b)| a + b)
            .collect::<Rgba>();
        out
    }
}

impl std::ops::Sub for Rgba {
    type Output = Rgba;

    fn sub(self, rhs: Self) -> Self::Output {
        self.into_iter()
            .zip(rhs)
            .map(|(a, b)| a - b)
            .collect()
    }
}

impl std::ops::Div<f64> for Rgba {
    type Output = Rgba;

    fn div(self, rhs: f64) -> Self::Output {
        self.into_iter()
            .map(|a| a / rhs)
            .collect()
    }
}

impl From<(f64, f64, f64, f64)> for Rgba {
    fn from((r, g, b, a): (f64, f64, f64, f64)) -> Self {
        Rgba {
            r,
            b,
            g,
            a
        }
    }
}

impl From<&[u8; 4]> for Rgba {
    fn from(slice: &[u8; 4]) -> Self {
        let [r, g, b, a] = slice.clone()
            .map(|a| a as f64 / 256.0);
        Self { r, g, b, a }
    }
}

impl Into<[u8; 4]> for Rgba {
    fn into(self) -> [u8; 4] {
        Into::<image::Rgba<u8>>::into(self).0
    }
}

impl Into<[f64; 4]> for Rgba {
    fn into(self) -> [f64; 4] {
        [
            self.r,
            self.g,
            self.b,
            self.a
        ]
    }
}

impl Into<i32> for Rgba {
    fn into(self) -> i32 {
        let data: [f64; 4] = self.into();
        let [r, g, b, a] = data.map(|a| (a * 256.0) as i32);
        r << 3 + g << 2 + b << 1 + a
    }
}

impl Into<image::Rgba<u8>> for Rgba {
    fn into(self) -> image::Rgba<u8> {
        image::Rgba(self.into_iter()
            .map(|a| a.min(1.0).max(0.0))
            .map(|a| a * 256.0)
            .map(|a| a as u8)
            .take(4)
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap())
    }
}

impl From<&image::Rgba<u8>> for Rgba {
    fn from(pixel: &image::Rgba<u8>) -> Self {
        (&pixel.0).into()
    }
}

impl IntoIterator for Rgba {
    type Item = f64;
    type IntoIter = <[f64; 4] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Into::<[f64; 4]>::into(self).into_iter()
    }
}

impl<'a> IntoIterator for &'a Rgba {
    type Item = f64;
    type IntoIter = <[f64; 4] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.clone().into_iter()
    }
}

impl FromIterator<f64> for Rgba {
    fn from_iter<T: IntoIterator<Item=f64>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        Rgba {
            r: iter.next().unwrap(),
            g: iter.next().unwrap(),
            b: iter.next().unwrap(),
            a: iter.next().unwrap()
        }
    }
}