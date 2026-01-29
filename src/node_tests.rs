use crate::world::node::Node;
use crate::world::point::Point;
use crate::world::position::Position;

#[test]
fn test_node_new() {
    let pos = Position::new(5.0, 10.0);
    let vel = Point::new(1.0, 2.0);
    let node = Node::new(pos, vel, false);
    
    assert_eq!(node.get_x(), 5.0);
    assert_eq!(node.get_y(), 10.0);
    assert_eq!(node.get_velocity_x(), 1.0);
    assert_eq!(node.get_velocity_y(), 2.0);
    assert_eq!(node.get_fixed(), false);
}

#[test]
fn test_node_new_no_vel() {
    let node = Node::new_no_vel(3.0, 4.0, false);
    
    assert_eq!(node.get_x(), 3.0);
    assert_eq!(node.get_y(), 4.0);
    assert_eq!(node.get_velocity_x(), 0.0);
    assert_eq!(node.get_velocity_y(), 0.0);
    assert_eq!(node.get_fixed(), false);
}

#[test]
fn test_node_zero() {
    let node = Node::zero();
    
    assert_eq!(node.get_x(), 0.0);
    assert_eq!(node.get_y(), 0.0);
    assert_eq!(node.get_velocity_x(), 0.0);
    assert_eq!(node.get_velocity_y(), 0.0);
    assert_eq!(node.get_fixed(), false);
}

#[test]
fn test_node_new_fixed() {
    let node = Node::new_fixed(10.0, 20.0);
    
    assert_eq!(node.get_x(), 10.0);
    assert_eq!(node.get_y(), 20.0);
    assert_eq!(node.get_fixed(), true);
    assert!(node.is_fixed());
    assert!(!node.is_movable());
}

#[test]
fn test_node_set_position() {
    let mut node = Node::zero();
    node.set_position(15.0, 25.0);
    
    assert_eq!(node.get_x(), 15.0);
    assert_eq!(node.get_y(), 25.0);
}

#[test]
fn test_node_set_velocity() {
    let mut node = Node::zero();
    node.set_velocity(5.0, 10.0);
    
    assert_eq!(node.get_velocity_x(), 5.0);
    assert_eq!(node.get_velocity_y(), 10.0);
}

#[test]
fn test_node_set_fixed() {
    let mut node = Node::zero();
    assert!(node.is_movable());
    
    node.set_fixed(true);
    assert_eq!(node.get_fixed(), true);
    assert!(node.is_fixed());
}

#[test]
fn test_node_make_fixed() {
    let mut node = Node::zero();
    node.make_fixed();
    
    assert_eq!(node.get_fixed(), true);
    assert!(node.is_fixed());
    assert!(!node.is_movable());
}

#[test]
fn test_node_make_movable() {
    let mut node = Node::new_fixed(0.0, 0.0);
    assert!(node.is_fixed());
    
    node.make_movable();
    assert_eq!(node.get_fixed(), false);
    assert!(node.is_movable());
    assert!(!node.is_fixed());
}

#[test]
fn test_node_get_position() {
    let node = Node::new_no_vel(7.0, 8.0, false);
    let pos = node.get_position();
    
    assert_eq!(pos.x, 7.0);
    assert_eq!(pos.y, 8.0);
}

#[test]
fn test_node_get_velocity() {
    let node = Node::new(
        Position::new(0.0, 0.0),
        Point::new(3.0, 4.0),
        false
    );
    let vel = node.get_velocity();
    
    assert_eq!(vel.x, 3.0);
    assert_eq!(vel.y, 4.0);
}

#[test]
fn test_node_get_speed() {
    let node = Node::new(
        Position::new(0.0, 0.0),
        Point::new(3.0, 4.0),
        false
    );
    
    assert_eq!(node.get_speed(), 5.0);
}

#[test]
fn test_node_apply_force() {
    let mut node = Node::zero();
    node.apply_force(2.0, 3.0);
    
    assert_eq!(node.get_velocity_x(), 2.0);
    assert_eq!(node.get_velocity_y(), 3.0);
}

#[test]
fn test_node_apply_force_accumulates() {
    let mut node = Node::zero();
    node.apply_force(1.0, 1.0);
    node.apply_force(2.0, 3.0);
    
    assert_eq!(node.get_velocity_x(), 3.0);
    assert_eq!(node.get_velocity_y(), 4.0);
}

#[test]
fn test_node_apply_force_fixed_node() {
    let mut node = Node::new_fixed(0.0, 0.0);
    node.apply_force(5.0, 10.0);
    
    // Fixed nodes should not be affected by forces
    assert_eq!(node.get_velocity_x(), 0.0);
    assert_eq!(node.get_velocity_y(), 0.0);
}

#[test]
fn test_node_apply_force_point() {
    let mut node = Node::zero();
    let force = Point::new(4.0, 5.0);
    node.apply_force_point(force);
    
    assert_eq!(node.get_velocity_x(), 4.0);
    assert_eq!(node.get_velocity_y(), 5.0);
}

#[test]
fn test_node_translate() {
    let mut node = Node::new_no_vel(5.0, 10.0, false);
    node.translate(3.0, 4.0);
    
    assert_eq!(node.get_x(), 8.0);
    assert_eq!(node.get_y(), 14.0);
}

#[test]
fn test_node_reset_velocity() {
    let mut node = Node::new(
        Position::new(5.0, 5.0),
        Point::new(10.0, 10.0),
        false
    );
    
    node.reset_velocity();
    
    assert_eq!(node.get_velocity_x(), 0.0);
    assert_eq!(node.get_velocity_y(), 0.0);
}

#[test]
fn test_node_distance_to() {
    let node1 = Node::new_no_vel(0.0, 0.0, false);
    let node2 = Node::new_no_vel(3.0, 4.0, false);
    
    assert_eq!(node1.distance_to(&node2), 5.0);
    assert_eq!(node2.distance_to(&node1), 5.0);
}

#[test]
fn test_node_distance_squared_to() {
    let node1 = Node::new_no_vel(0.0, 0.0, false);
    let node2 = Node::new_no_vel(3.0, 4.0, false);
    
    assert_eq!(node1.distance_squared_to(&node2), 25.0);
}

#[test]
fn test_node_update_with_acceleration() {
    let mut node = Node::new_no_vel(0.0, 0.0, false);
    let acceleration = Point::new(0.0, 10.0); // gravity-like
    let dt = 0.1;
    let friction = 1.0;
    
    node.update_with_acceleration(acceleration, dt, friction);
    
    // After integration, position should have changed
    assert!(node.get_y() > 0.0);
}

#[test]
fn test_node_update_with_acceleration_fixed() {
    let mut node = Node::new_fixed(5.0, 5.0);
    let acceleration = Point::new(0.0, 10.0);
    let dt = 0.1;
    let friction = 1.0;
    
    node.update_with_acceleration(acceleration, dt, friction);
    
    // Fixed node should not move
    assert_eq!(node.get_x(), 5.0);
    assert_eq!(node.get_y(), 5.0);
}

#[test]
fn test_node_is_fixed_is_movable() {
    let movable_node = Node::zero();
    assert!(movable_node.is_movable());
    assert!(!movable_node.is_fixed());
    
    let fixed_node = Node::new_fixed(0.0, 0.0);
    assert!(fixed_node.is_fixed());
    assert!(!fixed_node.is_movable());
}

#[test]
fn test_node_print() {
    let node = Node::new_no_vel(1.0, 2.0, false);
    let output = node.print();
    
    assert!(output.contains("position"));
    assert!(output.contains("velocity"));
    assert!(output.contains("fixed"));
}

#[test]
fn test_node_copy_clone() {
    let node1 = Node::new_no_vel(5.0, 10.0, false);
    let node2 = node1; // Copy
    let node3 = node1.clone(); // Clone
    
    assert_eq!(node1.get_x(), node2.get_x());
    assert_eq!(node1.get_y(), node2.get_y());
    assert_eq!(node1.get_x(), node3.get_x());
    assert_eq!(node1.get_y(), node3.get_y());
}
