use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::AddAssign;

use std::cmp::min;
use std::cmp::max;
use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::PartialEq;
use std::cmp::Ordering;



#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn zero() -> Vector2 {
        Vector2 {
            x: 0.0f32,
            y: 0.0f32
        }
    }

    // TODO(erick): Compare with epsilon
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    pub fn new(x0: f32, y0: f32) -> Vector2 {
        Vector2{
            x: x0,
            y: y0
        }
    }

    pub fn normalize_or_zero(&mut self) {
        let denom = (self.x * self.x + self.y * self.y).sqrt();

        if denom != 0.0f32 {
            self.x /= denom;
            self.y /= denom;
        } else {
            // if denom is zero the vector is already zero.
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f32) -> Vector2 {
        Vector2 {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Vector2) -> Vector2 {

        let result = Vector2 {
            x : self.x - rhs.x,
            y : self.y - rhs.y,
        };

        result
    }
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Vector2) -> Vector2 {

        let result = Vector2 {
            x : self.x + rhs.x,
            y : self.y + rhs.y,
        };

        result
    }
}


impl Div<f32> for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: f32) -> Vector2 {
        let result = Vector2 {
            x : self.x / rhs,
            y : self.y / rhs,
        };

        result
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<'a> AddAssign<&'a Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: &Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl PartialEq for Vector2 {
    // TODO(erick): This implementation is no different
    // from the default implementation (derive). We should
    // use deltas here.
    fn eq(&self, other: &Vector2) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Vector2 {}

// NOTE(erick): We could simply derive this trait
// since the default behavior is to do a lexicographic
// comparison.
impl Ord for Vector2 {
    fn cmp(&self, other: &Vector2) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }
        if self.x < other.x {
            return Ordering::Less;
        }
        if self.x > other.x {
            return Ordering::Greater;
        }

        if self.y < other.y {
            return Ordering::Less;
        }
        if self.y > other.y {
            return Ordering::Greater;
        }

        // Unreachable.
        Ordering::Equal
    }
}

impl PartialOrd for Vector2 {
    fn partial_cmp(&self, other: &Vector2) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
pub struct Rect2 {
    // NOTE(erick): (x0, y0) is always the left-bottom point
    // and (x1, y1) is always the right-top point.
    pub x0: f32,
    pub y0: f32,

    pub x1: f32,
    pub y1: f32,
}

impl Rect2 {
    pub fn from_point_and_dimensions(point: Vector2, width: f32, height: f32) -> Rect2 {
        Rect2 {
            x0: point.x,
            y0: point.y,

            x1: point.x + width,
            y1: point.y + height,
        }
    }

    pub fn from_points(p0: Vector2, p1: Vector2) -> Rect2 {
        Rect2 {
            x0: p0.x,
            y0: p0.y,

            x1: p1.x,
            y1: p1.y,
        }
    }

    pub fn bounding_rect(r0: &Rect2, r1: &Rect2) -> Rect2 {
        let p0_0 = r0.lower_left();
        let p0_1 = r1.lower_left();

        let p1_0 = r0.upper_right();
        let p1_1 = r1.upper_right();

        let result_p0 = min(p0_0, p0_1);
        let result_p1 = max(p1_0, p1_1);

        Rect2::from_points(result_p0, result_p1)
    }

    pub fn lower_left(&self) -> Vector2 {
        Vector2 {
            x: self.x0,
            y: self.y0,
        }
    }

    pub fn lower_right(&self) -> Vector2 {
        Vector2 {
            x: self.x1,
            y: self.y0,
        }
    }

    pub fn upper_left(&self) -> Vector2 {
        Vector2 {
            x: self.x0,
            y: self.y1,
        }
    }

    pub fn upper_right(&self) -> Vector2 {
        Vector2 {
            x: self.x1,
            y: self.y1,
        }
    }


    // TODO(erick): It would be nice if we had some unit-test for this thing.
    // NOTE(erick): This function returns None when no collision happened
    // or a right-handed Vector2 containing the wall otherwise (the vector
    // is not guaranteed to be normalized).
    pub fn collides_with(&self, other: &Rect2) -> Option<Vector2> {
        // TODO(erick): This does make sense. FIXME!!
        if self.x1 > other.x0 && self.x1 < other.x1 {
            if self.y1 <= other.y0 {
                return None;
            }
            if self.y0 >= other.y1 {
                return None;
            }
            // Collided from the right.
            return Some(other.lower_left() - other.upper_left());
        }
        if self.x0 < other.x1 && self.x0 > other.x0 {
            if self.y1 <= other.y0 {
                return None;
            }
            if self.y0 >= other.y1 {
                return None;
            }
            // Collided from the left.
            return Some(other.upper_right() - other.lower_right());
        }
        if self.y1 > other.y0 && self.y1 < other.y1 {
            if self.x1 <= other.x0 {
                return None;
            }
            if self.x0 >= other.x1 {
                return None;
            }
            // Collided from bellow.
            return Some(other.lower_right() - other.lower_left());
        }
        if self.y0 < other.y1 && self.y0 > other.y0 {
            if self.x1 <= other.x0 {
                return None;
            }
            if self.x0 >= other.x1 {
                return None;
            }
            // COllided from above.
            return Some(other.upper_left() - other.upper_right());
        }

        return None;
    }
}

impl<'a> Add<Vector2> for &'a Rect2 {
    type Output = Rect2;

    fn add(self, translation: Vector2) -> Rect2 {
        Rect2 {
            x0: self.x0 + translation.x,
            y0: self.y0 + translation.y,

            x1: self.x1 + translation.x,
            y1: self.y1 + translation.y,
        }
    }
}
