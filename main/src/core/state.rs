pub struct State {
    pub session_created: bool
}

impl State {
    pub fn new() -> Self {
        Self {
            session_created: false
        }
    }
}