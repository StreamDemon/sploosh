actor Counter {
    state: i64,

    pub fn inc(&mut self, n: i64) {
        self.state = self.state + n;
    }

    pub fn get(&self) -> i64 {
        self.state
    }
}

