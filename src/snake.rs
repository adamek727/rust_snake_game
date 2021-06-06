use super::primitives::Pose;
use std::collections::VecDeque;

pub struct Snake {
    body: VecDeque<Pose>,
}

impl Snake {
    pub fn new() -> Snake {
        let init_body:VecDeque<Pose> = vec![Pose{x: 0, y:0}].into();
        Snake {
            body: init_body
        }
    }

    pub fn front(&self) -> Pose {
        self.body.front().unwrap().clone()
    }

    pub fn back(&self) -> Pose {
        self.body.back().unwrap().clone()
    }

    pub fn grow_back(&mut self, cell: &Pose) {
        self.body.push_back(cell.clone());
    }

    pub fn move_up(&mut self) {
        let front = self.body.front().unwrap().clone();
        self.body.push_front(Pose{x: front.x, y: front.y - 1});
        self.body.pop_back();
    }

    pub fn move_down(&mut self) {
        let front = self.body.front().unwrap().clone();
        self.body.push_front(Pose{x: front.x, y: front.y + 1});
        self.body.pop_back();
    }

    pub fn move_left(&mut self) {
        let front = self.body.front().unwrap().clone();
        self.body.push_front(Pose{x: front.x - 1, y: front.y});
        self.body.pop_back();
    }

    pub fn move_right(&mut self) {
        let front = self.body.front().unwrap().clone();
        self.body.push_front(Pose{x: front.x + 1, y: front.y});
        self.body.pop_back();
    }

    pub fn is_part_of_body(&self, pose: &Pose) -> bool {
        for cell in self.body.iter() {
            if *cell == *pose {
                return true
            }
        }
        false
    }

    pub fn is_crashed(&self) -> bool {
        let mut crashed = false;
        let head = self.body.front().unwrap().clone();
        for (i, cell) in self.body.iter().enumerate() {
            if i > 0 {
                if head == *cell {
                    crashed = true;
                    break;
                }
            }
        }
        crashed
    }
}
