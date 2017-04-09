use std::ops::Add;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;
use std::ops::AddAssign;


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

    // TODO(erick): It would be nice if we had some unit-test for this thing.
    pub fn collides_with(&self, other: &Rect2) -> bool {
        if self.x1 > other.x0 && self.x1 < other.x0 {
            if self.y1 <= other.y0 {
                return false;
            }
            if self.y0 >= other.y1 {
                return false;
            }

            return true;
        }
        if self.x0 < other.x1 && self.x1 > other.x0 {
            if self.y1 <= other.y0 {
                return false;
            }
            if self.y0 >= other.y1 {
                return false;
            }
            return true;
        }
        if self.y1 > other.y0 && self.y1 < other.y0 {
            if self.x1 <= other.x0 {
                return false;
            }
            if self.x0 >= other.x1 {
                return false;
            }

            return true;
        }
        if self.y0 < other.y1 && self.y1 > other.y0 {
            if self.x1 <= other.x0 {
                return false;
            }
            if self.x0 >= other.x1 {
                return false;
            }

            return true;
        }

        return false;
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
