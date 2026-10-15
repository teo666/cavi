pub mod point;
pub mod position;
pub mod node;
pub mod wire;

use wasm_bindgen::prelude::*;
use crate::world::node::WasmNode;
use crate::world::point::WasmPoint;
use crate::world::wire::WasmWire;

#[wasm_bindgen]
pub struct WasmWorld {
    wires: Vec<WasmWire>,
    mouse: WasmPoint,
    wire_data_buffer: Vec<f32>,
    // Configuration
    mouse_radius: f32,
    pointer_radius: f32,
    response_coef: f32,
    friction: f32,
    acceleration: WasmPoint,
    /// Simulated time covered by one sub-step.
    time_step: f32,
    /// Sub-steps (integration + link relaxation) per step().
    substeps: u32,
    /// Link relaxation passes per sub-step.
    constraint_iterations: u32,
    /// Interpolation factor of the last build_render_data.
    render_alpha: f32,
}

#[wasm_bindgen]
impl WasmWorld {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmWorld {
        WasmWorld {
            wires: Vec::new(),
            mouse: WasmPoint { x: 0.0, y: 0.0 },
            wire_data_buffer: Vec::new(),
            // Default configuration values
            mouse_radius: 40.0,
            pointer_radius: 20.0,
            response_coef: 0.0,
            friction: 0.95,
            acceleration: WasmPoint { x: 0.0, y: 10.0 },
            time_step: 0.3,
            substeps: 2,
            constraint_iterations: 1,
            render_alpha: 1.0,
        }
    }

    /// One physics step, then the render buffer rebuilt for the new state —
    /// step() followed by build_render_data(1.0).
    pub fn update(&mut self) {
        self.step();
        self.build_render_data(1.0);
    }

    /// Advances the simulation by one step: `substeps` sub-steps of
    /// `time_step` each (mouse collision, link collisions, integration and
    /// `constraint_iterations` link relaxations). Does not touch the render
    /// buffer — see build_render_data. Node positions before the step are
    /// kept, so frames drawn between two steps can be interpolated.
    pub fn step(&mut self) {
        for w in self.wires.iter_mut() {
            w.snapshot_positions();
        }
        for _ in 0..self.substeps {
            for w in self.wires.iter_mut() {
                w.check_mouse_collision(self.mouse, self.mouse_radius);
                w.check_wire_elements_collisions(self.response_coef);
                w.update_with_iterations(self.time_step, self.friction, self.acceleration, self.constraint_iterations);
            }
        }
    }

    /// Rebuilds the render buffer (wire_data_ptr) from the current wires,
    /// with every movable node drawn `alpha` (0..1) of the way from where it
    /// was before the last step to where it is now: 1 draws the current
    /// state, lower values draw a state between the last two steps. Lets a
    /// display faster than the physics rate draw a new position on every
    /// frame instead of repeating the last one. Fixed nodes are always drawn
    /// where they are. Can be called any number of times between steps
    /// (e.g. after adding or deleting a wire).
    pub fn build_render_data(&mut self, alpha: f32) {
        let alpha = if alpha.is_finite() { alpha.clamp(0.0, 1.0) } else { 1.0 };
        self.render_alpha = alpha;

        // Calculate Bezier points for all wires and populate buffer directly
        self.wire_data_buffer.clear();

        for wire in &self.wires {
            let positions = wire.render_positions(alpha);
            let points = match wire.render_type {
                0 => positions,
                _ => Self::catmull_to_bezier_points(&positions), // 1 and default: bezier
            };
            
            // Store wire metadata
            self.wire_data_buffer.push(wire.node_count() as f32);
            self.wire_data_buffer.push(wire.radius);
            self.wire_data_buffer.push(wire.render_type as f32);
            self.wire_data_buffer.push((points.len() * 2) as f32);
            
            // Store path data (Bezier curve control points or segments)
            for point in &points {
                self.wire_data_buffer.push(point.x);
                self.wire_data_buffer.push(point.y);
            }
        }
    }

    /// Sets the mouse position
    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse.x = x;
        self.mouse.y = y;
    }

    /// Returns the number of wires
    pub fn wire_count(&self) -> usize {
        self.wires.len()
    }

    /// Gets a specific wire by index
    pub fn get_wire(&self, index: usize) -> Option<WasmWire> {
        self.wires.get(index).cloned()
    }

    /// Deletes a wire by index
    pub fn delete_wire(&mut self, index: usize) -> bool {
        if index < self.wires.len() {
            self.wires.remove(index);
            true
        } else {
            false
        }
    }

    /// Gets a node from a specific wire
    pub fn get_wire_node(&self, wire_idx: usize, node_idx: usize) -> Option<WasmNode> {
        self.wires.get(wire_idx).and_then(|wire| wire.get_node(node_idx))
    }

    /// Gets x coordinate of a node in a wire
    pub fn get_wire_node_x(&self, wire_idx: usize, node_idx: usize) -> f32 {
        self.wires.get(wire_idx)
            .map(|wire| wire.get_node_x(node_idx))
            .unwrap_or(0.0)
    }

    /// Gets y coordinate of a node in a wire
    pub fn get_wire_node_y(&self, wire_idx: usize, node_idx: usize) -> f32 {
        self.wires.get(wire_idx)
            .map(|wire| wire.get_node_y(node_idx))
            .unwrap_or(0.0)
    }

    /// Adds a default wire to the world
    pub fn add_wire_debug(&mut self) {
        self.wires.push(WasmWire::new());
    }

    /// Adds a wire between two points with specified radius
    pub fn add_wire(&mut self, xs: f32, ys: f32, xe: f32, ye: f32, radius: f32) {
        self.wires.push(WasmWire::new_wire(xs, ys, xe, ye, radius));
    }

    /// Adds a wire with specific node count and link target distance
    pub fn add_wire_with_count(&mut self, xs: f32, ys: f32, xe: f32, ye: f32, node_count: usize, link_target: f32, radius: f32, render_type: u8) {
        self.wires.push(WasmWire::new_with_count(xs, ys, xe, ye, node_count, link_target, radius, render_type));
    }

    /// Gets the number of nodes in a specific wire
    pub fn get_wire_node_count(&self, wire_idx: usize) -> usize {
        self.wires.get(wire_idx)
            .map(|wire| wire.node_count())
            .unwrap_or(0)
    }

    pub fn get_wire_radius(&self, wire_idx: usize) -> f32 {
        self.wires.get(wire_idx)
            .map(|wire| wire.get_radius())
            .unwrap_or(0.0)
    }

    /// Sets the radius of a specific wire
    pub fn set_wire_radius(&mut self, wire_idx: usize, radius: f32) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.set_radius(radius);
        }
    }

    /// Adds a node to a specific wire
    pub fn add_wire_node(&mut self, wire_idx: usize, x: f32, y: f32, fixed: bool) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.add_node(x, y, fixed);
        }
    }

    /// Adds a node to a specific wire at a specific index
    pub fn add_wire_node_at(&mut self, wire_idx: usize, node_idx: usize, x: f32, y: f32, fixed: bool) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.add_node_at(node_idx, x, y, fixed);
        }
    }

    /// Removes a node from a specific wire
    pub fn remove_wire_node(&mut self, wire_idx: usize, node_idx: usize) -> bool {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.remove_node(node_idx)
        } else {
            false
        }
    }

    /// Sets the position of the first node (start) of a wire
    pub fn set_wire_start(&mut self, wire_idx: usize, x: f32, y: f32) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.set_start(x, y);
        }
    }

    /// Sets the position of the last node (end) of a wire
    pub fn set_wire_end(&mut self, wire_idx: usize, x: f32, y: f32) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.set_end(x, y);
        }
    }

    /// Sets the number of nodes in a specific wire
    pub fn set_wire_node_count(&mut self, wire_idx: usize, node_count: usize) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.set_node_count(node_count);
        }
    }

    /// Sets the fixed state of a specific node in a wire
    pub fn set_wire_node_fixed(&mut self, wire_idx: usize, node_idx: usize, fixed: bool) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.set_node_fixed(node_idx, fixed);
        }
    }

    /// Sets the position of a specific node in a wire
    pub fn set_wire_node_position(&mut self, wire_idx: usize, node_idx: usize, x: f32, y: f32) {
        if let Some(wire) = self.wires.get_mut(wire_idx) {
            wire.set_node_position(node_idx, x, y);
        }
    }

    /// Calculates the default node count for a wire based on distance and link target
    pub fn wire_optimal_length(start_x: f32, start_y: f32, end_x: f32, end_y: f32, radius: f32) -> usize {
        WasmWire::optimal_length(WasmPoint { x: start_x, y: start_y }, WasmPoint { x: end_x, y: end_y }, radius)
    }

    /// Returns the pointer to the wire data buffer
    /// Buffer format per wire: [node_count, radius, render_type, path_length, ...path_data]
    pub fn wire_data_ptr(&self) -> *const f32 {
        self.wire_data_buffer.as_ptr()
    }

    /// Returns the length of the wire data buffer
    pub fn wire_data_len(&self) -> usize {
        self.wire_data_buffer.len()
    }

    // Configuration getters and setters
    
    /// Gets the mouse interaction radius
    pub fn get_mouse_radius(&self) -> f32 {
        self.mouse_radius
    }

    /// Sets the mouse interaction radius
    pub fn set_mouse_radius(&mut self, radius: f32) {
        self.mouse_radius = radius;
    }

    /// Gets the pointer collision radius
    pub fn get_pointer_radius(&self) -> f32 {
        self.pointer_radius
    }

    /// Sets the pointer collision radius
    pub fn set_pointer_radius(&mut self, radius: f32) {
        self.pointer_radius = radius;
    }

    /// Gets the collision response coefficient
    pub fn get_response_coef(&self) -> f32 {
        self.response_coef
    }

    /// Sets the collision response coefficient
    pub fn set_response_coef(&mut self, coef: f32) {
        self.response_coef = coef;
    }

    /// Gets the friction coefficient
    pub fn get_friction(&self) -> f32 {
        self.friction
    }

    /// Sets the friction coefficient
    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
    }

    /// Gets the acceleration vector
    pub fn get_acceleration(&self) -> WasmPoint {
        self.acceleration
    }

    /// Gets the x component of acceleration
    pub fn get_acceleration_x(&self) -> f32 {
        self.acceleration.x
    }

    /// Gets the y component of acceleration
    pub fn get_acceleration_y(&self) -> f32 {
        self.acceleration.y
    }

    /// Sets the acceleration vector
    pub fn set_acceleration(&mut self, x: f32, y: f32) {
        self.acceleration = WasmPoint { x, y };
    }

    /// Gets the simulated time covered by one sub-step (default 0.3)
    pub fn get_time_step(&self) -> f32 {
        self.time_step
    }

    /// Sets the simulated time covered by one sub-step. Larger is faster
    /// but less stable. Ignored unless finite and positive.
    pub fn set_time_step(&mut self, time_step: f32) {
        if time_step.is_finite() && time_step > 0.0 {
            self.time_step = time_step;
        }
    }

    /// Gets the number of sub-steps per step (default 2)
    pub fn get_substeps(&self) -> u32 {
        self.substeps
    }

    /// Sets the number of sub-steps per step (at least 1). Each sub-step
    /// advances time by `time_step`, so this changes speed as well as
    /// accuracy: for a stiffer cable at the same speed, raise
    /// `constraint_iterations` instead.
    pub fn set_substeps(&mut self, substeps: u32) {
        self.substeps = substeps.max(1);
    }

    /// Gets the link relaxation passes per sub-step (default 1)
    pub fn get_constraint_iterations(&self) -> u32 {
        self.constraint_iterations
    }

    /// Sets the link relaxation passes per sub-step (at least 1): more
    /// passes make cables stiffer (less stretch) without changing speed.
    pub fn set_constraint_iterations(&mut self, iterations: u32) {
        self.constraint_iterations = iterations.max(1);
    }

    /// Gets the interpolation factor used by the last build_render_data
    pub fn get_render_alpha(&self) -> f32 {
        self.render_alpha
    }

    /// x of a node as drawn by the last build_render_data (interpolated
    /// like the render buffer) — for DOM elements that must stay on the
    /// drawn cable, e.g. plugs.
    pub fn get_wire_node_render_x(&self, wire_idx: usize, node_idx: usize) -> f32 {
        self.wires.get(wire_idx)
            .map(|wire| wire.render_position(node_idx, self.render_alpha).x)
            .unwrap_or(0.0)
    }

    /// y of a node as drawn by the last build_render_data — see get_wire_node_render_x
    pub fn get_wire_node_render_y(&self, wire_idx: usize, node_idx: usize) -> f32 {
        self.wires.get(wire_idx)
            .map(|wire| wire.render_position(node_idx, self.render_alpha).y)
            .unwrap_or(0.0)
    }

    /// Whether a node of a wire is fixed (false if out of range) — without
    /// copying the node out like get_wire_node does.
    pub fn get_wire_node_fixed(&self, wire_idx: usize, node_idx: usize) -> bool {
        self.wires.get(wire_idx)
            .map(|wire| wire.is_node_fixed(node_idx))
            .unwrap_or(false)
    }

    /// Calculates Bezier points from node positions using Catmull-Rom to Bezier conversion
    fn catmull_to_bezier_points(nodes: &[WasmPoint]) -> Vec<WasmPoint> {
        if nodes.len() < 3 {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut p0 = nodes[0];
        let mut p1 = nodes[0];
        let mut p2 = nodes[1];
        let mut p3 = nodes[2];
        
        result.push(p0);

        for i in 0..nodes.len() - 1 {
            let bp1 = WasmPoint { 
                x: ((-p0.x + 6.0 * p1.x + p2.x) / 6.0), 
                y: ((-p0.y + 6.0 * p1.y + p2.y) / 6.0) 
            };
            let bp2 = WasmPoint { 
                x: ((p1.x + 6.0 * p2.x - p3.x) / 6.0), 
                y: ((p1.y + 6.0 * p2.y - p3.y) / 6.0) 
            };
            
            p0 = p1;
            p1 = p2;
            p2 = p3;
            
            if i + 3 == nodes.len() {
                p3 = nodes[i + 2];
            } else if i + 3 > nodes.len() {
                p3 = nodes[i + 1];
            } else {
                p3 = nodes[i + 3];
            }
            
            result.push(bp1);
            result.push(bp2);
            result.push(p1);
        }
        
        result
    }
}


