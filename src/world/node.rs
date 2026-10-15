use std::fmt;
use wasm_bindgen::prelude::*;
use crate::world::point::WasmPoint as WasmPoint;
use crate::world::position::WasmPosition as WasmPosition;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct WasmNode {
    pub position: WasmPosition,
    pub velocity: WasmPoint,
    pub fixed: bool,
    /// Distance moved with move_to while fixed since the current world step
    /// began, and during the previous step — the "hand" velocity a released
    /// node is thrown with (see held_motion).
    pending_motion: WasmPoint,
    last_motion: WasmPoint,
}

#[wasm_bindgen]
impl WasmNode {
    #[wasm_bindgen(constructor)]
    pub fn new(pos: WasmPosition, vel: WasmPoint, fix: bool) -> WasmNode {
        WasmNode {
            position: pos,
            velocity: vel,
            fixed: fix,
            pending_motion: WasmPoint::zero(),
            last_motion: WasmPoint::zero(),
        }
    }

    /// Creates a Node with no initial velocity
    pub fn new_no_vel(x: f32, y: f32, fix: bool) -> WasmNode {
        WasmNode {
            position: WasmPosition::new(x, y),
            velocity: WasmPoint::zero(),
            fixed: fix,
            pending_motion: WasmPoint::zero(),
            last_motion: WasmPoint::zero(),
        }
    }

    /// Creates a Node at origin with no velocity
    pub fn zero() -> WasmNode {
        WasmNode {
            position: WasmPosition::new(0.0, 0.0),
            velocity: WasmPoint::zero(),
            fixed: false,
            pending_motion: WasmPoint::zero(),
            last_motion: WasmPoint::zero(),
        }
    }

    /// Creates a fixed Node at the given position
    pub fn new_fixed(x: f32, y: f32) -> WasmNode {
        WasmNode {
            position: WasmPosition::new(x, y),
            velocity: WasmPoint::zero(),
            fixed: true,
            pending_motion: WasmPoint::zero(),
            last_motion: WasmPoint::zero(),
        }
    }

    /// Returns the current x position
    pub fn get_x(&self) -> f32 {
        self.position.curr.x
    }

    /// Returns the current y position
    pub fn get_y(&self) -> f32 {
        self.position.curr.y
    }

    /// Returns the velocity x component
    pub fn get_velocity_x(&self) -> f32 {
        self.velocity.x
    }

    /// Returns the velocity y component
    pub fn get_velocity_y(&self) -> f32 {
        self.velocity.y
    }

    /// Returns the fixed value (0.0 = movable, 1.0 = fixed)
    pub fn get_fixed(&self) -> bool {
        self.fixed
    }

    /// Sets the current position only — the previous one is kept, so the
    /// move counts as velocity at the next integration (Verlet). This is
    /// what collisions and constraints use internally; to place a node from
    /// outside the simulation, use move_to.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.set_curr(x, y);
    }

    /// Places the node from outside the simulation. A movable node is
    /// teleported, with no velocity. A fixed node (held by the app, e.g. a
    /// dragged or plugged cable end) only has its current position set: its
    /// previous position follows it at every integration, so it never
    /// builds up the whole distance moved while fixed as velocity (see
    /// WasmWorld::set_wire_node_fixed for the throw on release).
    pub fn move_to(&mut self, x: f32, y: f32) {
        if self.fixed {
            self.pending_motion += WasmPoint { x, y } - self.position.curr;
            self.position.set_curr(x, y);
        } else {
            self.position.set_curr(x, y);
            self.position.set_old(x, y);
        }
    }

    /// Sets the velocity
    pub fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.velocity.x = vx;
        self.velocity.y = vy;
    }

    /// Sets whether the node is fixed
    pub fn set_fixed(&mut self, fixed: bool) {
        self.fixed = fixed;
    }

    /// Makes the node fixed (immovable)
    pub fn make_fixed(&mut self) {
        self.fixed = true;
    }

    /// Makes the node movable
    pub fn make_movable(&mut self) {
        self.fixed = false;
    }

    /// Returns true if the node is fixed
    pub fn is_fixed(&self) -> bool {
        self.fixed != false
    }

    /// Returns true if the node is movable
    pub fn is_movable(&self) -> bool {
        self.fixed == false
    }

    /// Returns the current position as a Point
    pub fn get_position(&self) -> WasmPoint {
        self.position.get_curr()
    }

    /// Returns the velocity as a Point
    pub fn get_velocity(&self) -> WasmPoint {
        self.velocity
    }

    /// Returns the speed (magnitude of velocity)
    pub fn get_speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Applies a force to the node (changes velocity)
    pub fn apply_force(&mut self, fx: f32, fy: f32) {
        if !self.fixed {
            self.velocity.x += fx;
            self.velocity.y += fy;
        }
    }

    /// Applies a force from a Point
    pub fn apply_force_point(&mut self, force: WasmPoint) {
        if !self.fixed {
            self.velocity += force;
        }
    }

    /// Updates the position based on velocity and acceleration
    pub fn update_position(&mut self, dt: f32, friction: f32, acceleration: WasmPoint) {
        if self.fixed {
            // Track the app's moves: velocity on release = last movement only.
            self.position.old = self.position.curr;
        } else {
            let v = (self.position.curr - self.position.old) * friction;

            self.position.old = self.position.curr;
            self.position.curr += v + acceleration * dt * dt;
        }
    }

    /// Updates position with custom acceleration
    pub fn update_with_acceleration(&mut self, acceleration: WasmPoint, dt: f32, friction: f32) {
        if self.fixed {
            self.position.old = self.position.curr;
        } else {
            self.position.integrate(acceleration, dt, friction);
        }
    }

    /// Moves the node by a delta amount
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.curr.x += dx;
        self.position.curr.y += dy;
    }

    /// Resets the velocity to zero
    pub fn reset_velocity(&mut self) {
        self.position.reset_velocity();
        self.velocity = WasmPoint::zero();
    }

    /// Returns the distance to another node
    pub fn distance_to(&self, other: &WasmNode) -> f32 {
        self.position.curr.distance_to(&other.position.curr)
    }

    /// Returns the squared distance to another node (faster)
    pub fn distance_squared_to(&self, other: &WasmNode) -> f32 {
        self.position.curr.distance_squared_to(&other.position.curr)
    }

    /// Converts to a string representation
    pub fn print(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for WasmNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "position: {}, velocity: {}, fixed: {}", self.position, self.velocity, self.fixed)?;

        Ok(())
    }
}

// Non-wasm methods
impl WasmNode {
    /// Called by the world at the start of each step: the movement since
    /// the previous step becomes the last step's.
    pub(crate) fn begin_step(&mut self) {
        self.last_motion = self.pending_motion;
        self.pending_motion = WasmPoint::zero();
    }

    /// How far the app moved this (fixed) node over the most recent step
    /// interval: since the current step began if it moved since, else
    /// during the previous step. Zero once it has been held still for a
    /// whole step.
    pub(crate) fn held_motion(&self) -> WasmPoint {
        if self.pending_motion.x != 0.0 || self.pending_motion.y != 0.0 {
            self.pending_motion
        } else {
            self.last_motion
        }
    }
}
