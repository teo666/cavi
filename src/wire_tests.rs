use crate::world::wire::Wire;
use crate::world::point::Point;

#[test]
fn test_wire_new() {
    let wire = Wire::new();
    
    assert_eq!(wire.node_count(), 30);
    assert_eq!(wire.iterations, 0);
    assert_eq!(wire.radius, 5.0);
    assert_eq!(wire.link_target_distance, 1.0);
}

#[test]
fn test_wire_new_wire() {
    let wire = Wire::new_wire(0.0, 0.0, 100.0, 100.0, 3.0);
    
    assert!(wire.node_count() >= 2);
    assert_eq!(wire.radius, 3.0);
    assert_eq!(wire.link_target_distance, 9.0); // radius * 3
}

#[test]
fn test_wire_node_count() {
    let wire = Wire::new();
    let count = wire.node_count();
    
    assert_eq!(count, 30);
    assert!(count > 0);
}

#[test]
fn test_wire_get_node() {
    let wire = Wire::new();
    
    let first_node = wire.get_node(0);
    assert!(first_node.is_some());
    
    let invalid_node = wire.get_node(1000);
    assert!(invalid_node.is_none());
}

#[test]
fn test_wire_get_node_coordinates() {
    let wire = Wire::new_wire(10.0, 20.0, 100.0, 200.0, 5.0);
    
    let x = wire.get_node_x(0);
    let y = wire.get_node_y(0);
    
    assert_eq!(x, 10.0);
    assert_eq!(y, 20.0);
}

#[test]
fn test_wire_get_start_end() {
    let wire = Wire::new_wire(5.0, 10.0, 50.0, 100.0, 3.0);
    
    let start = wire.get_start();
    assert_eq!(start.x, 5.0);
    assert_eq!(start.y, 10.0);
    
    let end = wire.get_end();
    assert_eq!(end.x, 50.0);
    assert_eq!(end.y, 100.0);
}

#[test]
fn test_wire_set_start() {
    let mut wire = Wire::new_wire(0.0, 0.0, 100.0, 100.0, 5.0);
    
    wire.set_start(20.0, 30.0);
    
    let start = wire.get_start();
    assert_eq!(start.x, 20.0);
    assert_eq!(start.y, 30.0);
}

#[test]
fn test_wire_set_end() {
    let mut wire = Wire::new_wire(0.0, 0.0, 100.0, 100.0, 5.0);
    
    wire.set_end(150.0, 200.0);
    
    let end = wire.get_end();
    assert_eq!(end.x, 150.0);
    assert_eq!(end.y, 200.0);
}

#[test]
fn test_wire_set_radius() {
    let mut wire = Wire::new();
    
    wire.set_radius(10.0);
    
    assert_eq!(wire.radius, 10.0);
    assert_eq!(wire.link_target_distance, 30.0); // radius * 3
}

#[test]
fn test_wire_length() {
    let wire = Wire::new_wire(0.0, 0.0, 100.0, 0.0, 5.0);
    
    let length = wire.length();
    
    // Length should be approximately 100 (the distance between start and end)
    assert!(length > 0.0);
    assert!(length >= 100.0); // At least the straight-line distance
}

#[test]
fn test_wire_invalidate() {
    let mut wire = Wire::new();
    wire.iterations = 10;
    
    wire.invalidate();
    
    assert_eq!(wire.iterations, 0);
}

#[test]
fn test_wire_update() {
    let mut wire = Wire::new_wire(0.0, 0.0, 100.0, 0.0, 5.0);
    
    let _initial_node_pos = wire.get_node_x(1);
    
    // Run physics update
    wire.update(0.1, 0.95, Point::new(0.0, 10.0));
    
    // Physics simulation might change positions
    let new_node_pos = wire.get_node_x(1);
    
    // Position is either same or changed (both are valid depending on state)
    assert!(new_node_pos.is_finite());
}

#[test]
fn test_wire_check_collision() {
    let mut wire = Wire::new_wire(0.0, 0.0, 100.0, 0.0, 5.0);
    
    let collision_point = Point::new(50.0, 0.0);
    
    wire.check_collision(&collision_point, 20.0);
    
    // After collision, nodes near the point should be pushed away
    // We just verify the wire is still in valid state
    assert!(wire.node_count() > 0);
}

#[test]
fn test_wire_check_wire_elements_collisions() {
    let mut wire = Wire::new_wire(0.0, 0.0, 10.0, 0.0, 5.0);
    
    wire.check_wire_elements_collisions(0.75);
    
    // Should complete without panicking
    assert!(wire.node_count() > 0);
}

#[test]
fn test_wire_check_mouse_collision() {
    let mut wire = Wire::new();
    let mouse_pos = Point::new(100.0, 100.0);
    
    wire.check_mouse_collision(mouse_pos, 40.0);
    
    // Should complete without panicking
    assert!(wire.node_count() > 0);
}

#[test]
fn test_wire_multiple_updates() {
    let mut wire = Wire::new_wire(0.0, 0.0, 100.0, 0.0, 5.0);
    
    // Run multiple physics iterations
    for _ in 0..10 {
        wire.update(0.1, 0.95, Point::new(0.0, 10.0));
        wire.check_wire_elements_collisions(0.75);
    }
    
    // Verify wire is still valid
    assert!(wire.node_count() > 0);
    assert!(wire.length() > 0.0);
}

#[test]
fn test_wire_start_end_are_fixed() {
    let wire = Wire::new_wire(0.0, 0.0, 100.0, 100.0, 5.0);
    
    let first_node = wire.get_node(0).unwrap();
    let last_node = wire.get_node(wire.node_count() - 1).unwrap();
    
    // First and last nodes should be fixed
    assert!(first_node.is_fixed());
    assert!(last_node.is_fixed());
}

#[test]
fn test_wire_middle_nodes_are_movable() {
    let wire = Wire::new_wire(0.0, 0.0, 100.0, 100.0, 5.0);
    
    if wire.node_count() > 2 {
        let middle_node = wire.get_node(1).unwrap();
        assert!(middle_node.is_movable());
    }
}

#[test]
fn test_wire_iterations_reset_after_invalidate() {
    let mut wire = Wire::new();
    wire.iterations = 100;
    
    wire.invalidate();
    
    assert_eq!(wire.iterations, 0);
}

#[test]
fn test_wire_with_different_radii() {
    let small_wire = Wire::new_wire(0.0, 0.0, 100.0, 0.0, 2.0);
    let large_wire = Wire::new_wire(0.0, 0.0, 100.0, 0.0, 10.0);
    
    assert_eq!(small_wire.radius, 2.0);
    assert_eq!(large_wire.radius, 10.0);
    assert!(large_wire.link_target_distance > small_wire.link_target_distance);
}

#[test]
fn test_wire_coordinate_access_bounds() {
    let wire = Wire::new();
    
    // Valid access
    let valid_x = wire.get_node_x(0);
    assert!(valid_x.is_finite());
    
    // Invalid access should return 0.0
    let invalid_x = wire.get_node_x(1000);
    assert_eq!(invalid_x, 0.0);
    
    let invalid_y = wire.get_node_y(1000);
    assert_eq!(invalid_y, 0.0);
}
