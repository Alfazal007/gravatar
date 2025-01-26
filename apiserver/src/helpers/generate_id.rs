use std::time::{SystemTime, UNIX_EPOCH};

pub struct Snowflake {
    pub machine_id: u64,
    pub counter: u64,
}

impl Snowflake {
    pub fn generate_id(&mut self) -> Result<u64, String> {
        let now = SystemTime::now();
        let epoch_res = now.duration_since(UNIX_EPOCH);
        if epoch_res.is_err() {
            return Err("Issue generating the id".to_string());
        }

        let epoch = epoch_res.unwrap().as_secs();
        let shifted_epoch = epoch << 23;
        let shifted_machine_id = self.machine_id << 13;
        let current_counter = self.counter;
        self.counter = (self.counter + 1) % 8192;
        Ok(shifted_epoch | shifted_machine_id | current_counter)
    }
}
