

pub struct Events {
    pub held: Vec<sdl2::keyboard::Keycode>,
    pub pressed: Vec<sdl2::keyboard::Keycode>,
    pub released: Vec<sdl2::keyboard::Keycode>,
}

impl Events {
    pub fn new() -> Self {
        Events {
            held: Vec::new(),
            pressed: Vec::new(),
            released: Vec::new(),
        }
    }

    pub fn update_down(&mut self, key: sdl2::keyboard::Keycode) {
        if !self.pressed.contains(&key) && !self.held.contains(&key) {
            self.pressed.push(key);
        }
    }
    
    pub fn update_up(&mut self, key: sdl2::keyboard::Keycode) {
        self.held.retain(|&k| k != key);
        self.released.push(key);
    }
    
    pub fn update(&mut self) {
        while let Some(key) = self.pressed.pop() {
            self.held.push(key);
        }
        self.released.clear();
    }

    pub fn held_contains(&self, value: &sdl2::keyboard::Keycode) -> bool {
        self.held.contains(value)
    }
}


