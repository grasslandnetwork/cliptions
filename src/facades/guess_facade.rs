use chrono::{DateTime, Utc};
use ndarray::Array1;

use crate::types::Guess;

pub struct GuessFacade<'a> {
    inner: &'a Guess,
}

impl<'a> GuessFacade<'a> {
    pub fn new(inner: &'a Guess) -> Self { Self { inner } }
    pub fn text(&self) -> &str { &self.inner.text }
    pub fn timestamp(&self) -> DateTime<Utc> { self.inner.timestamp }
    pub fn has_embedding(&self) -> bool { self.inner.embedding.is_some() }
    pub fn embedding_array(&self) -> Option<Array1<f64>> { self.inner.get_embedding_array() }
}


