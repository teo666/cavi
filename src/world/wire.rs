use wasm_bindgen::prelude::*;
use crate::world::point::WasmPoint as WasmPoint;
use crate::world::node::WasmNode as WasmNode;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmWire {
    nodes: Vec<WasmNode>,
    /// Node positions at the start of the last world step, for render
    /// interpolation (see WasmWorld::build_render_data). Empty until the
    /// first step.
    prev_positions: Vec<WasmPoint>,
    pub iterations: u32,
    pub radius: f32,
    pub link_target_distance: f32,
    pub render_type: u8, // 0 = segments, 1 = bezier
}

#[wasm_bindgen]
impl WasmWire {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmWire {
        let nodes = (0..30)
            .map(|i| {
                WasmNode::new_no_vel(i as f32 * 5.0 + 50f32, 10f32 + 50f32,  i == 0 || i == 29)
            })
            .collect();
        let mut w = WasmWire {
            iterations: 10,
            nodes,
            prev_positions: Vec::new(),
            link_target_distance: 7.0,
            radius: 5.0,
            render_type: 1, // Default to bezier
        };
        w.invalidate();
        w
    }

    /// Creates a wire between two points with specified radius
    pub fn new_wire(xs: f32, ys: f32, xe: f32, ye: f32, radius: f32) -> WasmWire {
        let optimal_len = WasmWire::optimal_length(WasmPoint { x: xs, y: ys }, WasmPoint { x: xe, y: ye }, radius);
        let nodes = WasmWire::straight_nodes(xs, ys, xe, ye, optimal_len + 1);
        let mut w = WasmWire {
            iterations: 10,
            nodes,
            prev_positions: Vec::new(),
            link_target_distance: radius * 3.0,
            radius,
            render_type: 1, // Default to bezier
        };
        w.invalidate();
        w
    }

    /// Creates a wire with specific node count and link target distance
    pub fn new_with_count(xs: f32, ys: f32, xe: f32, ye: f32, node_count: usize, link_target: f32, radius: f32, render_type: u8) -> WasmWire {
        if node_count < 2 {
            panic!("Wire must have at least 2 nodes");
        }
        
        // Fixed start and end, movable nodes evenly spaced in between.
        let nodes = WasmWire::straight_nodes(xs, ys, xe, ye, node_count);

        let mut w = WasmWire {
            iterations: 10,
            nodes,
            prev_positions: Vec::new(),
            link_target_distance: link_target,
            radius,
            render_type,
        };
        w.invalidate();
        w
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// Returns the number of nodes in the wire
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Gets a specific node by index
    pub fn get_node(&self, index: usize) -> Option<WasmNode> {
        self.nodes.get(index).copied()
    }

    /// Gets the x coordinate of a node at the given index
    pub fn get_node_x(&self, index: usize) -> f32 {
        self.nodes.get(index).map(|n| n.get_x()).unwrap_or(0.0)
    }

    /// Gets the y coordinate of a node at the given index
    pub fn get_node_y(&self, index: usize) -> f32 {
        self.nodes.get(index).map(|n| n.get_y()).unwrap_or(0.0)
    }

    /// Sets the radius of the wire
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.link_target_distance = radius * 3.0;
    }

    /// Adds a new node at the specified position
    pub fn add_node(&mut self, x: f32, y: f32, fixed: bool) {
        let node = WasmNode::new_no_vel(x, y, fixed);
        self.nodes.push(node);
        self.invalidate();
    }

    /// Adds a new node at a specific index
    pub fn add_node_at(&mut self, index: usize, x: f32, y: f32, fixed: bool) {
        if index <= self.nodes.len() {
            let node = WasmNode::new_no_vel(x, y, fixed);
            self.nodes.insert(index, node);
            self.invalidate();
        }
    }

    /// Removes a node at the specified index
    pub fn remove_node(&mut self, index: usize) -> bool {
        if index < self.nodes.len() && self.nodes.len() > 2 {
            self.nodes.remove(index);
            self.invalidate();
            true
        } else {
            false // Can't remove if it would leave less than 2 nodes
        }
    }

    /// Gets the render type (0 = segments, 1 = bezier)
    pub fn get_render_type(&self) -> u8 {
        self.render_type
    }

    /// Sets the render type (0 = segments, 1 = bezier)
    pub fn set_render_type(&mut self, render_type: u8) {
        self.render_type = render_type;
    }

    /// Sets the number of nodes in the wire
    /// Redistributes nodes evenly between start and end points
    pub fn set_node_count(&mut self, new_count: usize) {
        if new_count < 2 {
            return; // Wire must have at least 2 nodes (start and end)
        }

        let current_count = self.nodes.len();
        if new_count == current_count {
            return; // No change needed
        }

        // Store start and end positions (they are fixed)
        let start_pos = self.nodes[0].get_position();
        let start_fixed = self.nodes[0].get_fixed();
        let end_pos = self.nodes[self.nodes.len() - 1].get_position();
        let end_fixed = self.nodes[self.nodes.len() - 1].get_fixed();

        // Create new node vector
        let mut new_nodes = Vec::with_capacity(new_count);
        
        // Add start node
        new_nodes.push(WasmNode::new_no_vel(start_pos.x, start_pos.y, start_fixed));
        
        // Add intermediate nodes evenly spaced
        for i in 1..new_count - 1 {
            let t = i as f32 / (new_count - 1) as f32;
            let x = start_pos.x + (end_pos.x - start_pos.x) * t;
            let y = start_pos.y + (end_pos.y - start_pos.y) * t;
            new_nodes.push(WasmNode::new_no_vel(x, y, false)); // Intermediate nodes are movable
        }
        
        // Add end node
        new_nodes.push(WasmNode::new_no_vel(end_pos.x, end_pos.y, end_fixed));

        // Replace nodes
        self.nodes = new_nodes;
        self.invalidate();
    }

    /// Gets the start point of the wire
    pub fn get_start(&self) -> WasmPoint {
        self.nodes.first().map(|n| n.get_position()).unwrap_or(WasmPoint::zero())
    }

    /// Gets the end point of the wire
    pub fn get_end(&self) -> WasmPoint {
        self.nodes.last().map(|n| n.get_position()).unwrap_or(WasmPoint::zero())
    }

    /// Sets the start point position (if it's fixed)
    pub fn set_start(&mut self, x: f32, y: f32) {
        if let Some(node) = self.nodes.first_mut() {
            node.set_position(x, y);
            self.invalidate();
        }
    }

    /// Sets the end point position (if it's fixed)
    pub fn set_end(&mut self, x: f32, y: f32) {
        if let Some(node) = self.nodes.last_mut() {
            node.set_position(x, y);
            self.invalidate();
        }
    }

    /// Sets the fixed state of a specific node
    pub fn set_node_fixed(&mut self, node_idx: usize, fixed: bool) {
        if let Some(node) = self.nodes.get_mut(node_idx) {
            node.set_fixed(fixed);
        }
    }

    /// Sets the position of a specific node
    pub fn set_node_position(&mut self, node_idx: usize, x: f32, y: f32) {
        if let Some(node) = self.nodes.get_mut(node_idx) {
            node.set_position(x, y);
            self.invalidate();
        }
    }

    /// Returns the length of the wire (sum of all segment distances)
    pub fn length(&self) -> f32 {
        let mut total = 0.0;
        for i in 1..self.nodes.len() {
            total += self.nodes[i - 1].distance_to(&self.nodes[i]);
        }
        total
    }

    /// Checks collision with a point and adjusts nodes
    pub fn check_collision(&mut self, point: &WasmPoint, pointer_radius: f32) {
        let mut reset = false;
        let rad = self.radius + pointer_radius;
        for node in &mut self.nodes {
            let dt = node.get_position() - *point;
            let i = dt.x.hypot(dt.y);
            if i == 0.0 {
                continue;
            }
            if node.is_movable() && i < rad {
                reset = true;
                node.set_position(
                    point.x + (dt.x / i) * rad,
                    point.y + (dt.y / i) * rad
                );
            }
        }
        if reset {
            self.invalidate()
        }
    }

    /// Updates the wire physics simulation
    pub fn update(&mut self, dt: f32, friction: f32, acceleration: WasmPoint) {
        self.update_with_iterations(dt, friction, acceleration, 1);
    }

    /// Integrates every node by `dt`, then relaxes the distance between
    /// neighbouring nodes `iterations` times. More iterations make the
    /// cable stiffer (less stretch) without making the simulation faster.
    pub fn update_with_iterations(&mut self, dt: f32, friction: f32, acceleration: WasmPoint, iterations: u32) {
        for node in self.nodes.iter_mut() {
            node.update_position(dt, friction, acceleration);
        }

        for _ in 0..iterations.max(1) {
            self.relax_links();
        }
    }

    /// Whether the node at `index` is fixed (false if out of range).
    pub fn is_node_fixed(&self, index: usize) -> bool {
        self.nodes.get(index).map(|n| n.get_fixed()).unwrap_or(false)
    }

    /// One pass moving every pair of neighbouring nodes towards
    /// `link_target_distance` from each other.
    fn relax_links(&mut self) {
        for i in 1..self.nodes.len() {
            let (left, right) = self.nodes.split_at_mut(i);
            let o1 = &mut left[left.len() - 1];
            let o2 = &mut right[0];
            let dp = o1.get_position() - o2.get_position();
            let mut dist = dp.x.hypot(dp.y);
            if dist == 0.0 {
                dist = 1.0
            }
            let fact = WasmPoint {
                x: dp.x / dist,
                y: dp.y / dist,
            };
            let delta = self.link_target_distance - dist;

            if o1.is_movable() {
                let new_pos = o1.get_position() + fact * 0.5 * delta;
                o1.set_position(new_pos.x, new_pos.y);
            }
            if o2.is_movable() {
                let new_pos = o2.get_position() - fact * 0.5 * delta;
                o2.set_position(new_pos.x, new_pos.y);
            }
        }
    }

    /// Checks for collisions between wire elements
    pub fn check_wire_elements_collisions(&mut self, response_coef: f32) {
        let dia = self.diameter();
        for i in 1..self.nodes.len() {
            let (left, right) = self.nodes.split_at_mut(i);
            let o1 = &mut left[left.len() - 1];
            let o2 = &mut right[0];
            let mut dt = o1.get_position() - o2.get_position();

            let mut d2 = dt.x.powi(2) + dt.y.powi(2);
            if d2 == 0.0 {
                d2 = 1.0
            }
            if d2 < dia * dia {
                d2 = d2.sqrt();
                dt = dt / d2;
                d2 = 0.5 * response_coef * (d2 - dia);
                
                if o1.is_movable() {
                    let new_pos = o1.get_position() - dt * d2;
                    o1.set_position(new_pos.x, new_pos.y);
                }
                if o2.is_movable() {
                    let new_pos = o2.get_position() - dt * d2;
                    o2.set_position(new_pos.x, new_pos.y);
                }
            }
        }
    }

    /// Checks collision with mouse/pointer position
    pub fn check_mouse_collision(&mut self, mouse: WasmPoint, mouse_radius: f32) {
        let mut reset = false;
        if self.intersect(&mouse) {
            for n in self.nodes.iter_mut() {
                let d = n.get_position() - mouse;
                let i = d.x.hypot(d.y);
                if i == 0.0 {
                    continue;
                }
                if n.is_movable() && i < (mouse_radius + self.radius) {
                    reset = true;
                    n.set_position(
                        mouse.x + (d.x / i) * (mouse_radius + self.radius),
                        mouse.y + (d.y / i) * (mouse_radius + self.radius)
                    );
                }
            }
            if reset {
                self.invalidate()
            }
        }
    }

    /// Resets the iteration counter
    pub fn invalidate(&mut self) {
        self.iterations = 0;
    }

    pub fn optimal_length(start: WasmPoint, end: WasmPoint, node_radius: f32) -> usize {
        let d = (start.x - end.x).hypot(start.y - end.y) / 4.0 / node_radius * 3.6;
        d.floor() as usize
    }
}

// Non-wasm methods
impl WasmWire {
    /// `count` nodes evenly spaced on the segment from (xs, ys) to (xe, ye):
    /// fixed at both ends, movable in between.
    fn straight_nodes(xs: f32, ys: f32, xe: f32, ye: f32, count: usize) -> Vec<WasmNode> {
        let count = count.max(2);
        (0..count)
            .map(|i| {
                let t = i as f32 / (count - 1) as f32;
                let terminal = i == 0 || i == count - 1;
                WasmNode::new_no_vel(xs + (xe - xs) * t, ys + (ye - ys) * t, terminal)
            })
            .collect()
    }

    /// Remembers the current node positions as the "previous" state for
    /// render interpolation — called by the world at the start of a step.
    pub(crate) fn snapshot_positions(&mut self) {
        self.prev_positions.clear();
        self.prev_positions.extend(self.nodes.iter().map(|n| n.get_position()));
    }

    /// Where node `index` should be drawn: `alpha` of the way from its
    /// position before the last step to its current one. Fixed nodes, and
    /// every node when the node count changed since the last step, are
    /// drawn where they are now — a fixed node is placed by the app (a
    /// plugged or dragged end) and must not lag behind it.
    pub(crate) fn render_position(&self, index: usize, alpha: f32) -> WasmPoint {
        let node = match self.nodes.get(index) {
            Some(node) => node,
            None => return WasmPoint::zero(),
        };
        let curr = node.get_position();
        if node.get_fixed() || alpha >= 1.0 || self.prev_positions.len() != self.nodes.len() {
            return curr;
        }
        let prev = self.prev_positions[index];
        WasmPoint {
            x: prev.x + (curr.x - prev.x) * alpha,
            y: prev.y + (curr.y - prev.y) * alpha,
        }
    }

    /// render_position for every node, in order.
    pub(crate) fn render_positions(&self, alpha: f32) -> Vec<WasmPoint> {
        (0..self.nodes.len()).map(|i| self.render_position(i, alpha)).collect()
    }

    fn intersect(&self, _point: &WasmPoint) -> bool {
        // TODO: implement proper collision detection with the wire
        true
    }

    fn diameter(&self) -> f32 {
        self.radius * 2.0
    }

}
