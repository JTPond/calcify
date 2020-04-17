extern crate rand;

use std::f64;
use std::str::FromStr;
use std::num::ParseFloatError;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::iter;
use std::fmt;

extern crate rmp;
use rmp::encode::*;

use self::rand::thread_rng;
use self::rand::distributions::{Distribution, Uniform};

use crate::utils;

use utils::Serializable;

/// Point, or Two Vector, depending on your perspective. 
/// A plot is a Collection of Points
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Returns new Point
    ///
    /// # Arguments
    ///
    /// * `x` - f64 Independent Variable
    /// * `y` - f64 Dependent Variable
    ///
    pub fn new(x: f64, y: f64) -> Point {
        Point {
            x,
            y,
        }
    }

    /// Returns a new Point with two random f64 from rand::Uniform between -1 and 1
    ///
    /// # Arguments
    ///
    /// * `max` - f64: The absolute maximum value of each individule componant of the constituent Point
    ///
    /// # Example
    /// ```
    /// use calcify::Point;
    /// let vec2 = Point::random(10.0);
    /// ```
    pub fn random(max: f64) -> Point {
        let between = Uniform::new_inclusive(-1.0f64,1.0f64);
        let mut rng = thread_rng();
        Point {
            x: between.sample(&mut rng)*max,
            y: between.sample(&mut rng)*max,
        }
    }

    /// Returns the length of the 2vector
    ///
    /// # Example
    /// ```
    /// use calcify::Point;
    /// let vec2 = Point::new(1.0,0.0);
    /// assert_eq!(vec2.r(),1.0);
    /// ```
    pub fn r(&self) -> f64 {
        (*self**self).sqrt()
    }
}

impl Serializable for Point {
    fn to_json(&self) -> String {
        format!("{{\"x\":{},\"y\":{}}}", self.x, self.y)
    }
    fn to_jsonc(&self) -> String {
        format!("[{},{}]", self.x, self.y)
    }
    fn to_msg(&self) -> Result<Vec<u8>,ValueWriteError> {
        let mut buf = Vec::with_capacity(3);
        write_array_len(&mut buf, 2)?;
        write_f64(&mut buf, self.x)?;
        write_f64(&mut buf, self.y)?;
        Ok(buf)
    }
}

impl FromStr for Point {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut x: f64 = std::f64::NAN;
        let mut y: f64 = std::f64::NAN;
        for dim in s.trim_matches(|p| p == '{' || p == '}' ).split(',') {
            let n_v: Vec<&str> = dim.split(':').collect();
            match n_v[0] {
                "\"x\"" => x = n_v[1].parse::<f64>()?,
                "\"y\"" => y = n_v[1].parse::<f64>()?,
                x => panic!("Unexpected invalid token {:?}", x),
            }
        }
        Ok(Point{x,y})
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.*}, {:.*}]", 5, self.x, 5, self.y)
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

impl iter::Sum for Point {
    fn sum<I>(iter: I) -> Point
    where I: Iterator<Item = Point> {
        iter.fold(Point { x: 0.0, y: 0.0 }, |a, b| a + b)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
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

impl SubAssign for Point {
    fn sub_assign(&mut self, other: Point) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, coef: f64) -> Point {
        Point {
            x: self.x * coef,
            y: self.y * coef,
        }
    }
}

impl Mul<Point> for f64 {
    type Output = Point;

    fn mul(self, vec: Point) -> Point {
        Point {
            x: vec.x * self,
            y: vec.y * self,
        }
    }
}

impl Mul<Point> for Point {
    type Output = f64;
    /// Dot product
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::Point;
    /// let point = Point::new(2.0,2.0);
    ///
    /// assert_eq!(
    ///    point*point,
    ///    8.0
    /// );
    /// ```
    fn mul(self, other: Point) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let _test_vec1 = Point::new(1.0,2.0);
        let _test_vec2 = Point::new(2.0,3.0);

        assert_eq!(
            _test_vec1+_test_vec2,
            Point::new(3.0,5.0)
        );
    }

    #[test]
    fn test_loop_add() {
        let mut _test_vec1 = Point::new(1.0,1.0);
        for _i in 0..9999{
            _test_vec1 += Point::new(1.0,1.0);
        }

        assert_eq!(
            _test_vec1,
            Point::new(10_000.0,10_000.0)
        );
    }

    #[test]
    fn test_sub() {
        let _test_vec1 = Point::new(3.0,5.0);
        let _test_vec2 = Point::new(2.0,3.0);

        assert_eq!(
            _test_vec1-_test_vec2,
            Point::new(1.0,2.0)
        );
    }

    #[test]
    fn test_loop_sub() {
        let mut _test_vec1 = Point::new(10_000.0,10_000.0);
        for _i in 0..9999{
            _test_vec1 -= Point::new(1.0,1.0);
        }

        assert_eq!(
            _test_vec1,
            Point::new(1.0,1.0)
        );
    }

    #[test]
    fn test_mul() {
        let _test_vec1 = Point::new(2.0,2.0);
        let _test_vec2 = Point::new(2.0,2.0);

        assert_eq!(
            _test_vec1*_test_vec2,
            8.0
        );
    }

    #[test]
    fn test_mul_coef() {
        let _test_vec1 = Point::new(2.0,2.0);

        assert_eq!(
            _test_vec1*2.0,
            Point::new(4.0,4.0)
        );
        assert_eq!(
            2.0*_test_vec1,
            Point::new(4.0,4.0)
        );
    }

    #[test]
    fn test_neg() {
        let _test_vec1 = Point::new(2.0,2.0);

        assert_eq!(
            -_test_vec1,
            Point::new(-2.0,-2.0)
        );
    }

    #[test]
    fn test_copy() {
        let xx = Point::new(1.0,1.0);
        let yy = xx;
        assert_eq!(
            xx+yy,
            Point::new(2.0,2.0)
        );
    }

    #[test]
    fn test_parse() {
        let xx = Point::new(1.0,1.0);
        let pp = xx.to_json();
        assert_eq!(Point::from_str(&pp).unwrap(),xx);
    }
}
