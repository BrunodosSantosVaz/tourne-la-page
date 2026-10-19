use super::*;

impl Heights {
    pub(super) fn new(values: Vec<f32>) -> Self {
        let mut this = Self {
            tree: vec![0.0; values.len() + 1],
            values,
        };
        for i in 0..this.values.len() {
            this.add(i, this.values[i]);
        }
        this
    }
    pub(super) fn add(&mut self, index: usize, delta: f32) {
        let mut i = index + 1;
        while i < self.tree.len() {
            self.tree[i] += delta;
            i += i.isolate_lowest_one();
        }
    }
    pub(super) fn set(&mut self, index: usize, value: f32) {
        let delta = value - self.values[index];
        self.values[index] = value;
        self.add(index, delta);
    }
    pub(super) fn prefix(&self, end: usize) -> f32 {
        let mut i = end.min(self.values.len());
        let mut sum = 0.0;
        while i > 0 {
            sum += self.tree[i];
            i &= i - 1;
        }
        sum
    }
    pub(super) fn push(&mut self, value: f32) {
        let i = self.values.len() + 1;
        let low = i.isolate_lowest_one();
        let sum = value + self.prefix(i - 1) - self.prefix(i - low);
        self.values.push(value);
        self.tree.push(sum);
    }
    pub(super) fn total(&self) -> f32 {
        self.prefix(self.values.len())
    }
    pub(super) fn find(&self, offset: f32) -> usize {
        let mut index = 0;
        let mut sum = 0.0;
        let mut step = self.tree.len().next_power_of_two() / 2;
        while step > 0 {
            let next = index + step;
            if next < self.tree.len() && sum + self.tree[next] <= offset {
                index = next;
                sum += self.tree[next];
            }
            step /= 2;
        }
        index.min(self.values.len().saturating_sub(1))
    }
}
