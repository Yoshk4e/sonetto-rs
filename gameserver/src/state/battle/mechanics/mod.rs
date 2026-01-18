pub mod bloodtithe;

use bloodtithe::BloodtitheState;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mechanics {
    pub bloodtithe: BloodtitheState,
}

impl Mechanics {
    pub fn new() -> Self {
        Self {
            bloodtithe: BloodtitheState::new(),
        }
    }
}
