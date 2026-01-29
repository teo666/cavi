use crate::world::point::Point;

#[test]
fn test_point_new() {
    let p = Point::new(3.0, 4.0);
    assert_eq!(p.x, 3.0);
    assert_eq!(p.y, 4.0);
}

#[test]
fn test_point_add() {
    let p1 = Point::new(1.0, 2.0);
    let p2 = Point::new(3.0, 4.0);
    let result = p1 + p2;
    assert_eq!(result.x, 4.0);
    assert_eq!(result.y, 6.0);
}

#[test]
fn test_point_sub() {
    let p1 = Point::new(5.0, 7.0);
    let p2 = Point::new(2.0, 3.0);
    let result = p1 - p2;
    assert_eq!(result.x, 3.0);
    assert_eq!(result.y, 4.0);
}

#[test]
fn test_point_mul() {
    let p = Point::new(2.0, 3.0);
    let result = p * 2.5;
    assert_eq!(result.x, 5.0);
    assert_eq!(result.y, 7.5);
}

#[test]
fn test_point_div() {
    let p = Point::new(10.0, 20.0);
    let result = p / 2.0;
    assert_eq!(result.x, 5.0);
    assert_eq!(result.y, 10.0);
}

#[test]
fn test_point_add_assign() {
    let mut p1 = Point::new(1.0, 2.0);
    let p2 = Point::new(3.0, 4.0);
    p1 += p2;
    assert_eq!(p1.x, 4.0);
    assert_eq!(p1.y, 6.0);
}

#[test]
fn test_point_sub_assign() {
    let mut p1 = Point::new(10.0, 15.0);
    let p2 = Point::new(3.0, 5.0);
    p1 -= p2;
    assert_eq!(p1.x, 7.0);
    assert_eq!(p1.y, 10.0);
}

#[test]
fn test_point_display() {
    let p = Point::new(3.5, 4.2);
    let display_string = format!("{}", p);
    assert_eq!(display_string, "x: 3.5 y: 4.2");
}

#[test]
fn test_point_copy_clone() {
    let p1 = Point::new(1.0, 2.0);
    let p2 = p1; // Copy
    let p3 = p1.clone(); // Clone
    
    assert_eq!(p1.x, p2.x);
    assert_eq!(p1.y, p2.y);
    assert_eq!(p1.x, p3.x);
    assert_eq!(p1.y, p3.y);
}

#[test]
fn test_point_negative_values() {
    let p1 = Point::new(-5.0, -10.0);
    let p2 = Point::new(3.0, 7.0);
    let result = p1 + p2;
    assert_eq!(result.x, -2.0);
    assert_eq!(result.y, -3.0);
}

#[test]
fn test_point_zero() {
    let p = Point::new(0.0, 0.0);
    assert_eq!(p.x, 0.0);
    assert_eq!(p.y, 0.0);
}

#[test]
fn test_point_chained_operations() {
    let p1 = Point::new(2.0, 3.0);
    let p2 = Point::new(1.0, 1.0);
    let result = (p1 + p2) * 2.0 - Point::new(1.0, 1.0);
    assert_eq!(result.x, 5.0);
    assert_eq!(result.y, 7.0);
}

// Utility methods tests
#[test]
fn test_point_length() {
    let p = Point::new(3.0, 4.0);
    assert_eq!(p.length(), 5.0);
    
    let p2 = Point::new(0.0, 0.0);
    assert_eq!(p2.length(), 0.0);
}

#[test]
fn test_point_length_squared() {
    let p = Point::new(3.0, 4.0);
    assert_eq!(p.length_squared(), 25.0);
}

#[test]
fn test_point_normalized() {
    let p = Point::new(3.0, 4.0);
    let normalized = p.normalized();
    assert_eq!(normalized.x, 0.6);
    assert_eq!(normalized.y, 0.8);
    assert!((normalized.length() - 1.0).abs() < 0.0001);
}

#[test]
fn test_point_normalize() {
    let mut p = Point::new(3.0, 4.0);
    p.normalize();
    assert_eq!(p.x, 0.6);
    assert_eq!(p.y, 0.8);
    assert!((p.length() - 1.0).abs() < 0.0001);
}

#[test]
fn test_point_normalize_zero() {
    let mut p = Point::new(0.0, 0.0);
    p.normalize();
    assert_eq!(p.x, 0.0);
    assert_eq!(p.y, 0.0);
}

#[test]
fn test_point_distance_to() {
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(3.0, 4.0);
    assert_eq!(p1.distance_to(&p2), 5.0);
}

#[test]
fn test_point_distance_squared_to() {
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(3.0, 4.0);
    assert_eq!(p1.distance_squared_to(&p2), 25.0);
}

#[test]
fn test_point_dot() {
    let p1 = Point::new(2.0, 3.0);
    let p2 = Point::new(4.0, 5.0);
    assert_eq!(p1.dot(&p2), 23.0); // 2*4 + 3*5 = 8 + 15 = 23
}

#[test]
fn test_point_cross() {
    let p1 = Point::new(2.0, 3.0);
    let p2 = Point::new(4.0, 5.0);
    assert_eq!(p1.cross(&p2), -2.0); // 2*5 - 3*4 = 10 - 12 = -2
}

#[test]
fn test_point_angle() {
    let p = Point::new(1.0, 0.0);
    assert_eq!(p.angle(), 0.0);
    
    let p2 = Point::new(0.0, 1.0);
    assert!((p2.angle() - std::f32::consts::PI / 2.0).abs() < 0.0001);
}

#[test]
fn test_point_angle_to() {
    let p1 = Point::new(1.0, 0.0);
    let p2 = Point::new(0.0, 1.0);
    let angle = p1.angle_to(&p2);
    assert!((angle - std::f32::consts::PI / 2.0).abs() < 0.0001);
}

#[test]
fn test_point_rotated() {
    let p = Point::new(1.0, 0.0);
    let rotated = p.rotated(std::f32::consts::PI / 2.0); // 90 degrees
    assert!((rotated.x - 0.0).abs() < 0.0001);
    assert!((rotated.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_point_rotate() {
    let mut p = Point::new(1.0, 0.0);
    p.rotate(std::f32::consts::PI / 2.0); // 90 degrees
    assert!((p.x - 0.0).abs() < 0.0001);
    assert!((p.y - 1.0).abs() < 0.0001);
}

#[test]
fn test_point_lerp() {
    let p1 = Point::new(0.0, 0.0);
    let p2 = Point::new(10.0, 20.0);
    
    let mid = p1.lerp(&p2, 0.5);
    assert_eq!(mid.x, 5.0);
    assert_eq!(mid.y, 10.0);
    
    let start = p1.lerp(&p2, 0.0);
    assert_eq!(start.x, 0.0);
    assert_eq!(start.y, 0.0);
    
    let end = p1.lerp(&p2, 1.0);
    assert_eq!(end.x, 10.0);
    assert_eq!(end.y, 20.0);
}

#[test]
fn test_point_perpendicular() {
    let p = Point::new(3.0, 4.0);
    let perp = p.perpendicular();
    assert_eq!(perp.x, -4.0);
    assert_eq!(perp.y, 3.0);
    
    // Perpendicular vectors should have dot product of 0
    assert_eq!(p.dot(&perp), 0.0);
}

#[test]
fn test_point_reflect() {
    let p = Point::new(1.0, -1.0);
    let normal = Point::new(0.0, 1.0);
    let reflected = p.reflect(&normal);
    assert_eq!(reflected.x, 1.0);
    assert_eq!(reflected.y, 1.0);
}

#[test]
fn test_point_clamped() {
    let p = Point::new(3.0, 4.0); // length = 5
    let clamped = p.clamped(3.0);
    assert!((clamped.length() - 3.0).abs() < 0.0001);
    
    let p2 = Point::new(1.0, 1.0); // length < 3
    let clamped2 = p2.clamped(3.0);
    assert_eq!(clamped2.x, 1.0);
    assert_eq!(clamped2.y, 1.0);
}

#[test]
fn test_point_approx_equal() {
    let p1 = Point::new(1.0, 2.0);
    let p2 = Point::new(1.0001, 2.0001);
    
    assert!(p1.approx_equal(&p2, 0.001));
    assert!(!p1.approx_equal(&p2, 0.00001));
}

#[test]
fn test_point_zero_constructor() {
    let p = Point::zero();
    assert_eq!(p.x, 0.0);
    assert_eq!(p.y, 0.0);
}

#[test]
fn test_point_one() {
    let p = Point::one();
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 1.0);
}

#[test]
fn test_point_unit_x() {
    let p = Point::unit_x();
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 0.0);
}

#[test]
fn test_point_unit_y() {
    let p = Point::unit_y();
    assert_eq!(p.x, 0.0);
    assert_eq!(p.y, 1.0);
}
