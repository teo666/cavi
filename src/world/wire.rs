use wasm_bindgen::prelude::*;
use crate::world::point::Point as Point;
use crate::world::node::Node as Node;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Wire {
    nodes: Vec<Node>,
    pub iterations: u32,
    pub radius: f32,
    pub link_target_distance: f32,
    pub render_type: u8, // 0 = segments, 1 = bezier
}

#[wasm_bindgen]
impl Wire {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Wire {
        let nodes = (0..30)
            .map(|i| {
                Node::new_no_vel(i as f32 * 5.0 + 50f32, 10f32 + 50f32,  i == 0 || i == 29)
            })
            .collect();
        let mut w = Wire {
            iterations: 10,
            nodes,
            link_target_distance: 7.0,
            radius: 5.0,
            render_type: 1, // Default to bezier
        };
        w.invalidate();
        w
    }

    /// Creates a wire between two points with specified radius
    pub fn new_wire(xs: f32, ys: f32, xe: f32, ye: f32, radius: f32) -> Wire {
        let mut nodes = vec![Node::new_no_vel(xs, ys, true), Node::new_no_vel(xe, ye, true)];
        let optimal_len = Wire::optimal_length(nodes[0].get_position(), nodes[nodes.len() - 1].get_position(), radius);
        for _i in 1..optimal_len {
            nodes.insert(1, Node::new_no_vel(100.0, 110.0, false))
        }
        let mut w = Wire {
            iterations: 10,
            nodes,
            link_target_distance: radius * 3.0,
            radius,
            render_type: 1, // Default to bezier
        };
        w.invalidate();
        w
    }

    /// Creates a wire with specific node count and link target distance
    pub fn new_with_count(xs: f32, ys: f32, xe: f32, ye: f32, node_count: usize, link_target: f32, radius: f32, render_type: u8) -> Wire {
        if node_count < 2 {
            panic!("Wire must have at least 2 nodes");
        }
        
        // Create start and end nodes as fixed points
        let mut nodes = vec![Node::new_no_vel(xs, ys, true), Node::new_no_vel(xe, ye, true)];
        
        // Insert movable nodes in between
        for _i in 1..node_count - 1 {
            nodes.insert(1, Node::new_no_vel(100.0, 110.0, false))
        }
        
        let mut w = Wire {
            iterations: 10,
            nodes,
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
    pub fn get_node(&self, index: usize) -> Option<Node> {
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
        let node = Node::new_no_vel(x, y, fixed);
        self.nodes.push(node);
        self.invalidate();
    }

    /// Adds a new node at a specific index
    pub fn add_node_at(&mut self, index: usize, x: f32, y: f32, fixed: bool) {
        if index <= self.nodes.len() {
            let node = Node::new_no_vel(x, y, fixed);
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
        new_nodes.push(Node::new_no_vel(start_pos.x, start_pos.y, start_fixed));
        
        // Add intermediate nodes evenly spaced
        for i in 1..new_count - 1 {
            let t = i as f32 / (new_count - 1) as f32;
            let x = start_pos.x + (end_pos.x - start_pos.x) * t;
            let y = start_pos.y + (end_pos.y - start_pos.y) * t;
            new_nodes.push(Node::new_no_vel(x, y, false)); // Intermediate nodes are movable
        }
        
        // Add end node
        new_nodes.push(Node::new_no_vel(end_pos.x, end_pos.y, end_fixed));

        // Replace nodes
        self.nodes = new_nodes;
        self.invalidate();
    }

    /// Gets the start point of the wire
    pub fn get_start(&self) -> Point {
        self.nodes.first().map(|n| n.get_position()).unwrap_or(Point::zero())
    }

    /// Gets the end point of the wire
    pub fn get_end(&self) -> Point {
        self.nodes.last().map(|n| n.get_position()).unwrap_or(Point::zero())
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

    /// Returns the length of the wire (sum of all segment distances)
    pub fn length(&self) -> f32 {
        let mut total = 0.0;
        for i in 1..self.nodes.len() {
            total += self.nodes[i - 1].distance_to(&self.nodes[i]);
        }
        total
    }

    /// Checks collision with a point and adjusts nodes
    pub fn check_collision(&mut self, point: &Point, pointer_radius: f32) {
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
    pub fn update(&mut self, dt: f32, friction: f32, acceleration: Point) {
        for node in self.nodes.iter_mut() {
            node.update_position(dt, friction, acceleration);
        }

        for i in 1..self.nodes.len() {
            let (left, right) = self.nodes.split_at_mut(i);
            let o1 = &mut left[left.len() - 1];
            let o2 = &mut right[0];
            let dp = o1.get_position() - o2.get_position();
            let mut dist = dp.x.hypot(dp.y);
            if dist == 0.0 {
                dist = 1.0
            }
            let fact = Point {
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
    pub fn check_mouse_collision(&mut self, mouse: Point, mouse_radius: f32) {
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

    pub fn optimal_length(start: Point, end: Point, node_radius: f32) -> usize {
        let d = (start.x - end.x).hypot(start.y - end.y) / 4.0 / node_radius * 3.6;
        d.floor() as usize
    }
}

// Non-wasm methods
impl Wire {
    /// Returns a reference to the nodes (for internal use by World)
    pub(crate) fn get_nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    fn intersect(&self, _point: &Point) -> bool {
        // TODO: implement proper collision detection with the wire
        true
    }

    fn diameter(&self) -> f32 {
        self.radius * 2.0
    }

}
