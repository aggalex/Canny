use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Add, Deref, DerefMut};
use std::rc::Rc;
use gtk::glib::{SignalHandlerId};
use gtk::prelude::*;
use gtk::Widget;
use crate::ObjectExt;

pub trait Container {
    fn add(&self, w: &impl IsA<Widget>);
}

pub trait ContainerAt {
    type Position;
    fn add(&self, w: &impl IsA<Widget>, position: Self::Position);
}

impl Container for gtk::Box {
    fn add(&self, w: &impl IsA<Widget>) {
        self.append(w)
    }
}

impl Container for gtk::Expander {
    fn add(&self, w: &impl IsA<Widget>) {
        self.set_child(Some(w))
    }
}

impl Container for gtk::ApplicationWindow {
    fn add(&self, w: &impl IsA<Widget>) {
        self.set_child(Some(w))
    }
}

impl Container for gtk::Window {
    fn add(&self, w: &impl IsA<Widget>) {
        self.set_child(Some(w))
    }
}

impl Container for gtk::Revealer {
    fn add(&self, w: &impl IsA<Widget>) {
        self.set_child(Some(w))
    }
}

impl Container for gtk::ScrolledWindow {
    fn add(&self, w: &impl IsA<Widget>) {
        self.set_child(Some(w))
    }
}

pub enum Side {
    Start,
    End
}

impl ContainerAt for gtk::HeaderBar {
    type Position = Side;

    fn add(&self, w: &impl IsA<Widget>, position: Side) {
        match position {
            Side::Start => self.pack_start(w),
            Side::End => self.pack_end(w)
        }
    }
}

impl ContainerAt for gtk::Stack {
    type Position = &'static str;

    fn add(&self, w: &impl IsA<Widget>, position: Self::Position) {
        self.add_named(w, Some(position));
    }
}

pub struct Title;

impl ContainerAt for gtk::ApplicationWindow {
    type Position = Title;

    fn add(&self, w: &impl IsA<Widget>, position: Self::Position) {
        self.set_titlebar(Some(w));
    }
}

impl ContainerAt for gtk::Grid {
    type Position = (i32, i32, i32, i32);

    fn add(&self, w: &impl IsA<Widget>, position: Self::Position) {
        self.attach(w, position.0, position.1, position.2, position.3)
    }
}

pub trait Addable {
    fn put_in(self, w: &impl Container) -> Self;
}

pub trait AddableAt {
    fn put_at<C: ContainerAt>(self, w: &C, position: C::Position) -> Self;
}

impl<W: IsA<Widget>> Addable for W {
    fn put_in(self, w: &impl Container) -> Self {
        w.add(&self);
        self
    }
}

impl<W: IsA<Widget>> AddableAt for W {
    fn put_at<C: ContainerAt>(self, w: &C, position: C::Position) -> Self {
        w.add(&self, position);
        self
    }
}

pub trait With: Sized {
    fn with<T>(self, f: impl FnOnce(Self) -> T) -> T;
}

impl<A: Clone> With for A {
    fn with<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self.clone())
    }
}

struct DoNothing<Args> (PhantomData<Args>);

impl<Args> FnMut<Args> for DoNothing<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {}
}

impl<Args> FnOnce<Args> for DoNothing<Args> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {}
}

impl<Args> Fn<Args> for DoNothing<Args> {
    extern "rust-call" fn call(&self, _: Args) -> Self::Output {}
}

struct DoAfter<Args: Clone> {
    base: Box<dyn Fn<Args, Output = ()>>,
    next: Box<dyn Fn<Args, Output = ()>>,
    _phantom: PhantomData<Args>
}

impl<Args: Clone> FnMut<Args> for DoAfter<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.base.deref()
            .borrow_mut()
            .call_mut(args.clone());
        self.next.call_mut(args);
    }
}

impl<Args: Clone> FnOnce<Args> for DoAfter<Args> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.base.deref()
            .borrow()
            .call(args.clone());
        self.next.call_once(args);
    }
}

impl<Args: Clone> Fn<Args> for DoAfter<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.base.deref()
            .borrow()
            .call(args.clone());
        self.next.call(args);
    }
}

#[derive(Clone)]
pub struct Event<Args>(Rc<RefCell<Box<dyn Fn<Args, Output = ()>>>>);

impl<Args: Clone + 'static> Event<Args> {
    pub fn new() -> Self {
        Event(Rc::new(RefCell::new(Box::new(
            DoNothing::<Args> (PhantomData)
        ) as Box<dyn Fn<Args, Output = ()>>)))
    }

    pub fn connect(&self, f: impl Fn<Args, Output = ()> + 'static) {
        let mut cell = self.0.deref()
            .borrow_mut();
        let base = std::mem::replace(&mut *cell, Box::new(DoNothing(PhantomData)));
        *cell = Box::new(DoAfter {
            base,
            next: Box::new(f),
            _phantom: PhantomData
        })
    }
}

impl<Args> FnMut<Args> for Event<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        self.0.deref()
            .borrow_mut()
            .deref_mut()
            .call_mut(args)
    }
}

impl<Args> FnOnce<Args> for Event<Args> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        self.0.deref()
            .borrow()
            .call(args)
    }
}

impl<Args> Fn<Args> for Event<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        self.0.deref()
            .borrow()
            .call(args)
    }
}

#[macro_export]
macro_rules! sdyn {
    ($t:path) => (&'static (dyn $t + 'static))
}