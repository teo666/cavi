use std::fmt;
use std::ops::{Add, Sub, Mul,Div, SubAssign, AddAssign};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[wasm_bindgen] 
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x: x,
            y: y,
        }
    }

    /// Returns the length (magnitude) of the vector
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns the squared length (avoids sqrt for performance)
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns a normalized version of this point (unit vector)
    pub fn normalized(&self) -> Point {
        let len = self.length();
        if len > 0.0 {
            Point {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Point { x: 0.0, y: 0.0 }
        }
    }

    /// Normalizes this point in place
    pub fn normalize(&mut self) {
        let len = self.length();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
        }
    }

    /// Returns the distance to another point
    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the squared distance to another point (avoids sqrt)
    pub fn distance_squared_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Returns the dot product with another point
    pub fn dot(&self, other: &Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the cross product (z component) with another point
    pub fn cross(&self, other: &Point) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Returns the angle of this vector in radians
    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Returns the angle to another point in radians
    pub fn angle_to(&self, other: &Point) -> f32 {
        let dot = self.dot(other);
        let det = self.cross(other);
        det.atan2(dot)
    }

    /// Returns a point rotated by the given angle (in radians)
    pub fn rotated(&self, angle: f32) -> Point {
        let cos = angle.cos();
        let sin = angle.sin();
        Point {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Rotates this point in place by the given angle (in radians)
    pub fn rotate(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();
        let new_x = self.x * cos - self.y * sin;
        let new_y = self.x * sin + self.y * cos;
        self.x = new_x;
        self.y = new_y;
    }

    /// Linear interpolation between this point and another
    pub fn lerp(&self, other: &Point, t: f32) -> Point {
        Point {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Returns a perpendicular vector (rotated 90 degrees CCW)
    pub fn perpendicular(&self) -> Point {
        Point {
            x: -self.y,
            y: self.x,
        }
    }

    /// Reflects this vector across a normal vector
    pub fn reflect(&self, normal: &Point) -> Point {
        let dot = self.dot(normal);
        Point {
            x: self.x - 2.0 * dot * normal.x,
            y: self.y - 2.0 * dot * normal.y,
        }
    }

    /// Clamps the length of the vector to a maximum value
    pub fn clamped(&self, max_length: f32) -> Point {
        let len_sq = self.length_squared();
        if len_sq > max_length * max_length {
            let len = len_sq.sqrt();
            Point {
                x: self.x * max_length / len,
                y: self.y * max_length / len,
            }
        } else {
            *self
        }
    }

    /// Returns true if this point is approximately equal to another
    pub fn approx_equal(&self, other: &Point, epsilon: f32) -> bool {
        (self.x - other.x).abs() < epsilon && (self.y - other.y).abs() < epsilon
    }

    /// Returns a zero point (0, 0)
    pub fn zero() -> Point {
        Point { x: 0.0, y: 0.0 }
    }

    /// Returns a point with both components set to 1
    pub fn one() -> Point {
        Point { x: 1.0, y: 1.0 }
    }

    /// Returns a unit point in the X direction (1, 0)
    pub fn unit_x() -> Point {
        Point { x: 1.0, y: 0.0 }
    }

    /// Returns a unit point in the Y direction (0, 1)
    pub fn unit_y() -> Point {
        Point { x: 0.0, y: 1.0 }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, scalar: f32) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for Point {
    type Output = Point;
    fn div(self, scalar: f32) -> Point {
        Point {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}



impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x: {} y: {}", self.x, self.y)?;
        Ok(())
    }
}
