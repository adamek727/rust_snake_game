use rand::Rng;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Pose {
    pub x: usize,
    pub y: usize,
}

impl Pose {
    pub fn random_in_range(x_min: usize, x_max: usize, y_min: usize, y_max: usize) -> Pose {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(x_min..=x_max);
        let y = rng.gen_range(y_min..=y_max);
        Pose { x, y }
    }
}