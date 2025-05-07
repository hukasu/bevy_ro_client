use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Warnings<W: Warning> {
    warnings: Vec<W>,
}

impl<W: Warning> Default for Warnings<W> {
    fn default() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }
}

impl<W: Warning> Deref for Warnings<W> {
    type Target = Vec<W>;

    fn deref(&self) -> &Self::Target {
        &self.warnings
    }
}

impl<W: Warning> DerefMut for Warnings<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.warnings
    }
}

pub trait Warning: Display {}
