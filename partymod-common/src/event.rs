pub struct EventManager {
    handlers: Vec<fn(&sdl3::event::Event)>,
}

impl EventManager {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn register_handler(&mut self, handler: fn(&sdl3::event::Event)) {
        self.handlers.push(handler);
    }

    pub fn process_events(&self, event_pump: &mut sdl3::EventPump) {
        for event in event_pump.poll_iter() {
            for handler in &self.handlers {
                handler(&event);
            }
        }
    }
}
