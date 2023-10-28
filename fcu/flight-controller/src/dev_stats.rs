use shared::fcu_hal::FcuDevStatsFrame;

pub struct DevStatsCollector {
    is_collecting: bool,
    collection_time_length: f32,
    configured_update_rate: f32,
    stats_frame: Option<FcuDevStatsFrame>,

    // Collection metadata
    current_update_start_timestamp: f32,
    collection_start_timestamp: f32,
    last_update_timestamp: f32,

    // Running values to calculate stats
    update_elapsed_sum: f32,
    update_latency_sum: f32,
    update_latency_max: f32,
    packet_queue_len_sum: u32,
    packet_queue_len_max: u32,
    update_sample_count: u32,
}

impl DevStatsCollector {
    pub fn new() -> Self {
        Self {
            is_collecting: false,
            collection_time_length: 1.0,
            configured_update_rate: 0.02,
            stats_frame: None,

            // Collection metadata
            current_update_start_timestamp: 0.0,
            collection_start_timestamp: 0.0,
            last_update_timestamp: 0.0,

            // Running values to calculate stats
            update_elapsed_sum: 0.0,
            update_latency_sum: 0.0,
            update_latency_max: 0.0,
            packet_queue_len_sum: 0,
            packet_queue_len_max: 0,
            update_sample_count: 0,
        }
    }

    pub fn log_update_start(
        &mut self,
        timestamp: f32,
        packet_queue_len: u32,
        _cpu_utilization: f32,
    ) {
        if !self.is_collecting {
            return;
        }

        self.current_update_start_timestamp = timestamp;
        self.packet_queue_len_sum += packet_queue_len;
        self.packet_queue_len_max = self.packet_queue_len_max.max(packet_queue_len);

        if self.last_update_timestamp > 0.0 {
            let latency = timestamp - self.last_update_timestamp - self.configured_update_rate;

            self.update_latency_sum += latency;
            self.update_latency_max = self.update_latency_max.max(latency);
        }
    }

    pub fn log_update_end(&mut self, timestamp: f32) {
        if !self.is_collecting {
            return;
        }

        let elapsed = timestamp - self.current_update_start_timestamp;
        self.update_elapsed_sum += elapsed;
        self.update_sample_count += 1;
        self.last_update_timestamp = timestamp;

        if timestamp - self.collection_start_timestamp >= self.collection_time_length {
            self.end_collection(timestamp);
        }
    }

    pub fn start_collection(&mut self, timestamp: f32) {
        if self.is_collecting {
            return;
        }

        self.is_collecting = true;
        self.collection_start_timestamp = timestamp;
    }

    pub fn pop_stats_frame(&mut self) -> Option<FcuDevStatsFrame> {
        if let Some(frame) = self.stats_frame.clone() {
            let ret = Some(frame);
            self.stats_frame = None;

            ret
        } else {
            None
        }
    }

    fn end_collection(&mut self, timestamp: f32) {
        self.is_collecting = false;

        let sample_count = self.update_sample_count as f32;

        self.stats_frame = Some(FcuDevStatsFrame {
            timestamp: (timestamp * 1e3) as u64,
            cpu_utilization: 0.0,
            fcu_update_latency_avg: self.update_latency_sum / sample_count,
            fcu_update_latency_max: self.update_latency_max,
            packet_queue_length_avg: (self.packet_queue_len_sum as f32) / sample_count,
            packet_queue_length_max: self.packet_queue_len_max,
            fcu_update_elapsed_avg: self.update_elapsed_sum / sample_count,
        });
    }
}
