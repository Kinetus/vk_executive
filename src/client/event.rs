#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Event {
    DoneWork,
    GotWork,
}