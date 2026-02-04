use std::fmt;
use wasm_bindgen::prelude::*;
use crate::world::point::WasmPoint as WasmPoint;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct WasmPosition {
    pub curr: WasmPoint,
    pub old: WasmPoint,
}

#[wasm_bindgen]
impl WasmPosition {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> WasmPosition {
        let point = WasmPoint::new(x, y);
        WasmPosition {
            curr: point,
            old: point,
        }
    }

    /// Creates a Position with different current and old positions
    pub fn new_with_old(curr_x: f32, curr_y: f32, old_x: f32, old_y: f32) -> WasmPosition {
        WasmPosition {
            curr: WasmPoint::new(curr_x, curr_y),
            old: WasmPoint::new(old_x, old_y),
        }
    }

    /// Creates a Position from two Points
    pub fn from_points(curr: WasmPoint, old: WasmPoint) -> WasmPosition {
        WasmPosition { curr, old }
    }

    /// Returns the current x coordinate
    pub fn get_curr_x(&self) -> f32 {
        self.curr.x
    }

    /// Returns the current y coordinate
    pub fn get_curr_y(&self) -> f32 {
        self.curr.y
    }

    /// Returns the old x coordinate
    pub fn get_old_x(&self) -> f32 {
        self.old.x
    }

    /// Returns the old y coordinate
    pub fn get_old_y(&self) -> f32 {
        self.old.y
    }

    /// Returns the current position as a Point
    pub fn get_curr(&self) -> WasmPoint {
        self.curr
    }

    /// Returns the old position as a Point
    pub fn get_old(&self) -> WasmPoint {
        self.old
    }

    /// Sets the current position
    pub fn set_curr(&mut self, x: f32, y: f32) {
        self.curr.x = x;
        self.curr.y = y;
    }

    /// Sets the old position
    pub fn set_old(&mut self, x: f32, y: f32) {
        self.old.x = x;
        self.old.y = y;
    }

    /// Sets the current position from a Point
    pub fn set_curr_point(&mut self, point: WasmPoint) {
        self.curr = point;
    }

    /// Sets the old position from a Point
    pub fn set_old_point(&mut self, point: WasmPoint) {
        self.old = point;
    }

    /// Updates the position: stores current as old, then sets new current
    pub fn update(&mut self, x: f32, y: f32) {
        self.old = self.curr;
        self.curr.x = x;
        self.curr.y = y;
    }

    /// Updates the position from a Point
    pub fn update_point(&mut self, point: WasmPoint) {
        self.old = self.curr;
        self.curr = point;
    }

    /// Returns the velocity (difference between current and old positions)
    pub fn velocity(&self) -> WasmPoint {
        self.curr - self.old
    }

    /// Returns the speed (magnitude of velocity)
    pub fn speed(&self) -> f32 {
        self.velocity().length()
    }

    /// Integrates the position using Verlet integration
    /// new_pos = curr + (curr - old) * damping + acceleration * dt^2
    pub fn integrate(&mut self, acceleration: WasmPoint, dt: f32, damping: f32) {
        let velocity = (self.curr - self.old) * damping;
        let new_pos = self.curr + velocity + acceleration * (dt * dt);
        self.old = self.curr;
        self.curr = new_pos;
    }

    /// Applies a constraint by moving the current position
    pub fn constrain(&mut self, target: WasmPoint) {
        self.curr = target;
    }

    /// Returns the distance between current and old positions
    pub fn displacement(&self) -> f32 {
        self.curr.distance_to(&self.old)
    }

    /// Resets old position to match current (stops motion)
    pub fn reset_velocity(&mut self) {
        self.old = self.curr;
    }
}

impl fmt::Display for WasmPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "curr: {} old: {}", self.curr, self.old)?;
        Ok(())
    }
}