use std::fmt;
use wasm_bindgen::prelude::*;
use crate::world::point::Point as Point;
use crate::world::position::Position as Position;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub position: Position,
    pub velocity: Point,
    pub fixed: bool,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(constructor)]
    pub fn new(pos: Position, vel: Point, fix: bool) -> Node {
        Node {
            position: pos,
            velocity: vel,
            fixed: fix,
        }
    }

    /// Creates a Node with no initial velocity
    pub fn new_no_vel(x: f32, y: f32, fix: bool) -> Node {
        Node {
            position: Position::new(x, y),
            velocity: Point::zero(),
            fixed: fix,
        }
    }

    /// Creates a Node at origin with no velocity
    pub fn zero() -> Node {
        Node {
            position: Position::new(0.0, 0.0),
            velocity: Point::zero(),
            fixed: false,
        }
    }

    /// Creates a fixed Node at the given position
    pub fn new_fixed(x: f32, y: f32) -> Node {
        Node {
            position: Position::new(x, y),
            velocity: Point::zero(),
            fixed: true,
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

    /// Sets the current position
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.set_curr(x, y);
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
    pub fn get_position(&self) -> Point {
        self.position.get_curr()
    }

    /// Returns the velocity as a Point
    pub fn get_velocity(&self) -> Point {
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
    pub fn apply_force_point(&mut self, force: Point) {
        if !self.fixed {
            self.velocity += force;
        }
    }

    /// Updates the position based on velocity and acceleration
    pub fn update_position(&mut self, dt: f32, friction: f32, acceleration: Point) {
        if !self.fixed {
            let v = (self.position.curr - self.position.old) * friction;

            self.position.old = self.position.curr;
            self.position.curr += v + acceleration * dt * dt;
        }
    }

    /// Updates position with custom acceleration
    pub fn update_with_acceleration(&mut self, acceleration: Point, dt: f32, friction: f32) {
        if !self.fixed {
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
        self.velocity = Point::zero();
    }

    /// Returns the distance to another node
    pub fn distance_to(&self, other: &Node) -> f32 {
        self.position.curr.distance_to(&other.position.curr)
    }

    /// Returns the squared distance to another node (faster)
    pub fn distance_squared_to(&self, other: &Node) -> f32 {
        self.position.curr.distance_squared_to(&other.position.curr)
    }

    /// Converts to a string representation
    pub fn print(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "position: {}, velocity: {}, fixed: {}", self.position, self.velocity, self.fixed)?;

        Ok(())
    }
}
