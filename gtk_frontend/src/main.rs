#![feature(unboxed_closures, fn_traits)]

#[macro_use]
mod image;
pub mod util;
mod section;

#[macro_use]
extern crate computer_vision;
extern crate image as img;

use std::collections::HashMap;
use gtk::{Application, FileFilter, Widget};
use gtk::prelude::*;
use util::With;
use crate::image::*;
use crate::section::SectionBuilder;
use crate::util::{Addable, AddableAt, Event, Side, Title};

fn main() {

    let app = Application::builder()
        .application_id("com.github.aggalex.computer-vision")
        .build();

    app.connect_activate(build_ui);

    app.run();
}


fn build_ui(app: &Application) {

    gtk::ApplicationWindow::builder()
        .application(app)
        .title("computer vision")
        .build()
        .with(|w| {
            let image = Image::new();
            let i = image.downgrade();
            let img = image.clone();
            let window = w.clone();

            let load = Event::new();

            let chooser = gtk::FileChooserDialog::builder()
                .title("Open File")
                .transient_for(&window)
                .action(gtk::FileChooserAction::Open)
                .filter(&{
                    let filter = gtk::FileFilter::new();
                    filter.add_suffix("png");
                    filter
                })
                .build()
                .with(|w: gtk::FileChooserDialog| {
                    println!("Showing file chooser");
                    let chooser = w.downgrade();
                    w.add_buttons(&[
                        ("Open", gtk::ResponseType::Accept),
                        ("Cancel", gtk::ResponseType::Cancel)
                    ]);
                    let load = load.clone();
                    w.connect_response(move |d, response| {
                        chooser.upgrade().unwrap().hide();
                        match response {
                            gtk::ResponseType::Accept => {
                                let file = d.file().unwrap();
                                img
                                    .set_new(&file.path().unwrap());
                                load();
                            }
                            _ => {}
                        }
                    });
                    w
                });

            gtk::HeaderBar::builder()
                .build()
                .put_at(&w, Title)
                .with(|w| {
                    gtk::Button::builder()
                        .icon_name("document-open-symbolic")
                        .build()
                        .put_at(&w, Side::Start)
                        .with(|w| {
                            w.connect_clicked(move |_| {
                                println!("Open file");
                                chooser.show();
                            });
                        });
                });

            gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(6)
                .margin_end(12)
                .margin_start(12)
                .margin_bottom(12)
                .margin_top(12)
                .build()
                .put_in(&w)
                .with(|w| {
                    let i = i.clone();

                    gtk::Box::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .spacing(6)
                        .hexpand(false)
                        .width_request(150)
                        .build()
                        .put_in(&w)
                        .with(|w| {

                            SectionBuilder::builder()
                                .label("Gaussian Blur")
                                .scale("size", 0..10)
                                .sensitivity_event(&load)
                                .connect_clicked(i.clone()
                                    .with(|i| move |d: &[f64]| {
                                        let mut d = d[0] as usize;
                                        d += d % 2;
                                        i
                                            .upgrade()
                                            .unwrap()
                                            .gaussian_blur(d)
                                    }))
                                .build()
                                .put_in(&w);

                            gtk::Separator::builder()
                                .orientation(gtk::Orientation::Horizontal)
                                .build()
                                .put_in(&w);


                            SectionBuilder::builder()
                                .label("Gaussian Noise")
                                .scale("variance", 0..10)
                                .scale("intensity", 20..127)
                                .sensitivity_event(&load)
                                .connect_clicked(i.clone()
                                    .with(|i| move |d: &[f64]| i
                                        .upgrade()
                                        .unwrap().
                                        gaussian_noise(10f64 - d[0], d[1])))
                                .build()
                                .put_in(&w);

                            gtk::Separator::builder()
                                .orientation(gtk::Orientation::Horizontal)
                                .build()
                                .put_in(&w);

                            SectionBuilder::builder()
                                .label("Salt & Pepper Noise")
                                .scale("variance", 0..10)
                                .sensitivity_event(&load)
                                .connect_clicked(i.clone()
                                    .with(|i| move |d: &[f64]| i
                                        .upgrade()
                                        .unwrap()
                                        .snp_noise(d[0])))
                                .build()
                                .put_in(&w);

                            gtk::Separator::builder()
                                .orientation(gtk::Orientation::Horizontal)
                                .build()
                                .put_in(&w);

                            SectionBuilder::builder()
                                .label("Grayscale")
                                .sensitivity_event(&load)
                                .connect_clicked(i.clone()
                                    .with(|i| move |_: &[f64]| i
                                        .upgrade()
                                        .unwrap()
                                        .grayscale()))
                                .build()
                                .put_in(&w);

                            gtk::Separator::builder()
                                .orientation(gtk::Orientation::Horizontal)
                                .build()
                                .put_in(&w);

                            SectionBuilder::builder()
                                .label("Gradient")
                                .sensitivity_event(&load)
                                .connect_clicked(i.clone()
                                    .with(|i| move |_: &[f64]| i
                                        .upgrade()
                                        .unwrap()
                                        .gradient()))
                                .build()
                                .put_in(&w);

                            gtk::Separator::builder()
                                .orientation(gtk::Orientation::Horizontal)
                                .build()
                                .put_in(&w);

                            SectionBuilder::builder()
                                .label("Canny")
                                .expandable(true)
                                .scale("threshold 1", 0..1)
                                .scale("threshold 2", 0..1)
                                .scale("threshold 3", 0..1)
                                .sensitivity_event(&load)
                                .connect_clicked(i.clone()
                                    .with(|i| move |d: &[f64]| i
                                        .upgrade()
                                        .unwrap()
                                        .canny(d.to_vec())))
                                .build()
                                .put_in(&w);

                            gtk::Separator::builder()
                                .orientation(gtk::Orientation::Horizontal)
                                .build()
                                .put_in(&w);
                        });

                    gtk::Separator::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .build()
                        .put_in(&w);

                    gtk::ScrolledWindow::builder()
                        .min_content_height(200)
                        .min_content_width(200)
                        .hexpand(true)
                        .vexpand(true)
                        .build()
                        .put_in(&w)
                        .with(|w| {

                            image
                                .as_widget()
                                .put_in(&w);
                        });

                });
            w.show();
        })

}
