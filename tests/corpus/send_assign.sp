/// Send statements (§2.7 statement-head rule, §16 send_stmt) and every
/// assignment-target shape (§16 assign_target).
actor Worker {
    jobs: i64,

    pub fn run(&mut self, n: i64) {
        self.jobs = self.jobs + n;
    }
}

fn dispatch(worker: Handle<Worker>, hub: Hub, ptr: &mut i64) {
    send worker.run(1);
    send hub.pool.run(2);
    let mut values = vec![0; 4];
    let mut count = 0;
    count = count + 1;
    values[0] = count;
    *ptr = count;
}
