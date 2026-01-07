use crate::map::{LevelMap, TileType};
use crate::player_types::JumpStyle;
use abi_stable::std_types::{RStr, RString};
use hackers::hackrs::stable_abi::StableUiBackend_TO;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ControlMode {
    SideScroll, // Gravity, Jump
    TopDown,    // Classic Zelda/RPG
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteBody {
    pub width: f32,
    pub height: f32,
    pub head_offset: f32, // From top down
    pub feet_offset: f32, // From bottom up
    pub collision_offset: [f32; 2],
}

impl Default for SpriteBody {
    fn default() -> Self {
        Self {
            width: 50.0,
            height: 100.0,
            head_offset: 10.0,
            feet_offset: 5.0,
            collision_offset: [0.0, 0.0],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlayerState {
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
    WallSlide,
    Afk,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Down,
    DownLeft,
    Left,
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
}

#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub move_direction: [f32; 2], // x, y (-1.0 to 1.0)
    pub jump_held: bool,
    pub run_held: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: [f32; 2],
    pub velocity: [f32; 2],
    pub speed: f32,
    pub control_mode: ControlMode,
    pub is_grounded: bool,
    pub is_jumping: bool,
    pub body: SpriteBody,
    pub state: PlayerState,
    pub facing_direction: Direction,
    pub afk_timer: f32,

    // Physics Config
    pub gravity: f32,
    pub jump_force: f32,
    pub friction: f32,
    pub max_run_speed: f32,

    // Wall Mechanics
    pub on_wall_left: bool,
    pub on_wall_right: bool,
    pub wall_slide_speed: f32,
    pub wall_jump_force: [f32; 2],
    pub jump_cut_off: f32,

    // Charge Jump
    pub jump_style: JumpStyle,
    pub jump_charge: f32,     // Current charge (seconds)
    pub max_charge_time: f32, // Seconds to reach max jump
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: [100.0, 100.0],
            velocity: [0.0, 0.0],
            speed: 200.0,
            control_mode: ControlMode::SideScroll,
            is_grounded: false,
            is_jumping: false,
            body: SpriteBody::default(),
            state: PlayerState::Idle,
            facing_direction: Direction::Right,
            afk_timer: 0.0,
            gravity: 500.0,
            jump_force: 400.0,
            friction: 0.90,
            max_run_speed: 400.0,
            on_wall_left: false,
            on_wall_right: false,
            wall_slide_speed: 50.0,
            wall_jump_force: [300.0, 400.0], // X push, Y up
            jump_cut_off: 0.5,
            jump_style: JumpStyle::Normal,
            jump_charge: 0.0,
            max_charge_time: 0.5,
        }
    }
}

impl Player {
    pub fn update(&mut self, input: &InputState, level_data: &LevelMap, dt: f32) {
        let mut input_active = false;

        match self.control_mode {
            ControlMode::TopDown => {
                let mut delta_x = 0.0;
                let mut delta_y = 0.0;
                let speed = self.speed * dt;

                if input.move_direction[1] < -0.1 {
                    delta_y -= speed;
                    input_active = true;
                    self.facing_direction = Direction::Up;
                }
                if input.move_direction[1] > 0.1 {
                    delta_y += speed;
                    input_active = true;
                    self.facing_direction = Direction::Down;
                }
                if input.move_direction[0] < -0.1 {
                    delta_x -= speed;
                    input_active = true;
                    self.facing_direction = Direction::Left;
                }
                if input.move_direction[0] > 0.1 {
                    delta_x += speed;
                    input_active = true;
                    self.facing_direction = Direction::Right;
                }

                // Diagonal Checks for Direction
                if delta_x > 0.0 && delta_y > 0.0 {
                    self.facing_direction = Direction::DownRight;
                }
                if delta_x > 0.0 && delta_y < 0.0 {
                    self.facing_direction = Direction::UpRight;
                }
                if delta_x < 0.0 && delta_y > 0.0 {
                    self.facing_direction = Direction::DownLeft;
                }
                if delta_x < 0.0 && delta_y < 0.0 {
                    self.facing_direction = Direction::UpLeft;
                }

                self.pos[0] += delta_x;
                self.pos[1] += delta_y;

                // State logic for TopDown
                if input_active {
                    self.state = PlayerState::Walk;
                    self.velocity = [delta_x / dt, delta_y / dt];
                } else {
                    self.state = PlayerState::Idle;
                    self.velocity = [0.0, 0.0];
                }
            }
            ControlMode::SideScroll => {
                // Reset wall state
                self.on_wall_left = false;
                self.on_wall_right = false;

                // Input
                // Use move_speed for acceleration force
                let move_accel = self.speed;
                let x_input = input.move_direction[0];

                let holding_left = x_input < -0.1;
                let holding_right = x_input > 0.1;

                if holding_left {
                    input_active = true;
                    self.facing_direction = Direction::Left;
                }
                if holding_right {
                    input_active = true;
                    self.facing_direction = Direction::Right;
                }

                // Accel / Friction
                if x_input.abs() > 0.0 {
                    self.velocity[0] += x_input * move_accel * 10.0 * dt;
                    // Cap speed
                    self.velocity[0] =
                        self.velocity[0].clamp(-self.max_run_speed, self.max_run_speed);
                } else {
                    // Friction
                    self.velocity[0] *= self.friction;
                    if self.velocity[0].abs() < 10.0 {
                        self.velocity[0] = 0.0;
                    }
                }

                // Collision Logic Phase 1: Horizontal (detect walls)
                self.pos[0] += self.velocity[0] * dt;

                let player_rect_x = [
                    self.pos[0] + self.body.collision_offset[0] - self.body.width / 2.0,
                    self.pos[1] + self.body.collision_offset[1] - self.body.height / 2.0,
                    self.body.width,
                    self.body.height,
                ];

                for tile in &level_data.tiles {
                    if tile.tile_type == TileType::Wall || tile.tile_type == TileType::Floor {
                        let tile_rect = [
                            tile.position[0],
                            tile.position[1],
                            tile.size[0],
                            tile.size[1],
                        ];

                        // AABB Check
                        if player_rect_x[0] < tile_rect[0] + tile_rect[2]
                            && player_rect_x[0] + player_rect_x[2] > tile_rect[0]
                            && player_rect_x[1] < tile_rect[1] + tile_rect[3]
                            && player_rect_x[1] + player_rect_x[3] > tile_rect[1]
                        {
                            // Horizontal Collision Resolution
                            if self.velocity[0] > 0.0 {
                                // Moving Right -> Snap to Left of tile
                                self.pos[0] = tile_rect[0]
                                    - self.body.collision_offset[0]
                                    - self.body.width / 2.0
                                    - 0.1;
                                self.velocity[0] = 0.0;
                                self.on_wall_right = true;
                            } else if self.velocity[0] < 0.0 {
                                // Moving Left -> Snap to Right of tile
                                self.pos[0] = tile_rect[0] + tile_rect[2]
                                    - self.body.collision_offset[0]
                                    + self.body.width / 2.0
                                    + 0.1;
                                self.velocity[0] = 0.0;
                                self.on_wall_left = true;
                            }
                        }
                    }
                }

                // Wall Slide Logic & Gravity
                let is_wall_sliding = !self.is_grounded
                    && ((self.on_wall_left && holding_left)
                        || (self.on_wall_right && holding_right));

                if is_wall_sliding {
                    // Slide down slowly
                    if self.velocity[1] > self.wall_slide_speed {
                        self.velocity[1] = self.wall_slide_speed;
                    } else {
                        self.velocity[1] += self.gravity * dt * 0.5; // Reduced gravity
                    }
                    self.state = PlayerState::WallSlide;
                } else {
                    // Normal Gravity
                    self.velocity[1] += self.gravity * dt;
                }

                // Jump & Wall Jump
                match self.jump_style {
                    JumpStyle::Normal => {
                        if input.jump_held {
                            if self.is_grounded {
                                self.velocity[1] = -self.jump_force;
                                self.is_grounded = false;
                                self.is_jumping = true;
                                input_active = true;
                            } else if is_wall_sliding || self.on_wall_left || self.on_wall_right {
                                // Wall Jump Logic (Same as before)
                                if self.on_wall_left {
                                    self.velocity[0] = self.wall_jump_force[0];
                                    self.velocity[1] = -self.wall_jump_force[1];
                                } else if self.on_wall_right {
                                    self.velocity[0] = -self.wall_jump_force[0];
                                    self.velocity[1] = -self.wall_jump_force[1];
                                }
                                self.is_jumping = true;
                                input_active = true;
                            }
                        }

                        // Variable Jump Height: Cut velocity if released
                        if !input.jump_held && self.velocity[1] < -0.1 {
                            self.velocity[1] *= self.jump_cut_off;
                        }
                    }
                    JumpStyle::Charge => {
                        if input.jump_held {
                            if self.is_grounded {
                                // Charge up
                                self.jump_charge += dt;
                                if self.jump_charge > self.max_charge_time {
                                    self.jump_charge = self.max_charge_time;
                                }
                            } else if is_wall_sliding || self.on_wall_left || self.on_wall_right {
                                // Immediate wall jump
                                if self.on_wall_left {
                                    self.velocity[0] = self.wall_jump_force[0];
                                    self.velocity[1] = -self.wall_jump_force[1];
                                } else if self.on_wall_right {
                                    self.velocity[0] = -self.wall_jump_force[0];
                                    self.velocity[1] = -self.wall_jump_force[1];
                                }
                                self.is_jumping = true;
                                input_active = true;
                            }
                        } else {
                            // Key released
                            if self.is_grounded && self.jump_charge > 0.0 {
                                // Execute Jump
                                let charge_ratio = self.jump_charge / self.max_charge_time;
                                // Min jump force (e.g. 50%) + remaining based on charge
                                let effective_force = self.jump_force * (0.5 + 0.5 * charge_ratio);

                                self.velocity[1] = -effective_force;
                                self.is_grounded = false;
                                self.is_jumping = true;
                                input_active = true;
                                self.jump_charge = 0.0;
                            } else {
                                self.jump_charge = 0.0;
                            }
                        }
                    }
                }

                // Run Logic (Shift)
                let is_running = input.run_held;
                // current_move_speed logic removed as it was unused and physics uses max_run_speed for cap currently.
                // TODO: Apply correct speed cap for walking vs running if needed.

                // 2. Vertical Movement & Collision
                self.pos[1] += self.velocity[1] * dt;

                // Horizontal speed limit applied via velocity[0] accumulation above

                if holding_left {
                    self.facing_direction = Direction::Left;
                    input_active = true;
                    if self.is_grounded {
                        self.state = if is_running {
                            PlayerState::Run
                        } else {
                            PlayerState::Walk
                        };
                    }
                } else if holding_right {
                    self.facing_direction = Direction::Right;
                    input_active = true;
                    if self.is_grounded {
                        self.state = if is_running {
                            PlayerState::Run
                        } else {
                            PlayerState::Walk
                        };
                    }
                } else {
                    // Deceleration (handled above via friction on velocity)
                }

                let mut grounded_this_frame = false;
                let player_rect_y = [
                    self.pos[0] + self.body.collision_offset[0] - self.body.width / 2.0,
                    self.pos[1] + self.body.collision_offset[1] - self.body.height / 2.0,
                    self.body.width,
                    self.body.height,
                ];

                for tile in &level_data.tiles {
                    if tile.tile_type == TileType::Wall || tile.tile_type == TileType::Floor {
                        let tile_rect = [
                            tile.position[0],
                            tile.position[1],
                            tile.size[0],
                            tile.size[1],
                        ];

                        // AABB Check
                        if player_rect_y[0] < tile_rect[0] + tile_rect[2]
                            && player_rect_y[0] + player_rect_y[2] > tile_rect[0]
                            && player_rect_y[1] < tile_rect[1] + tile_rect[3]
                            && player_rect_y[1] + player_rect_y[3] > tile_rect[1]
                        {
                            // Vertical Collision Resolution
                            if self.velocity[1] > 0.0 {
                                // Falling -> Land on top
                                self.pos[1] = tile_rect[1] - self.body.height / 2.0;
                                self.velocity[1] = 0.0;
                                grounded_this_frame = true;
                            } else if self.velocity[1] < 0.0 {
                                // Jumping -> Bonk head on bottom
                                self.pos[1] = tile_rect[1] + tile_rect[3] + self.body.height / 2.0;
                                self.velocity[1] = 0.0;
                            }
                        }
                    }
                }

                self.is_grounded = grounded_this_frame;
                if self.is_grounded {
                    self.is_jumping = false;
                }

                // Default Floor Fallback (if fallen off map)
                if self.pos[1] > 2000.0 {
                    // Reset to spawn
                    self.pos = level_data.spawn_point;
                    self.velocity = [0.0, 0.0];
                }

                // State Logic SideScroll
                if !self.is_grounded {
                    if self.velocity[1] < 0.0 {
                        self.state = PlayerState::Jump;
                    } else {
                        self.state = PlayerState::Fall;
                    }
                } else if self.velocity[0].abs() > 10.0 {
                    if self.state != PlayerState::Run {
                        self.state = PlayerState::Walk;
                    }
                } else {
                    self.state = PlayerState::Idle;
                }
            }
        }

        // AFK Logic
        if input_active {
            self.afk_timer = 0.0;
            if self.state == PlayerState::Afk {
                self.state = PlayerState::Idle;
            }
        } else {
            // Only increment if idle
            if self.state == PlayerState::Idle {
                self.afk_timer += dt;
                if self.afk_timer > 5.0 {
                    self.state = PlayerState::Afk;
                }
            }
        }
    }
}
