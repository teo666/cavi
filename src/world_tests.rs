use crate::world::WasmWorld as World;

/// The render buffer as a slice (native tests: the pointer is plain memory).
fn buffer(world: &World) -> Vec<f32> {
    let len = world.wire_data_len();
    if len == 0 {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(world.wire_data_ptr(), len) }.to_vec()
}

/// A world with one segments-rendered wire, so the buffer holds raw node positions.
fn world_with_wire() -> World {
    let mut world = World::new();
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 30.0, 4.0, 0);
    world
}

/// The node positions written to the buffer for wire 0 (segments render type).
fn drawn_points(world: &World) -> Vec<(f32, f32)> {
    let data = buffer(world);
    let path_len = data[3] as usize;
    data[4..4 + path_len].chunks(2).map(|c| (c[0], c[1])).collect()
}

#[test]
fn test_defaults_match_the_previous_hardcoded_values() {
    let world = World::new();
    assert_eq!(world.get_time_step(), 0.3);
    assert_eq!(world.get_substeps(), 2);
    assert_eq!(world.get_constraint_iterations(), 1);
    assert_eq!(world.get_render_alpha(), 1.0);
}

#[test]
fn test_setters_reject_invalid_values() {
    let mut world = World::new();
    world.set_time_step(0.0);
    world.set_time_step(f32::NAN);
    assert_eq!(world.get_time_step(), 0.3);
    world.set_substeps(0);
    assert_eq!(world.get_substeps(), 1);
    world.set_constraint_iterations(0);
    assert_eq!(world.get_constraint_iterations(), 1);
}

#[test]
fn test_new_wire_starts_straight_between_its_ends() {
    let world = world_with_wire();
    for i in 0..5 {
        assert!((world.get_wire_node_x(0, i) - i as f32 * 25.0).abs() < 1e-4);
        assert_eq!(world.get_wire_node_y(0, i), 0.0);
    }
    assert!(world.get_wire_node_fixed(0, 0));
    assert!(!world.get_wire_node_fixed(0, 2));
    assert!(world.get_wire_node_fixed(0, 4));
    assert!(!world.get_wire_node_fixed(7, 0)); // out of range
}

#[test]
fn test_update_is_step_then_build_render_data_at_one() {
    let mut a = world_with_wire();
    let mut b = world_with_wire();
    for _ in 0..10 {
        a.update();
        b.step();
        b.build_render_data(1.0);
    }
    assert_eq!(buffer(&a), buffer(&b));
}

#[test]
fn test_step_alone_leaves_the_render_buffer_untouched() {
    let mut world = world_with_wire();
    world.update();
    let before = buffer(&world);
    world.step();
    assert_eq!(buffer(&world), before);
}

#[test]
fn test_build_render_data_interpolates_movable_nodes_between_steps() {
    let mut world = world_with_wire(); // gravity pulls the middle down
    world.update();
    let before: Vec<(f32, f32)> = (0..5)
        .map(|i| (world.get_wire_node_x(0, i), world.get_wire_node_y(0, i)))
        .collect();
    world.step();
    let after: Vec<(f32, f32)> = (0..5)
        .map(|i| (world.get_wire_node_x(0, i), world.get_wire_node_y(0, i)))
        .collect();
    assert!(after[2].1 > before[2].1 + 0.1, "the middle node should have moved");

    world.build_render_data(0.0);
    let drawn = drawn_points(&world);
    assert_eq!(drawn[2], before[2]); // movable: the pre-step state
    assert_eq!(drawn[0], after[0]); // fixed: where it is now

    world.build_render_data(0.5);
    let drawn = drawn_points(&world);
    assert!((drawn[2].1 - (before[2].1 + after[2].1) / 2.0).abs() < 1e-4);
    assert!((world.get_wire_node_render_y(0, 2) - drawn[2].1).abs() < 1e-4);

    world.build_render_data(1.0);
    assert_eq!(drawn_points(&world)[2], after[2]);
}

#[test]
fn test_alpha_is_clamped_and_nan_draws_the_current_state() {
    let mut world = world_with_wire();
    world.update();
    world.step();
    world.build_render_data(f32::NAN);
    assert_eq!(world.get_render_alpha(), 1.0);
    world.build_render_data(7.0);
    assert_eq!(world.get_render_alpha(), 1.0);
    world.build_render_data(-1.0);
    assert_eq!(world.get_render_alpha(), 0.0);
}

#[test]
fn test_a_wire_whose_node_count_changed_is_drawn_as_it_is_now() {
    let mut world = world_with_wire();
    world.update();
    world.step();
    world.add_wire_node_at(0, 2, 50.0, 40.0, false);
    world.build_render_data(0.0);
    let drawn = drawn_points(&world);
    assert_eq!(drawn.len(), 6);
    assert_eq!(drawn[2], (50.0, 40.0));
}

#[test]
fn test_build_render_data_covers_a_wire_added_since_the_last_step() {
    let mut world = world_with_wire();
    world.update();
    world.add_wire_with_count(0.0, 50.0, 100.0, 50.0, 4, 30.0, 4.0, 0);
    world.build_render_data(0.0);
    let data = buffer(&world);
    // wire 0: 4 header values + 10 coords; wire 1: 4 header values + 8 coords
    assert_eq!(data.len(), 4 + 10 + 4 + 8);
}

#[test]
fn test_more_constraint_iterations_stretch_the_cable_less() {
    let stretch = |iterations: u32| {
        let mut world = World::new();
        world.set_constraint_iterations(iterations);
        world.set_acceleration(0.0, 60.0);
        // Rest length 4 * 20 = 80 between ends 80 apart: gravity stretches it.
        world.add_wire_with_count(0.0, 0.0, 80.0, 0.0, 5, 20.0, 2.0, 0);
        for _ in 0..200 {
            world.update();
        }
        let mut length = 0.0;
        for i in 1..5 {
            let dx = world.get_wire_node_x(0, i) - world.get_wire_node_x(0, i - 1);
            let dy = world.get_wire_node_y(0, i) - world.get_wire_node_y(0, i - 1);
            length += (dx * dx + dy * dy).sqrt();
        }
        length - 80.0
    };
    assert!(stretch(8) < stretch(1));
}
