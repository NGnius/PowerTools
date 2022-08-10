use super::StateError;

pub trait OnPoll {
    fn on_poll(&self) -> Result<(), StateError>;
}
