pub struct LoopTimer<T> {
    elapsed_time: T,
    interval: T,
}

impl<T> LoopTimer<T>
where
    T: core::ops::AddAssign + core::ops::SubAssign + core::cmp::PartialOrd + Default + Copy,
{
    pub fn new(interval: T) -> Self {
        Self {
            elapsed_time: Default::default(),
            interval,
        }
    }

    pub fn should_update(&mut self, dt: T) -> bool {
        self.elapsed_time += dt;

        if self.elapsed_time >= self.interval {
            self.elapsed_time -= self.interval;
            true
        } else {
            false
        }
    }
}
