use crate::bytes::BytesMut;
use alloc::boxed::Box;
use hashbrown::HashMap;

pub struct Service<'a> {
    callbacks: HashMap<usize, Box<dyn FnMut(&mut BytesMut, &mut BytesMut) + 'a>>,
}

impl<'a> Service<'a> {}

impl<'a> Service<'a> {
    pub fn new() -> Self {
        Self {
            callbacks: Default::default(),
        }
    }

    pub fn register(&mut self, id: usize, callback: impl FnMut(&mut BytesMut, &mut BytesMut) + 'a) -> bool {
        if self.callbacks.contains_key(&id) {
            return false;
        }
        self.callbacks.insert(id, Box::new(callback));
        true
    }

    pub fn execute(&mut self, id: usize, input: &mut BytesMut, output: &mut BytesMut) -> bool {
        self.callbacks
            .get_mut(&id)
            .map(|func| func(input, output))
            .is_some()
    }
}

use core::fmt::{Display, Formatter, Result};

impl<'a> Display for Service<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "The service has {} callbacks registered.",
            self.callbacks.len()
        )?;
        Ok(())
    }
}
