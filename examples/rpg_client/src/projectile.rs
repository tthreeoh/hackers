use crate::map::LevelMap;
use crate::player::{ControlMode, InputState, Player};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProjectileType {
    Rolling, // Physics based (gravity + bounce/roll)
    Bullet,  // Linear, no gravity, stops on wall
    Boom,    // Stationary explosion
}

#[derive(Clone, Debug)]
pub struct Projectile {
    pub projectile_type: ProjectileType,
    pub sprite_path: String,
    pub active: bool,
    pub lifetime: f32,
    pub scale: f32,
    pub wrap: bool,
    pub age: f32,

    // Physics simulation
    // We reuse Player struct for physics as it already handles map collisions,
    // gravity, velocity, etc. We just drive it with "fake" input.
    pub sim: Player,
}

impl Projectile {
    pub fn new(
        pos: [f32; 2],
        vel: [f32; 2],
        p_type: ProjectileType,
        sprite: String,
        scale: f32,
        size: [f32; 2],
        wrap: bool,
    ) -> Self {
        let mut sim = Player::default();
        sim.pos = pos;
        sim.velocity = vel;
        sim.body.width = size[0];
        sim.body.height = size[1];
        sim.body.collision_offset = [0.0, 0.0];

        match p_type {
            ProjectileType::Rolling => {
                sim.control_mode = ControlMode::SideScroll;
                sim.friction = 0.99; // Low friction
                sim.gravity = 800.0;
            }
            ProjectileType::Bullet => {
                sim.control_mode = ControlMode::SideScroll;
                sim.gravity = 0.0;
                sim.friction = 1.0; // No friction
                sim.max_run_speed = 2000.0; // Allow high speed
            }
            ProjectileType::Boom => {
                sim.control_mode = ControlMode::SideScroll;
                sim.gravity = 0.0; // No gravity for boom
                sim.friction = 0.0; // Stationary
            }
        }

        Self {
            projectile_type: p_type,
            sprite_path: sprite,
            active: true,
            lifetime: 5.0, // Default, override for boom
            scale,
            wrap,
            age: 0.0,
            sim,
        }
    }

    pub fn update(&mut self, level_data: &LevelMap, dt: f32) -> Option<[f32; 2]> {
        if !self.active {
            return None;
        }

        self.lifetime -= dt;
        self.age += dt;

        if self.lifetime <= 0.0 {
            self.active = false;
            return None;
        }

        let input = InputState::default();
        let mut explode_pos: Option<[f32; 2]> = None;

        match self.projectile_type {
            ProjectileType::Rolling => {
                // Bounce logic
                if self.sim.on_wall_left {
                    self.sim.velocity[0] = 300.0; // Bounce right
                    self.sim.on_wall_left = false;
                } else if self.sim.on_wall_right {
                    self.sim.velocity[0] = -300.0; // Bounce left
                    self.sim.on_wall_right = false;
                }
            }
            ProjectileType::Bullet => {
                // Bullet logic: if it hits a wall, it dies.
                if self.sim.on_wall_left || self.sim.on_wall_right {
                    self.active = false;
                    explode_pos = Some(self.sim.pos);
                }
            }
            ProjectileType::Boom => {
                // Boom just sits there until lifetime expires
            }
        }

        self.sim.update(&input, level_data, dt);

        if self.wrap && level_data.width > 0.0 {
            if self.sim.pos[0] < 0.0 {
                self.sim.pos[0] += level_data.width;
            } else if self.sim.pos[0] > level_data.width {
                self.sim.pos[0] -= level_data.width;
            }
        }

        explode_pos
    }
}
