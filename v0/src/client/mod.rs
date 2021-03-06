use crate::constants;
use crate::event_queue;
use event_queue::*;

use crate::lib_core;
use lib_core::{ecs::Mask, ecs::MaskType, ecs::World, input::ClientInputMapper};

use crate::network;
use network::{connection_layer::ConnectionManager, stream_manager::StreamManager};

pub struct Client {
    pub world: World,
    connection: ConnectionManager,
    input_handler: ClientInputMapper,
    stream_manager: StreamManager,
}

impl Client {
    pub fn new() -> Self {
        let mut client = Self {
            world: World::new(constants::SIMULATION_HZ as u32),
            connection: ConnectionManager::new(constants::MAX_CLIENT_CONNECTIONS),
            input_handler: ClientInputMapper::new(constants::SIMULATION_HZ as u32),
            stream_manager: StreamManager::new(),
        };

        client
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        //TODO: wire up

        match player.player_type {
            PlayerTypes::Local => match self.input_handler.add_local_player() {
                Some(local_player_id) => {
                    self.world.add_player(local_player_id).unwrap();
                }
                None => {}
            },
            PlayerTypes::Remote => {
                unimplemented!("IMPLEMENT REMOTE");
            }
            PlayerTypes::Spectator => {
                unimplemented!("IMPLEMENT SPECTATOR");
            }
        }

        Ok(())
    }

    pub fn execute(
        &mut self,
        event_queue: &mut EventQueue,
        socket_out_queue: &mut EventQueue,
    ) -> Result<(), String> {
        // Connection manager stuff
        self.connection.read_all(event_queue)?;

        // Handle input
        {
            self.input_handler.execute(event_queue)?;

            for input in event_queue.events().iter().filter_map(|e| match e {
                Some((_duration, event)) => match event {
                    Events::InputPoll(input) => Some(input),
                    _ => None,
                },
                None => None,
            }) {
                //TODO: should this really belong here? not sure
                const INPUT_PASS: MaskType = Mask::PLAYER_INPUT | Mask::PLAYER_INPUT_ID;
                for entity in self
                    .world
                    .masks
                    .iter()
                    .enumerate()
                    .filter(|(i, mask)| **mask & INPUT_PASS == INPUT_PASS)
                    .map(|(i, mask)| i)
                {
                    if self.world.player_input_id[entity] == input.player_input_id {
                        self.world.inputs[entity] = *input;
                    }
                }
            }
        }

        // Execute sim
        self.world.dispatch()?;

        // Send out events
        self.connection.write_all(event_queue, socket_out_queue)?;
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum PlayerTypes {
    Local,
    Remote,
    Spectator,
}

pub struct Player {
    pub player_type: PlayerTypes,
    pub remote_addr: Option<network::SocketAddr>,
}

pub struct RollbackManager {}

impl RollbackManager {
    fn save_state(&mut self) {
        unimplemented!()
    }

    fn load_state(&mut self) {
        unimplemented!()
    }

    fn advance_game_state(&mut self) {
        unimplemented!()
    }

    fn register_input(&mut self, player_id: u8, frame: u16) {
        unimplemented!()
    }
}
