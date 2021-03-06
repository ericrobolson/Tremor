use crate::lib_core::{
    ecs::systems::System,
    input::PlayerInput,
    math::{FixedNumber, Quaternion, Vec3},
    spatial::{
        physics::Capsule, physics::CollisionShapes, physics::Sphere, Aabb, Camera, CollisionList,
        PhysicBodies, Transform,
    },
    time::{GameFrame, Timer},
    voxels::{Chunk, ChunkManager, Voxel},
};

#[macro_export]
macro_rules! mask {
    ($mask_type:expr, $($next_mask:expr),*) => {
        $mask_type $(| $next_mask)*
    }; //;
}

// Simple macro to get the matching entities in the world.
macro_rules! matching_entities {
    ($world:tt, $mask_type:expr) => {
        $world
            .masks
            .iter()
            .enumerate()
            .filter(|(i, mask)| **mask & $mask_type == $mask_type)
            .map(|(i, mask)| i)
    };
}
mod assemblages;
pub mod operations;
mod systems;

const MAX_ENTITIES: usize = 400;

pub type Entity = usize;
// TODO: write a simple 'join'

macro_rules! m_world {
    (components: [$(($component_id:ident, $component_type:ty, $mask_name:ident, $mask_value:expr, $component_init:expr, $component_reset:expr),)*],
    systems: [$(($system_id:ident, $system_type:ty),)*]

) => {
        pub type MaskType = u32;
        pub struct Mask {}
        impl Mask {
            $(
                pub const $mask_name: MaskType = $mask_value;
            )*
        }

        pub struct World{
            next_entity_id: Entity,
            initialized: bool,
            entities_to_delete: usize,
            timer: Timer,
            frame: GameFrame,
            delta_t: FixedNumber,
            // Singular components
            pub world_voxels: ChunkManager,
            pub camera: Camera,
            //
            // Components
            //
            $(
                pub $component_id: Vec<$component_type>,
            )*
            //
            // Systems
            //
            $(
                $system_id: $system_type,
            )*
        }

        impl World{

            pub fn new(sim_hz: u32) -> Self{
                let mut world = Self{
                    timer: Timer::new(sim_hz),
                    next_entity_id: 0,
                    initialized: false,
                    entities_to_delete: 0,
                    frame: 0,
                    delta_t: 0.into(),
                    // Singular components
                    world_voxels: ChunkManager::new(16, 8, 16),
                    camera: Camera::new( (0, 0, 550).into(), (0, -20, 0).into()),
                    //
                    // Components
                    //
                    $(
                    $component_id: Vec::with_capacity(MAX_ENTITIES),
                    )*
                    //
                    // Systems
                    //
                    $(
                    $system_id: <$system_type>::new(MAX_ENTITIES),
                    )*
                };

                world.reset().unwrap();

                world
            }

            pub fn serialize(&self) -> Vec<u8> {
                unimplemented!("TODO: serialization")
            }

            pub fn deserialize(bytes: &Vec<u8>) -> Result<Self, String> {
                unimplemented!("TODO: serialization")
            }

            pub fn delta_t(&self) -> FixedNumber{
                self.delta_t
            }

            pub fn max_entities(&self ) -> usize{
                MAX_ENTITIES
            }

            pub fn reset(&mut self) -> Result<(), String>{
                // Init components
                for i in 0..MAX_ENTITIES{
                    if !self.initialized{
                        $(
                        self.$component_id.push($component_init);
                        )*
                    } else{
                        $(
                        self.$component_id[i] = $component_reset;
                        )*
                    }
                }

                // Reset systems
                {
                    $(
                        self.$system_id.reset();
                    )*
                }

                // Create basic ground voxels
                {
                  // assemblages::assemble_box_shape();
                }

                // Create capsule for testing
                {
                    match self.add_entity() {
                        Some(entity) => {
                            let transform = Transform::new((-100, 10, 0).into(), Quaternion::default(), Vec3::one());
                            let mut velocity = Transform::default();
                            velocity.rotation = Quaternion::from_z_rotation(FixedNumber::fraction(100.into()));

                            assemblages::assemble_capsule_shape(entity, transform, velocity,PhysicBodies::Kinematic,10.into(), 30.into(), self)?;
                        }
                        None => {}
                    }
                }

                // Create capsule for testing
                {
                    match self.add_entity() {
                        Some(entity) => {
                            let transform = Transform::new((-200, -50, 0).into(), Quaternion::default(), Vec3::one());
                            let mut velocity = Transform::default();

                            assemblages::assemble_capsule_shape(entity, transform, velocity,PhysicBodies::Static,10.into(), 400.into(), self)?;
                        }
                        None => {}
                    }
                }

                 // Create spheres for testing
                let spacing = 10;
                for x in 0..1{
                    let x = x * spacing;
                    for y in 0..1{
                        let y = y * spacing;
                            match self.add_entity() {
                                Some(entity) => {
                                    let x_vel: Vec3 = (0,0,0).into();

                                    let transform = Transform::new((-10 + x, 5 + y, 0).into(), Quaternion::default(), Vec3::one());


                                    assemblages::assemble_sphere_shape(
                                        entity,
                                        transform,
                                        Transform::new( x_vel / 10.into(), Quaternion::default(), Vec3::one()),
                                        Voxel::Metal,
                                        self
                                    )?;
                                },
                                _ => {}
                        }
                    }
                }

                self.initialized = true;
                self.entities_to_delete = 0;
                self.frame = 0;

                Ok(())
            }

            pub fn add_player(&mut self, input_id: usize) -> Result<Option<Entity>, String>{
                {
                    match self.add_entity() {
                        Some(entity) => {
                            self.add_component(entity, Mask::PLAYER_INPUT)?;
                            self.add_component(entity, Mask::PLAYER_INPUT_ID)?;
                            self.player_input_id[entity] = input_id;

                            self.add_component(entity, Mask::TRACKABLE)?;

                            let x_pos = 0 * 25;
                            let transform = Transform::new((x_pos as i32,10,0).into(), Quaternion::default(), Vec3::one());
                            assemblages::assemble_sphere_shape(entity, transform, Transform::default(), Voxel::Bone, self)?;

                            self.forces[entity].position.y = FixedNumber::from_i32(30);

                            //assemblages::assemble_capsule_shape(entity, transform, Transform::default(), self)?;


                            return Ok(Some(entity));
                        }
                        None => {}
                    }
                }

                Ok(None)
            }

            pub fn dispatch(&mut self) -> Result<(), String>{
                let delta_t = self.timer.delta_t();
                if self.timer.can_run(){
                    self.frame += 1;
                    self.delta_t = delta_t;
                    self.delta_t = 1.into(); // Since this is a fixed timestep, we can use delta_t = 1

                    self.world_voxels.update_frame(self.frame);

                    // Systems
                    {
                        // Dispatch
                        $(
                        <$system_type>::dispatch(self);
                        )*
                        // Cleanup
                        $(
                        <$system_type>::cleanup(self);
                        )*
                    }
                }

                for i in 0..MAX_ENTITIES {
                    // Remove deleted entities
                    if self.entities_to_delete > 0 && self.deleted[i] == true{
                        self.entities_to_delete -= 1;

                        self.masks[i] = Mask::EMPTY;
                        self.deleted[i] = false;
                    }

                    // Figure out next entity id
                    if self.masks[i] == Mask::EMPTY && self.next_entity_id <= i{
                        self.next_entity_id = i;
                    }
                }

                Ok(())
            }

            pub fn add_entity(&mut self) -> Option<Entity> {
                if self.next_entity_id >= MAX_ENTITIES{
                    return None;
                }

                for entity in self.masks.iter().enumerate().filter(|(_i, mask)| **mask == Mask::EMPTY).map(|(entity, _)| entity){
                    return Some(entity);
                }

                None
            }

            pub fn delete_entity(&mut self, entity: Entity) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Attempted to delete out of bounds entity.".into());
                }

                self.deleted[entity] = true;
                self.entities_to_delete += 1;

                Ok(())
            }

            pub fn add_component(&mut self, entity: Entity, mask: MaskType) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Out of bounds entity for adding component".into());
                }

                self.masks[entity] |= mask;

                Ok(())
            }

            pub fn has_component(&self, entity: Entity, mask: MaskType) -> bool {
                if entity >= MAX_ENTITIES {
                    return false;
                }

                return self.masks[entity] & mask == mask;
            }

            pub fn remove_component(&mut self, entity: Entity, mask: MaskType) -> Result<(), String>{
                if entity >= MAX_ENTITIES{
                    return Err("Out of bounds entity for deleting component".into());
                }

                let m = self.masks[entity];

                self.masks[entity] = m & !mask;

                Ok(())
            }

            // Helper methods
            pub fn set_position(&mut self, entity: Entity, position: Vec3) {
                self.prev_position[entity] = self.transforms[entity].position;
                self.transforms[entity].position = position;
            }

        }
    };
}

//TODO: Figure out a way to get rid of the manually specified bitshifting
//TODO: figure out a way to implement a trait that requires a 'default()'?

m_world![
    components: [
        // Sys components
        (masks, MaskType, EMPTY, 0 << 0, Mask::EMPTY, Mask::EMPTY),
        (deleted, bool, DELETED, 1 << 0, false, false),
        (parents, Entity, PARENT, 1 << 1, 0,0),
        // Engine components
        (inputs, PlayerInput, PLAYER_INPUT, 1 << 3, PlayerInput::new(), PlayerInput::new()),
        (player_input_id, usize, PLAYER_INPUT_ID, 1 << 4, 0,0),
        (transforms, Transform, TRANSFORM, 1 << 5, Transform::default(), Transform::default()),
        (velocities, Transform, VELOCITY, 1 << 6, Transform::default(), Transform::default()),
        (forces, Transform, FORCE, 1 << 7, Transform::default(), Transform::default()),
        (collision_shapes, CollisionShapes, COLLISION_SHAPE, 1 << 8, CollisionShapes::default(), CollisionShapes::default()),
        (collision_lists, CollisionList, COLLISIONS, 1 << 9, CollisionList::new(), CollisionList::new()),

        // Entity is trackable by the camera
        (trackable, bool, TRACKABLE, 1 << 12, true, true),
        (bodies, PhysicBodies, BODY, 1 << 13, PhysicBodies::Static, PhysicBodies::Static),

        (prev_position, Vec3, PREV_POSITION, 1 << 14, Vec3::new(), Vec3::new()),

        // Voxels
        (voxel_chunks, Chunk, VOXEL_CHUNK, 1 << 16, Chunk::new(16,16,16,2), Chunk::new(16,16,16, 2)),
    ],
    systems:[
        (input_action_system, systems::InputActions),
        //(physics, systems::Physics),
        (verlet_simulation, systems::VerletParticleSystem),
    ]
];
