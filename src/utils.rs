use std::{collections::VecDeque, sync::Arc};

use futures::lock::Mutex;

pub fn game_type_to_path(game_type: &String) -> String {
    if game_type == "knockout" {
        String::from("knockout/KnockoutGame.x86_64")
    } else {
        return String::from("");
    }
}

pub struct NumberPool {
    available: VecDeque<u32>,
}

impl NumberPool {
    pub fn new(range: std::ops::Range<u32>) -> Self {
        Self { available: range.collect() }
    }
    pub fn get(&mut self) -> Option<u32> {
        return self.available.pop_front();
    }
    pub fn release(&mut self, num: u32) {
        self.available.push_back(num);
    }
}
pub type SharedNumberPool = Arc<Mutex<NumberPool>>;
