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
    /// Consecutive world steps in which no node moved more than the world's
    /// sleep threshold (see WasmWorld::step / note_step).
    quiet_steps: u32,
    /// Asleep: skipped by the simulation until woken (see invalidate).
    asleep: bool,
    /// Bounding box of the nodes, kept while asleep (nodes don't move then).
    sleep_min: WasmPoint,
    sleep_max: WasmPoint,
    pub radius: f32,
    pub link_target_distance: f32,
    pub render_type: u8, // 0 = segments, 1 = bezier
    /// Resistance to bending, from 0 (a chain of links, the default) to 1 —
    /// see relax_bends.
    bend_stiffness: f32,
    /// The way each end (first node, last node) leaves its anchor, as a unit
    /// vector, if set — see relax_bends.
    end_directions: [Option<WasmPoint>; 2],
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
            nodes,
            prev_positions: Vec::new(),
            quiet_steps: 0,
            asleep: false,
            sleep_min: WasmPoint::zero(),
            sleep_max: WasmPoint::zero(),
            link_target_distance: 7.0,
            radius: 5.0,
            render_type: 1, // Default to bezier
            bend_stiffness: 0.0,
            end_directions: [None, None],
        };
        w.invalidate();
        w
    }

    /// Creates a wire between two points with specified radius
    pub fn new_wire(xs: f32, ys: f32, xe: f32, ye: f32, radius: f32) -> WasmWire {
        let optimal_len = WasmWire::optimal_length(WasmPoint { x: xs, y: ys }, WasmPoint { x: xe, y: ye }, radius);
        let nodes = WasmWire::straight_nodes(xs, ys, xe, ye, optimal_len + 1);
        let mut w = WasmWire {
            nodes,
            prev_positions: Vec::new(),
            quiet_steps: 0,
            asleep: false,
            sleep_min: WasmPoint::zero(),
            sleep_max: WasmPoint::zero(),
            link_target_distance: radius * 3.0,
            radius,
            render_type: 1, // Default to bezier
            bend_stiffness: 0.0,
            end_directions: [None, None],
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
            nodes,
            prev_positions: Vec::new(),
            quiet_steps: 0,
            asleep: false,
            sleep_min: WasmPoint::zero(),
            sleep_max: WasmPoint::zero(),
            link_target_distance: link_target,
            radius,
            render_type,
            bend_stiffness: 0.0,
            end_directions: [None, None],
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

    /// Sets the radius (thickness) of the wire. Does not change the distance
    /// between nodes — see set_link_distance.
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.invalidate();
    }

    /// Rest distance between neighbouring nodes (the cable's "tension")
    pub fn get_link_distance(&self) -> f32 {
        self.link_target_distance
    }

    /// Sets the rest distance between neighbouring nodes
    pub fn set_link_distance(&mut self, distance: f32) {
        if distance.is_finite() && distance > 0.0 {
            self.link_target_distance = distance;
            self.invalidate();
        }
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

    /// Places the start node (see WasmNode::move_to)
    pub fn set_start(&mut self, x: f32, y: f32) {
        if let Some(node) = self.nodes.first_mut() {
            node.move_to(x, y);
            self.invalidate();
        }
    }

    /// Places the end node (see WasmNode::move_to)
    pub fn set_end(&mut self, x: f32, y: f32) {
        if let Some(node) = self.nodes.last_mut() {
            node.move_to(x, y);
            self.invalidate();
        }
    }

    /// Sets the fixed state of a specific node
    pub fn set_node_fixed(&mut self, node_idx: usize, fixed: bool) {
        if let Some(node) = self.nodes.get_mut(node_idx) {
            node.set_fixed(fixed);
            // A released end must start falling.
            self.invalidate();
        }
    }

    /// Places a node from outside the simulation (see WasmNode::move_to)
    pub fn set_node_position(&mut self, node_idx: usize, x: f32, y: f32) {
        if let Some(node) = self.nodes.get_mut(node_idx) {
            node.move_to(x, y);
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
        if !self.intersect(point, rad) {
            return;
        }
        for node in &mut self.nodes {
            let dt = node.get_position() - *point;
            let d2 = dt.x * dt.x + dt.y * dt.y;
            if d2 == 0.0 || d2 >= rad * rad {
                continue;
            }
            let i = d2.sqrt();
            if node.is_movable() {
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
            self.relax_bends();
        }
    }

    /// Resistance to bending, from 0 (a chain, the default) to 1.
    pub fn get_bend_stiffness(&self) -> f32 {
        self.bend_stiffness
    }

    /// Sets the resistance to bending, clamped to 0..=1 (see relax_bends).
    /// It adds up over the relaxation passes: with `n` constraint
    /// iterations, a stiffness `k` acts like about 1 - (1 - k)^n.
    pub fn set_bend_stiffness(&mut self, stiffness: f32) {
        if stiffness.is_finite() {
            self.bend_stiffness = stiffness.max(0.0).min(1.0);
            self.invalidate();
        }
    }

    /// Sets the way end `end` (0: first node, otherwise the last) leaves its
    /// anchor — a plug's direction out of its jack — normalized; a zero or
    /// non-finite vector clears it. Only acts with a bend stiffness (see
    /// relax_bends).
    pub fn set_end_direction(&mut self, end: usize, dx: f32, dy: f32) {
        let length = dx.hypot(dy);
        let direction = if length.is_finite() && length > 0.0 {
            Some(WasmPoint { x: dx / length, y: dy / length })
        } else {
            None
        };
        self.end_directions[if end == 0 { 0 } else { 1 }] = direction;
        self.invalidate();
    }

    /// x of the direction of end `end` (see set_end_direction), 0 if unset.
    pub fn get_end_direction_x(&self, end: usize) -> f32 {
        self.end_directions[if end == 0 { 0 } else { 1 }].map(|d| d.x).unwrap_or(0.0)
    }

    /// y of the direction of end `end` (see set_end_direction), 0 if unset.
    pub fn get_end_direction_y(&self, end: usize) -> f32 {
        self.end_directions[if end == 0 { 0 } else { 1 }].map(|d| d.y).unwrap_or(0.0)
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

    /// One pass straightening every three consecutive nodes a, b, c by
    /// `bend_stiffness`: the cable resists bending, so it curves smoothly
    /// instead of folding at any node (a hanging chain drops straight down
    /// and bends only at its lowest point). Position-based: the curvature
    /// a + c - 2b is reduced by `bend_stiffness`, the correction shared out
    /// by inverse mass, so a fixed node never moves and the link lengths are
    /// left to relax_links. An end with a direction gets the same treatment
    /// against a virtual fixed node `link_target_distance` behind it: the
    /// cable leaves that end along its direction, curving away gradually.
    fn relax_bends(&mut self) {
        let k = self.bend_stiffness;
        let n = self.nodes.len();
        if k <= 0.0 || n < 2 {
            return;
        }
        let weight = |node: &WasmNode| if node.is_movable() { 1.0 } else { 0.0 };

        if let Some(d) = self.end_directions[0] {
            let behind = self.nodes[0].get_position() - d * self.link_target_distance;
            let (_, b, c) = bend(
                behind,
                self.nodes[0].get_position(),
                self.nodes[1].get_position(),
                0.0,
                weight(&self.nodes[0]),
                weight(&self.nodes[1]),
                k,
            );
            self.shift(0, b);
            self.shift(1, c);
        }
        for i in 1..n - 1 {
            let (a, b, c) = bend(
                self.nodes[i - 1].get_position(),
                self.nodes[i].get_position(),
                self.nodes[i + 1].get_position(),
                weight(&self.nodes[i - 1]),
                weight(&self.nodes[i]),
                weight(&self.nodes[i + 1]),
                k,
            );
            self.shift(i - 1, a);
            self.shift(i, b);
            self.shift(i + 1, c);
        }
        if let Some(d) = self.end_directions[1] {
            let behind = self.nodes[n - 1].get_position() - d * self.link_target_distance;
            let (a, b, _) = bend(
                self.nodes[n - 2].get_position(),
                self.nodes[n - 1].get_position(),
                behind,
                weight(&self.nodes[n - 2]),
                weight(&self.nodes[n - 1]),
                0.0,
                k,
            );
            self.shift(n - 2, a);
            self.shift(n - 1, b);
        }
    }

    /// Moves a movable node by `delta` (as a constraint does: see set_position).
    fn shift(&mut self, index: usize, delta: WasmPoint) {
        let node = &mut self.nodes[index];
        if node.is_movable() && (delta.x != 0.0 || delta.y != 0.0) {
            let p = node.get_position() + delta;
            node.set_position(p.x, p.y);
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
        let rad = mouse_radius + self.radius;
        if self.intersect(&mouse, rad) {
            for n in self.nodes.iter_mut() {
                let d = n.get_position() - mouse;
                let d2 = d.x * d.x + d.y * d.y;
                if d2 == 0.0 || d2 >= rad * rad {
                    continue;
                }
                let i = d2.sqrt();
                if n.is_movable() {
                    reset = true;
                    n.set_position(mouse.x + (d.x / i) * rad, mouse.y + (d.y / i) * rad);
                }
            }
            if reset {
                self.invalidate()
            }
        }
    }

    /// Wakes the wire up (see is_sleeping) and restarts its count of still
    /// steps — called by every change that can set it in motion.
    pub fn invalidate(&mut self) {
        self.quiet_steps = 0;
        self.asleep = false;
    }

    /// Whether the wire is asleep: it stopped moving, so the world skips it
    /// until something wakes it (see WasmWorld::set_sleep_enabled).
    pub fn is_sleeping(&self) -> bool {
        self.asleep
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
        for n in self.nodes.iter_mut() {
            n.begin_step();
        }
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

    /// Unfixes a node held by the app, throwing it with the velocity it had
    /// while held: how far the app moved it over the most recent step
    /// interval (WasmNode::held_motion), spread over the step's `substeps`.
    /// A node held still is just released; a node that was already free is
    /// left as it is.
    pub(crate) fn release_node(&mut self, index: usize, substeps: u32) {
        if let Some(node) = self.nodes.get_mut(index) {
            if !node.get_fixed() {
                return;
            }
            node.set_fixed(false);
            let curr = node.get_position();
            node.position.old = curr - node.held_motion() / substeps.max(1) as f32;
            self.invalidate();
        }
    }

    /// Whether `point` is within `reach` of the wire's bounding box — a
    /// cheap test that rules out most wires before checking every node.
    pub(crate) fn intersect(&self, point: &WasmPoint, reach: f32) -> bool {
        let (min, max) = if self.asleep { (self.sleep_min, self.sleep_max) } else { self.bounds() };
        point.x >= min.x - reach && point.x <= max.x + reach && point.y >= min.y - reach && point.y <= max.y + reach
    }

    /// Bounding box of the nodes' current positions.
    fn bounds(&self) -> (WasmPoint, WasmPoint) {
        let mut min = WasmPoint { x: f32::INFINITY, y: f32::INFINITY };
        let mut max = WasmPoint { x: f32::NEG_INFINITY, y: f32::NEG_INFINITY };
        for n in &self.nodes {
            let p = n.get_position();
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        (min, max)
    }

    /// Called by the world after each step of an awake wire: counts the
    /// step as still if no node moved more than `threshold` since the
    /// step's snapshot, and puts the wire to sleep after `steps_needed`
    /// still steps in a row — dropping the nodes' residual velocity, so it
    /// doesn't jump when woken.
    pub(crate) fn note_step(&mut self, threshold: f32, steps_needed: u32) {
        if self.prev_positions.len() != self.nodes.len() {
            self.quiet_steps = 0;
            return;
        }
        let t2 = threshold * threshold;
        let still = self.nodes.iter().zip(&self.prev_positions).all(|(n, prev)| {
            let d = n.get_position() - *prev;
            d.x * d.x + d.y * d.y <= t2
        });
        if !still {
            self.quiet_steps = 0;
            return;
        }
        self.quiet_steps += 1;
        if self.quiet_steps >= steps_needed {
            for n in self.nodes.iter_mut() {
                n.position.old = n.position.curr;
            }
            let (min, max) = self.bounds();
            self.sleep_min = min;
            self.sleep_max = max;
            self.asleep = true;
        }
    }

    fn diameter(&self) -> f32 {
        self.radius * 2.0
    }

}

/// The corrections straightening a, b, c by `k` (see WasmWire::relax_bends):
/// the curvature C = a + c - 2b, reduced as a position-based constraint
/// with inverse masses `wa`, `wb`, `wc` (0 for a fixed node).
fn bend(a: WasmPoint, b: WasmPoint, c: WasmPoint, wa: f32, wb: f32, wc: f32, k: f32) -> (WasmPoint, WasmPoint, WasmPoint) {
    let zero = WasmPoint::zero();
    let s = wa + wc + 4.0 * wb;
    if s == 0.0 {
        return (zero, zero, zero);
    }
    let curvature = a + c - b * 2.0;
    let f = k / s;
    (curvature * (-f * wa), curvature * (2.0 * f * wb), curvature * (-f * wc))
}
