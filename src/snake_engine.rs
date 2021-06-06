extern crate timer;
extern crate chrono;

use std::sync::{mpsc, Arc, Mutex};
use super::keyboard_handler::KeyEvent;
use super::snake::Snake;
use super::primitives::Pose;

use termion::raw::IntoRawMode;
use std::io::{Write, stdout};
use std::ops::Deref;
use self::timer::{Guard, Timer};

use std::collections::HashSet;

use rand::Rng;

pub fn run_game(key_event_rx: mpsc::Receiver<KeyEvent>) {
    let snake_engine = SnakeEngine::new(key_event_rx);
    snake_engine.run();
}

#[derive(Copy, Clone)]
struct MapSize {
    pub height: usize,
    pub width: usize,
}


#[derive(Copy, Clone, PartialEq)]
enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
    NotMoving,
}

enum SimulationEvent {
    SnakeCrashed,
}

struct SnakeEngine {
    snake: Arc<Mutex<Snake>>,
    map_size: MapSize,
    key_event_rx: mpsc::Receiver<KeyEvent>,
    snake_dir: Arc<Mutex<SnakeDirection>>,
    food: Arc<Mutex<HashSet<Pose>>>,
    score: Arc<Mutex<usize>>,
}

impl SnakeEngine {

    pub fn new(key_event_rx: mpsc::Receiver<KeyEvent>) -> SnakeEngine {
        let map_height: usize = 10;
        let map_widht: usize = 10;
        let mut init_food: HashSet<Pose> = HashSet::new();
        init_food.insert( Pose::random_in_range(0,map_widht-1, 0, map_height-1));

        SnakeEngine {
            snake: Arc::new(Mutex::new(Snake::new())),
            map_size: MapSize{height: map_height, width: map_widht},
            key_event_rx,
            snake_dir: Arc::new(Mutex::new(SnakeDirection::NotMoving)),
            food: Arc::new(Mutex::new(init_food)),
            score: Arc::new(Mutex::new(0)),
        }
    }

    pub fn run(self) {

        let (sim_event_tx, sim_event_rx) = mpsc::channel();

        let render_timer = timer::Timer::new();
        let _render_callback = self.create_render_routine(&render_timer);

        let sim_timer = timer::Timer::new();
        let _sim_callback = self.create_simulation_routine(&sim_timer, sim_event_tx);

        let map_size = self.map_size;

        loop {
            let key_event = self.key_event_rx.recv_timeout(std::time::Duration::from_millis(33));
            match key_event {
                Ok(ke) => {
                    let mut dir = self.snake_dir.deref().lock().unwrap();
                    match ke {
                        KeyEvent::ArrowUp => {
                            if *dir != SnakeDirection::Down {
                                *dir = SnakeDirection::Up
                            }
                        }
                        KeyEvent::ArrowDown => {
                            if *dir != SnakeDirection::Up {
                                *dir = SnakeDirection::Down
                            }
                        }
                        KeyEvent::ArrowLeft => {
                            if *dir != SnakeDirection::Right {
                                *dir = SnakeDirection::Left
                            }
                        }
                        KeyEvent::ArrowRight => {
                            if *dir != SnakeDirection::Left {
                                *dir = SnakeDirection::Right
                            }
                        }
                        KeyEvent::QKey => {
                            break;
                        }
                    }
                }
                _ => {}
            }

            match sim_event_rx.recv_timeout(std::time::Duration::from_secs(0)) {
                Ok(se) => {
                    match se {
                        SimulationEvent::SnakeCrashed => {
                            let mut stdout = stdout().into_raw_mode().unwrap();
                            write!(stdout, "{}The End: Snake Crashed", termion::cursor::Goto(1, (map_size.height + 4) as u16)).unwrap();
                            stdout.flush().unwrap();
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn create_simulation_routine(&self, timer: &Timer, sim_event_tx: mpsc::Sender<SimulationEvent>) -> Guard {

        let snake_dir = self.snake_dir.clone();
        let snake_guard = self.snake.clone();
        let map_size_sim = self.map_size.clone();
        let food = self.food.clone();
        let score = self.score.clone();

        timer.schedule_repeating(chrono::Duration::milliseconds(500), move || {

            let mut snake = snake_guard.lock().unwrap();
            let mut did_eat_food = false;
            let mut foods_to_remove: HashSet<Pose> = HashSet::new();

            let snake_back = snake.back();
            let dir = snake_dir.deref().lock().unwrap();
            match *dir {
                SnakeDirection::Up => {
                    if snake.front().y > 0 {
                        snake.move_up();
                    } else {
                        sim_event_tx.send(SimulationEvent::SnakeCrashed).unwrap();
                    }
                }
                SnakeDirection::Down => {
                    if snake.front().y < map_size_sim.height-1 {
                        snake.move_down();
                    } else {
                        sim_event_tx.send(SimulationEvent::SnakeCrashed).unwrap();
                    }
                }
                SnakeDirection::Left => {
                    if snake.front().x > 0 {
                        snake.move_left();
                    } else {
                        sim_event_tx.send(SimulationEvent::SnakeCrashed).unwrap();
                    }
                }
                SnakeDirection::Right => {
                    if snake.front().x < map_size_sim.width-1 {
                        snake.move_right();
                    } else {
                        sim_event_tx.send(SimulationEvent::SnakeCrashed).unwrap();
                    }
                }
                SnakeDirection::NotMoving => {}
            }

            if snake.is_crashed() {
                sim_event_tx.send(SimulationEvent::SnakeCrashed).unwrap();
            }

            for food_entity in food.lock().unwrap().iter() {
                if snake.is_part_of_body(food_entity) {
                    *score.lock().unwrap() += 100;
                    did_eat_food = true;
                    foods_to_remove.insert((*food_entity).clone());
                }
            }
            for ftr in foods_to_remove {
                food.lock().unwrap().remove(&ftr);
            }


            if did_eat_food {
                snake.grow_back(&snake_back);
            }


            let mut rng = rand::thread_rng();
            if rng.gen_range(0.0..1.0) < 0.1 {
                loop {
                    let proposed_food = Pose::random_in_range(0, map_size_sim.width - 1, 0, map_size_sim.height - 1);
                    if !snake.is_part_of_body(&proposed_food) {
                        food.lock().unwrap().insert(proposed_food);
                        break;
                    }
                }
            }
        })
    }

    fn create_render_routine(&self, timer: &Timer) -> Guard{

        let map_size = self.map_size.clone();
        let snake_guard = self.snake.clone();
        let score = self.score.clone();
        let food = self.food.clone();
        let mut stdout = stdout().into_raw_mode().unwrap();

        timer.schedule_repeating(chrono::Duration::milliseconds(33), move || {

            let snake = snake_guard.lock().unwrap();
            write!(stdout, "{}",  termion::clear::All).unwrap();

            for y in 0 ..= map_size.height+1 {
                write!(stdout, "{}", termion::cursor::Goto(1, (y + 1) as u16)).unwrap();
                for x in 0 ..= map_size.width+1 {
                    if  y == 0 || y == map_size.height + 1 ||
                        x == 0 || x == map_size.width + 1 {
                        print!("x");
                    } else if snake.is_part_of_body(&Pose{x: x-1, y: y-1}) {
                        print!("*");
                    } else {
                        print!(" ");
                    }
                }
            }

            for food_entity in food.lock().unwrap().iter() {
                write!(stdout, "{}{}", termion::cursor::Goto((food_entity.x + 2) as u16, (food_entity.y + 2) as u16), 'o').unwrap();
            }

            write!(stdout, "{}Score: {}", termion::cursor::Goto(1, (map_size.height + 3) as u16), score.lock().unwrap()).unwrap();
            stdout.flush().unwrap();
        })
    }
}

