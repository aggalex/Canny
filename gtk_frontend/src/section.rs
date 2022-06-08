use gtk::prelude::*;
use gtk::Widget;
use crate::util::{Addable, AddableAt, Event, With};

pub struct SectionBuilder {
    name: Option<String>,
    scales: Vec<(&'static str, gtk::Scale)>,
    button: gtk::Button,
    expandable: bool,
}

impl SectionBuilder {
    pub fn builder() -> SectionBuilder {
        SectionBuilder {
            name: None,
            scales: vec![],
            button: gtk::Button::new(),
            expandable: false
        }
    }

    pub fn sensitivity_event(self, event: &Event<()>) -> Self {
        let btn = self.button.clone();
        btn.set_sensitive(false);
        event.connect(move || btn.set_sensitive(true));
        self
    }

    pub fn scale(mut self, name: &'static str, range: std::ops::Range<u8>) -> Self {
        self.scales.push((name, gtk::Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .adjustment(&gtk::Adjustment::builder()
                .lower(range.start as f64)
                .upper(range.end as f64)
                .build())
            .build()
        ));
        self
    }

    pub fn label(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self.button.set_label(name);
        self
    }

    /// For now does nothing
    pub fn expandable(mut self, value: bool) -> Self {
        self.expandable = true;
        self
    }

    pub fn connect_clicked(self, event: impl Fn(&[f64]) + 'static) -> Self {
        let scales = self.scales.clone();
        self.button.connect_clicked(move |_| {
            event(&scales.iter()
                .map(|(_, s)| s.value())
                .collect::<Vec<_>>()
            )
        });
        self
    }

    pub fn build(self) -> impl IsA<Widget> {
        let name = self.name.expect("Missing name of section");
        gtk::Expander::builder()
            .label(&name)
            .build()
            .with(|w| {
                let r = gtk::Revealer::builder()
                    .transition_type(gtk::RevealerTransitionType::SlideDown)
                    .build()
                    .put_in(&w)
                    .with(|w| {
                        gtk::Grid::builder()
                            .build()
                            .put_in(&w)
                            .with(|w| {
                                let size = self.scales.len() as i32;
                                for (row, (name, scale)) in self.scales
                                        .into_iter()
                                        .enumerate()
                                {
                                    let row = row as i32;
                                    gtk::Label::builder()
                                        .xalign(1f32)
                                        .label(name)
                                        .justify(gtk::Justification::Right)
                                        .build()
                                        .put_at(&w, (0, row, 1, 1));
                                    scale.put_at(&w, (1, row, 1, 1));
                                }
                                self.button.put_at(&w, (0, size, 2, 1));
                            });
                        w
                    });
                w.connect_expanded_notify(move |e| {
                    r.set_reveal_child(e.is_expanded());
                });
                w
            })
    }
}