use crate::types::Participant;

pub struct ParticipantFacade<'a> {
    inner: &'a Participant,
}

impl<'a> ParticipantFacade<'a> {
    pub fn new(inner: &'a Participant) -> Self { Self { inner } }
    pub fn social_id(&self) -> &str { &self.inner.social_id }
    pub fn username(&self) -> &str { &self.inner.username }
    pub fn is_verified(&self) -> bool { self.inner.verified }
    pub fn wallet(&self) -> &str { &self.inner.wallet }
    pub fn score(&self) -> f64 { self.inner.score }
    pub fn guess_text(&self) -> &str { &self.inner.guess.text }
}


