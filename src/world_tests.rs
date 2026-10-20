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

// --- Moving nodes from outside the simulation ---

/// Drags the last node of wire 0 (fixed) 15 units per step for 20 steps,
/// holds it still for `still_steps` steps, releases it, and returns the
/// velocity it carries (current - previous position, per sub-step).
fn release_after_drag(still_steps: usize) -> f32 {
    let mut world = World::new();
    world.set_mouse(-1e6, -1e6);
    world.set_acceleration(0.0, 0.0);
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 6, 20.0, 4.0, 0);
    world.update();
    for i in 1..=20 {
        world.set_wire_node_position(0, 5, 100.0 + i as f32 * 15.0, 0.0);
        world.update();
    }
    for _ in 0..still_steps {
        world.update();
    }
    world.set_wire_node_fixed(0, 5, false);
    let node = world.get_wire_node(0, 5).unwrap();
    assert!(!node.get_fixed());
    node.position.curr.x - node.position.old.x
}

#[test]
fn test_releasing_a_dragged_end_throws_it_with_its_last_movement_only() {
    // Before: the whole 300-unit drag counted as velocity. Now: the last
    // step's 15 units, over the default 2 sub-steps.
    assert!((release_after_drag(0) - 7.5).abs() < 1e-3);
}

#[test]
fn test_releasing_an_end_held_still_just_drops_it() {
    assert!(release_after_drag(3).abs() < 1e-3);
}

#[test]
fn test_moving_a_free_node_teleports_it_without_velocity() {
    let mut world = World::new();
    world.set_mouse(-1e6, -1e6);
    world.set_acceleration(0.0, 0.0);
    world.set_sleep_enabled(false);
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 3, 50.0, 4.0, 0);
    world.update();
    world.set_wire_node_fixed(0, 2, false);
    world.set_wire_node_fixed(0, 0, false);
    // Move the whole (free) wire far away: no node may keep flying afterwards.
    for i in 0..3 {
        let x = world.get_wire_node_x(0, i);
        world.set_wire_node_position(0, i, x + 1000.0, 500.0);
    }
    let before: Vec<f32> = (0..3).map(|i| world.get_wire_node_x(0, i)).collect();
    world.update();
    for i in 0..3 {
        assert!((world.get_wire_node_x(0, i) - before[i]).abs() < 1.0);
    }
}

// --- Held wire and pointer_radius ---

/// Distance between the dragged end of wire 0 and its neighbour, after
/// dragging it to (200, 150) with the mouse on it for 120 steps.
fn drag_end_with_mouse(hold: bool) -> f32 {
    let mut world = World::new();
    world.add_wire_with_count(0.0, 0.0, 200.0, 0.0, 11, 20.0, 4.0, 1);
    if hold {
        world.set_held_wire(0);
    }
    for _ in 0..120 {
        world.set_wire_node_position(0, 10, 200.0, 150.0);
        world.set_mouse(200.0, 150.0);
        world.update();
    }
    let dx = world.get_wire_node_x(0, 10) - world.get_wire_node_x(0, 9);
    let dy = world.get_wire_node_y(0, 10) - world.get_wire_node_y(0, 9);
    (dx * dx + dy * dy).sqrt()
}

#[test]
fn test_a_held_wire_is_not_pushed_by_the_mouse_on_its_own_end() {
    // Unheld, the mouse (radius 40) pushes the end's neighbour out to ~44.
    assert!(drag_end_with_mouse(false) > 30.0);
    // Held, the link stays near its rest length (20) plus gravity stretch.
    assert!(drag_end_with_mouse(true) < 27.0);
}

#[test]
fn test_while_holding_other_wires_are_pushed_within_pointer_radius() {
    // How far the other wire's middle node ends up from a mouse right above it.
    let pushed = |hold: bool, pointer_radius: f32| {
        let mut world = World::new();
        world.set_acceleration(0.0, 0.0);
        world.set_pointer_radius(pointer_radius);
        world.add_wire_with_count(0.0, -300.0, 100.0, -300.0, 3, 50.0, 4.0, 0); // held
        world.add_wire_with_count(-100.0, 0.0, 100.0, 0.0, 3, 100.0, 4.0, 0); // other
        if hold {
            world.set_held_wire(0);
        }
        world.set_mouse(0.0, -1.0);
        world.step();
        world.get_wire_node_y(1, 1) + 1.0
    };
    let by_mouse = pushed(false, 20.0); // mouse_radius 40
    assert!(pushed(true, 20.0) > 0.0);
    assert!(pushed(true, 20.0) < by_mouse);
    assert!(pushed(true, 80.0) > by_mouse);
    // Not holding: pointer_radius plays no part.
    assert_eq!(pushed(false, 80.0), by_mouse);
}

#[test]
fn test_the_held_wire_index_follows_deletions() {
    let mut world = World::new();
    for i in 0..3 {
        world.add_wire_with_count(0.0, i as f32 * 50.0, 100.0, i as f32 * 50.0, 3, 50.0, 4.0, 0);
    }
    world.set_held_wire(2);
    world.delete_wire(0);
    assert_eq!(world.get_held_wire(), Some(1));
    world.delete_wire(1);
    assert_eq!(world.get_held_wire(), None);
    world.set_held_wire(7); // out of range: ignored
    assert_eq!(world.get_held_wire(), None);
}

// --- Sleeping ---

/// A straight, gravity-free wire at rest: still from the first step.
fn resting_world() -> World {
    let mut world = World::new();
    world.set_mouse(-1e6, -1e6);
    world.set_acceleration(0.0, 0.0);
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 25.0, 4.0, 0);
    world
}

#[test]
fn test_a_still_wire_falls_asleep_after_sleep_steps() {
    let mut world = resting_world();
    for _ in 0..29 {
        world.update();
    }
    assert!(!world.is_wire_sleeping(0));
    world.update();
    assert!(world.is_wire_sleeping(0));
    assert_eq!(world.sleeping_wire_count(), 1);
}

#[test]
fn test_a_moving_wire_stays_awake() {
    let mut world = World::new(); // gravity on: the wire sags for a while
    world.set_mouse(-1e6, -1e6);
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 30.0, 4.0, 0);
    for _ in 0..30 {
        world.update();
    }
    assert!(!world.is_wire_sleeping(0));
}

#[test]
fn test_a_sleeping_wire_wakes_up_on_changes() {
    let wake = |change: &dyn Fn(&mut World)| {
        let mut world = resting_world();
        for _ in 0..40 {
            world.update();
        }
        assert!(world.is_wire_sleeping(0));
        change(&mut world);
        world.step();
        !world.is_wire_sleeping(0)
    };
    assert!(wake(&|w| w.set_acceleration(0.0, 10.0)));
    assert!(wake(&|w| w.set_friction(0.9)));
    assert!(wake(&|w| w.set_wire_node_fixed(0, 4, false)));
    assert!(wake(&|w| w.set_wire_node_position(0, 0, 0.0, 20.0)));
    assert!(wake(&|w| w.add_wire_node_at(0, 2, 50.0, 0.0, false)));
    assert!(wake(&|w| w.set_wire_link_distance(0, 30.0)));
    assert!(wake(&|w| w.set_mouse(50.0, 10.0))); // within reach
    assert!(!wake(&|w| w.set_mouse(50.0, 500.0))); // far away
    assert!(wake(&|w| w.set_sleep_enabled(false)));
}

#[test]
fn test_with_sleeping_off_no_wire_sleeps() {
    let mut world = resting_world();
    world.set_sleep_enabled(false);
    for _ in 0..100 {
        world.update();
    }
    assert!(!world.is_wire_sleeping(0));
}

#[test]
fn test_sleep_settings_are_configurable() {
    let mut world = resting_world();
    assert!(world.get_sleep_enabled());
    assert_eq!(world.get_sleep_threshold(), 0.01);
    assert_eq!(world.get_sleep_steps(), 30);
    world.set_sleep_steps(5);
    for _ in 0..5 {
        world.update();
    }
    assert!(world.is_wire_sleeping(0));
    world.set_sleep_steps(0);
    assert_eq!(world.get_sleep_steps(), 1);
    world.set_sleep_threshold(-1.0); // ignored
    assert_eq!(world.get_sleep_threshold(), 0.01);
}

#[test]
fn test_a_sleeping_wire_is_still_drawn() {
    let mut world = resting_world();
    for _ in 0..40 {
        world.update();
    }
    assert!(world.is_wire_sleeping(0));
    world.build_render_data(0.5);
    assert_eq!(drawn_points(&world).len(), 5);
}

#[test]
fn test_the_render_type_of_a_wire_can_be_changed() {
    let mut world = world_with_wire(); // segments
    world.update();
    assert_eq!(world.get_wire_render_type(0), 0);
    let segments = buffer(&world)[3];
    world.set_wire_render_type(0, 1);
    world.build_render_data(1.0);
    assert_eq!(world.get_wire_render_type(0), 1);
    assert!(buffer(&world)[3] > segments); // bezier: 3 points per segment
}

// Bend stiffness and end directions (WasmWire::relax_bends).

/// A world with no gravity and the mouse out of reach.
fn still_world() -> World {
    let mut world = World::new();
    world.set_mouse(-1e6, -1e6);
    world.set_acceleration(0.0, 0.0);
    world
}

/// A world with three nodes, the middle one lifted to (50, 20): links of
/// exactly that length, so only bending can move it.
fn bent_world(stiffness: f32) -> World {
    let mut world = still_world();
    let link = (50.0f32).hypot(20.0);
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 3, link, 4.0, 0);
    world.set_wire_node_position(0, 1, 50.0, 20.0);
    world.set_wire_bend_stiffness(0, stiffness);
    world
}

/// The largest turn between consecutive links of wire 0, in radians.
fn sharpest_turn(world: &World) -> f32 {
    let n = world.get_wire_node_count(0);
    let p = |i: usize| (world.get_wire_node_x(0, i), world.get_wire_node_y(0, i));
    (1..n - 1)
        .map(|i| {
            let (a, b, c) = (p(i - 1), p(i), p(i + 1));
            let d1 = (b.1 - a.1).atan2(b.0 - a.0);
            let d2 = (c.1 - b.1).atan2(c.0 - b.0);
            let mut turn = (d2 - d1).abs();
            if turn > std::f32::consts::PI {
                turn = 2.0 * std::f32::consts::PI - turn;
            }
            turn
        })
        .fold(0.0, f32::max)
}

#[test]
fn test_bend_stiffness_defaults_to_zero_and_is_clamped() {
    let mut world = still_world();
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 25.0, 4.0, 0);
    assert_eq!(world.get_wire_bend_stiffness(0), 0.0);
    world.set_wire_bend_stiffness(0, 2.0);
    assert_eq!(world.get_wire_bend_stiffness(0), 1.0);
    world.set_wire_bend_stiffness(0, -1.0);
    assert_eq!(world.get_wire_bend_stiffness(0), 0.0);
    world.set_wire_bend_stiffness(0, 0.3);
    world.set_wire_bend_stiffness(0, f32::NAN); // ignored
    assert_eq!(world.get_wire_bend_stiffness(0), 0.3);
    assert_eq!(world.get_wire_bend_stiffness(9), 0.0); // out of range
}

#[test]
fn test_without_bend_stiffness_a_bent_wire_stays_bent() {
    let mut world = bent_world(0.0);
    world.update();
    assert!((world.get_wire_node_y(0, 1) - 20.0).abs() < 1e-3);
}

#[test]
fn test_bend_stiffness_straightens_a_bent_wire() {
    let mut world = bent_world(0.5);
    world.update();
    let y = world.get_wire_node_y(0, 1);
    assert!(y < 19.0, "middle node at y = {}", y);
}

#[test]
fn test_fixed_nodes_never_move_however_stiff() {
    let mut world = bent_world(1.0);
    for _ in 0..50 {
        world.update();
    }
    assert_eq!((world.get_wire_node_x(0, 0), world.get_wire_node_y(0, 0)), (0.0, 0.0));
    assert_eq!((world.get_wire_node_x(0, 2), world.get_wire_node_y(0, 2)), (100.0, 0.0));
}

#[test]
fn test_a_stiff_hanging_wire_curves_instead_of_folding() {
    // 240 of rope from (0, 0) down to (40, 200): a chain drops straight
    // down like a plumb line and folds where the rope runs out; a cable
    // curves all along instead.
    let hang = |stiffness: f32| {
        let mut world = World::new();
        world.set_mouse(-1e6, -1e6);
        world.set_sleep_enabled(false);
        world.set_acceleration(0.0, 5.0);
        world.set_constraint_iterations(4);
        world.add_wire_with_count(0.0, 0.0, 40.0, 200.0, 13, 20.0, 4.0, 0);
        world.set_wire_bend_stiffness(0, stiffness);
        for _ in 0..600 {
            world.update();
        }
        sharpest_turn(&world)
    };
    let chain = hang(0.0);
    let cable = hang(0.3);
    assert!(chain > 1.5, "the chain folds: sharpest turn {}", chain);
    assert!(cable < 0.6, "the cable curves: sharpest turn {}", cable);
}

#[test]
fn test_an_end_direction_pulls_the_cable_out_along_it() {
    let straight = |direction: Option<(f32, f32)>| {
        let mut world = still_world();
        world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 25.0, 4.0, 0);
        world.set_wire_bend_stiffness(0, 0.5);
        if let Some((dx, dy)) = direction {
            world.set_wire_end_direction(0, 0, dx, dy);
            world.set_wire_end_direction(0, 1, dx, dy);
        }
        for _ in 0..20 {
            world.update();
        }
        (world.get_wire_node_y(0, 1), world.get_wire_node_y(0, 3))
    };
    assert_eq!(straight(None), (0.0, 0.0));
    // Both ends leave downwards: the nodes next to them go down.
    let (first, last) = straight(Some((0.0, 1.0)));
    assert!(first > 1.0 && last > 1.0, "nodes next to the ends at y = {}, {}", first, last);
}

#[test]
fn test_an_end_direction_is_normalized_and_can_be_cleared() {
    let mut world = still_world();
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 25.0, 4.0, 0);
    world.set_wire_end_direction(0, 1, 0.0, -3.0);
    assert_eq!((world.get_wire_end_direction_x(0, 1), world.get_wire_end_direction_y(0, 1)), (0.0, -1.0));
    assert_eq!((world.get_wire_end_direction_x(0, 0), world.get_wire_end_direction_y(0, 0)), (0.0, 0.0));
    world.set_wire_end_direction(0, 1, 0.0, 0.0);
    assert_eq!(world.get_wire_end_direction_y(0, 1), 0.0);
}

#[test]
fn test_bend_settings_survive_resizing_the_wire() {
    let mut world = still_world();
    world.add_wire_with_count(0.0, 0.0, 100.0, 0.0, 5, 25.0, 4.0, 0);
    world.set_wire_bend_stiffness(0, 0.4);
    world.set_wire_end_direction(0, 0, 1.0, 0.0);
    world.set_wire_node_count(0, 9);
    world.add_wire_node_at(0, 4, 50.0, 0.0, false);
    world.remove_wire_node(0, 2);
    assert_eq!(world.get_wire_bend_stiffness(0), 0.4);
    assert_eq!(world.get_wire_end_direction_x(0, 0), 1.0);
}

#[test]
fn test_changing_bend_settings_wakes_a_sleeping_wire() {
    let mut world = resting_world();
    for _ in 0..40 {
        world.update();
    }
    assert!(world.is_wire_sleeping(0));
    world.set_wire_bend_stiffness(0, 0.5);
    assert!(!world.is_wire_sleeping(0));

    for _ in 0..40 {
        world.update();
    }
    assert!(world.is_wire_sleeping(0));
    world.set_wire_end_direction(0, 0, 0.0, 1.0);
    assert!(!world.is_wire_sleeping(0));
}

