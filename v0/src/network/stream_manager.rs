use crate::event_queue;
use crate::lib_core::{time::Timer, LookUpGod};
use crate::network::{Packet, Sequence};

use event_queue::*;

pub struct StreamManager {
    send_timer: Timer,
    next_packet_id: Sequence,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            send_timer: Timer::new(20),
            next_packet_id: 1,
        }
    }

    pub fn queue_outbound(
        &mut self,
        event_queue: &EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        if self.send_timer.can_run() {}

        Ok(())
    }

    pub fn parse_inbound(&mut self, event_queue: &mut EventQueue) {}
}
