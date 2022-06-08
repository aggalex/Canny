use std::fmt::Display;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, RwLock, Weak};
use std::thread;
use gdk_pixbuf::glib::{Bytes, MainContext};
use gdk_pixbuf::glib::clone::{Downgrade, Upgrade};
use gtk::glib::{Cast, PRIORITY_DEFAULT, WeakRef};
use computer_vision::cpu::{CpuGenerator, CpuPipeline, Image as RgbaImage};
use computer_vision::pipeline::{Generator, Pipeline};
use crate::{AddableAt, Continue, IsA, With};

#[derive(Copy, Clone, Debug)]
pub struct GaussianCoeff {
    pub mean: f64,
    pub variance: f64
}

#[derive(Clone)]
pub struct Image {
    pixbuf: Arc<RwLock<RgbaImage>>,
    stack: gtk::Stack,
}

#[derive(Clone)]
pub struct WeakImage {
    pixbuf: Weak<RwLock<RgbaImage>>,
    stack: WeakRef<gtk::Stack>,
}

impl Image {
    pub fn new() -> Image {
        let this = gtk::Stack::builder()
            .hexpand(true)
            .vexpand(true)
            .transition_type(gtk::StackTransitionType::Crossfade)
            .build()
            .with(|w| {

                gtk::Spinner::builder()
                    .hexpand(true)
                    .vexpand(true)
                    .halign(gtk::Align::Center)
                    .valign(gtk::Align::Center)
                    .build()
                    .put_at(&w, "spinner")
                    .with(|spinner| spinner.start());

                gtk::Picture::builder()
                    .hexpand(true)
                    .vexpand(true)
                    .build()
                    .put_at(&w, "image");

                Image {
                    pixbuf: Arc::new(RwLock::new(RgbaImage::empty(0, 0))),
                    stack: w
                }
            });
        this
    }
    
    pub fn set_new(&self, file: &Path) {
        match image::io::Reader::open(file.clone())
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            .and_then(|img| img.decode()
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            ) {
            Ok(img) => {
                println!("Setting image to {}", file.display());
                *self.pixbuf.write().unwrap() = img.into_rgba8().into();
                self.stack.child_by_name("image")
                    .unwrap()
                    .dynamic_cast::<gtk::Picture>()
                    .unwrap()
                    .set_filename(Some(&file))
            },
            Err(err) => eprintln!("{}", err)
        }
    }

    pub fn as_widget(&self) -> impl IsA<gtk::Widget> {
        self.stack.set_visible_child_name("image");
        self.stack.clone()
    }

    fn calculate(&self, f: impl FnOnce(&RgbaImage) -> RgbaImage + 'static + Send) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        let pixbuf = self.pixbuf.clone();

        self.stack.set_visible_child_name("spinner");

        thread::spawn(move || {
            println!("Calculating");
            let surface = pixbuf.read().unwrap();
            let data = f(&*surface);
            println!("Calculated: {}x{}", data.width(), data.height());
            let dir = std::env::temp_dir().with(|mut dir| {
                dir.push("img.png");
                dir.as_path().to_owned()
            });
            data.save(dir.clone()).unwrap();
            sender.send(
                dir
            ).expect("Could not send through channel");
        });

        let weak_self = self.downgrade();

        receiver.attach(
            None,
            move |new_image| {
                let this = weak_self.upgrade().unwrap();

                this.set_new(&new_image);

                this.stack.set_visible_child_name("image");

                Continue(false)
            }
        );
    }

    pub fn downgrade(&self) -> WeakImage {
        WeakImage {
            pixbuf: Arc::downgrade(&self.pixbuf),
            stack: self.stack.downgrade()
        }
    }

    pub fn gaussian_blur(&self, size: usize) {
        assert_ne!(size % 2, 0);
        println!("Gaussian Blur: {:#?}", size);

        self.calculate(move |surface| CpuPipeline::default()
            .filter(CpuGenerator::new(size)
                .gaussian_needle((size >> 1 + 1) as f64 / 10.0 + 0.1))
            .apply(&surface.clone().into())
            .into());
    }

    pub fn snp_noise(&self, variance: f64) {
        println!("S&P noise: {:#?}", variance);
        self.calculate(move |surface| CpuPipeline::default()
            .add(CpuGenerator::new(surface.width().max(surface.height()) as usize)
                .salt_and_pepper_noise(variance))
            .apply(&surface.clone().into())
            .into())
    }

    pub fn gaussian_noise(&self, variance: f64, intensity: f64) {
        println!("Gaussian Noise: {:#?}", variance);

        self.calculate(move |surface| CpuPipeline::default()
            .add(CpuGenerator::new(surface.width().max(surface.height()) as usize)
                .gaussian_noise(0.5, variance, intensity))
            .apply(&surface.clone().into())
            .into())
    }

    pub fn canny(&self, threshold: Vec<f64>) {
        self.calculate(move |surface| CpuPipeline::default()
            .canny(threshold)
            .apply(&surface.clone().into())
            .into())
    }
    
    pub fn grayscale(&self) {
        self.calculate(move |surface| CpuPipeline::default()
            .grayscale()
            .apply(&surface.clone().into())
            .into())
    }
    
    pub fn gradient(&self) {
        self.calculate(move |surface| CpuPipeline::default()
            .gradient()
            .apply(&surface.clone().into())
            .into())
    }
}

impl WeakImage {
    pub fn upgrade(&self) -> Option<Image> {
        Some(Image {
            pixbuf: self.pixbuf.upgrade()?,
            stack: self.stack.upgrade()?
        })
    }
}