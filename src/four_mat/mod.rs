use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::fmt;

mod four_vec;
pub use four_vec::C_LIGHT;
pub use four_vec::K_BOLTZ;
pub use four_vec::E_CHARGE;
pub use four_vec::Q_ALPHA;
pub use four_vec::G_ALPHA;
pub use four_vec::H_BAR;
pub use four_vec::EP_NAUGHT;
pub use four_vec::MU_NAUGHT;
pub use four_vec::BIG_G;
pub use four_vec::M_PROTON;
pub use four_vec::M_ELECTRON;
pub use four_vec::Consts;
pub use four_vec::Sinv;
pub use four_vec::beta;
pub use four_vec::gamma;
pub use four_vec::FourVec;

pub use four_vec::ThreeMat;
pub use four_vec::ThreeVec;
pub use four_vec::{radians_between, degrees_between};

/// Four Matrix
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FourMat {
    /// Four rows, each a calcify::FourVec
    n0: FourVec,
    n1: FourVec,
    n2: FourVec,
    n3: FourVec,
}

impl FourMat {
    /// Returns a new FourMat from four FourVecs
    ///
    /// # Arguments
    ///
    /// * `n0` - calcify::FourVec
    /// * `n1` - calcify::FourVec
    /// * `n2` - calcify::FourVec
    /// * `n3` - calcify::FourVec
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::FourMat;
    /// let mat4 = FourMat::new(
    ///               FourVec::new(1.0,2.0,3.0,4.0),
    ///               FourVec::new(5.0,6.0,7.0,8.0),
    ///               FourVec::new(9.0,10.0,11.0,12.0),
    ///               FourVec::new(13.0,14.0,15.0,16.0)
    ///            );
    /// ```
    pub fn new(n0: FourVec, n1: FourVec, n2: FourVec, n3: FourVec) -> FourMat {
        FourMat {
            n0,
            n1,
            n2,
            n3,
        }
    }

    /// Returns a new FourMat identity matrix
    ///
    /// # Example
    /// ```
    /// use calcify::FourMat;
    /// let mat4 = FourMat::eye();
    ///
    /// assert_eq!(*mat4.n1().m1(),1.0);
    /// ```
    pub fn eye() -> FourMat {
        FourMat {
            n0: FourVec::new(1.0,0.0,0.0,0.0),
            n1: FourVec::new(0.0,1.0,0.0,0.0),
            n2: FourVec::new(0.0,0.0,1.0,0.0),
            n3: FourVec::new(0.0,0.0,0.0,1.0),
        }
    }

    /// Returns a new FourMat zero matrix
    ///
    /// # Example
    /// ```
    /// use calcify::FourMat;
    /// let mat4 = FourMat::zero();
    ///
    /// assert_eq!(*mat4.n1().m1(),0.0);
    /// ```
    pub fn zero() -> FourMat {
        FourMat {
            n0: FourVec::new(0.0,0.0,0.0,0.0),
            n1: FourVec::new(0.0,0.0,0.0,0.0),
            n2: FourVec::new(0.0,0.0,0.0,0.0),
            n3: FourVec::new(0.0,0.0,0.0,0.0),
        }
    }

    /// Returns a new FourMat metric tensor
    ///
    /// # Example
    /// ```
    /// use calcify::FourMat;
    /// let mat4 = FourMat::metric();
    ///
    /// assert_eq!(*mat4.n0().m0(),1.0);
    /// assert_eq!(*mat4.n1().m1(),-1.0);
    /// assert_eq!(*mat4.n2().m1(),0.0);
    /// ```
    pub fn metric() -> FourMat {
        FourMat {
            n0: FourVec::new(1.0,0.0,0.0,0.0),
            n1: FourVec::new(0.0,-1.0,0.0,0.0),
            n2: FourVec::new(0.0,0.0,-1.0,0.0),
            n3: FourVec::new(0.0,0.0,0.0,-1.0),
        }
    }

    /// Returns a new FourMat one matrix
    ///
    /// # Example
    /// ```
    /// use calcify::FourMat;
    /// let mat4 = FourMat::one();
    ///
    /// assert_eq!(*mat4.n1().m1(),1.0);
    /// ```
    pub fn one() -> FourMat {
        FourMat {
            n0: FourVec::new(1.0,1.0,1.0,1.0),
            n1: FourVec::new(1.0,1.0,1.0,1.0),
            n2: FourVec::new(1.0,1.0,1.0,1.0),
            n3: FourVec::new(1.0,1.0,1.0,1.0),
        }
    }

    /// Returns a reference to the first row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::FourMat;
    /// let mat4 = FourMat::new(
    ///               FourVec::new(1.0,2.0,3.0,4.0),
    ///               FourVec::new(5.0,6.0,7.0,8.0),
    ///               FourVec::new(9.0,10.0,11.0,12.0),
    ///               FourVec::new(13.0,14.0,15.0,16.0)
    ///            );
    /// let row_zero: FourVec = *mat4.n0();
    /// let element_zero_zero: f64 = *mat4.n0().m0();
    /// assert_eq!(row_zero,FourVec::new(1.0,2.0,3.0,4.0));
    /// assert_eq!(element_zero_zero,1.0);
    /// ```
    pub fn n0(&self) -> &FourVec {
        &self.n0
    }


    /// Returns a reference to the second row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::FourMat;
    /// let mat4 = FourMat::new(
    ///               FourVec::new(1.0,2.0,3.0,4.0),
    ///               FourVec::new(5.0,6.0,7.0,8.0),
    ///               FourVec::new(9.0,10.0,11.0,12.0),
    ///               FourVec::new(13.0,14.0,15.0,16.0)
    ///            );
    /// let row_one: FourVec = *mat4.n1();
    /// let element_one_one: f64 = *mat4.n1().m1();
    /// assert_eq!(row_one,FourVec::new(5.0,6.0,7.0,8.0));
    /// assert_eq!(element_one_one,6.0);
    /// ```
    pub fn n1(&self) -> &FourVec {
        &self.n1
    }

    /// Returns a reference to the third row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::FourMat;
    /// let mat4 = FourMat::new(
    ///               FourVec::new(1.0,2.0,3.0,4.0),
    ///               FourVec::new(5.0,6.0,7.0,8.0),
    ///               FourVec::new(9.0,10.0,11.0,12.0),
    ///               FourVec::new(13.0,14.0,15.0,16.0)
    ///            );
    /// let row_two: FourVec = *mat4.n2();
    /// let element_two_two: f64 = *mat4.n2().m2();
    /// assert_eq!(row_two,FourVec::new(9.0,10.0,11.0,12.0));
    /// assert_eq!(element_two_two,11.0);
    /// ```
    pub fn n2(&self) -> &FourVec {
        &self.n2
    }

    /// Returns a reference to the forth row of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::FourMat;
    /// let mat4 = FourMat::new(
    ///               FourVec::new(1.0,2.0,3.0,4.0),
    ///               FourVec::new(5.0,6.0,7.0,8.0),
    ///               FourVec::new(9.0,10.0,11.0,12.0),
    ///               FourVec::new(13.0,14.0,15.0,16.0)
    ///            );
    /// let row_three: FourVec = *mat4.n3();
    /// let element_three_three: f64 = *mat4.n3().m3();
    /// assert_eq!(row_three,FourVec::new(13.0,14.0,15.0,16.0));
    /// assert_eq!(element_three_three,16.0);
    /// ```
    pub fn n3(&self) -> &FourVec {
        &self.n3
    }

    /// Returns a new memory FourVec of the first column of the matrix.
    ///
    /// # Example
    /// ```
    /// use calcify::FourVec;
    /// use calcify::FourMat;
    /// let mat4 = FourMat::new(
    ///               FourVec::new(1.0,2.0,3.0,4.0),
    ///               FourVec::new(5.0,6.0,7.0,8.0),
    ///               FourVec::new(9.0,10.0,11.0,12.0),
    ///               FourVec::new(13.0,14.0,15.0,16.0)
    ///            );
    /// let col_one: FourVec = mat4.c0();
    /// let element_one_one: f64 = *mat4.c0().m0();
    /// assert_eq!(col_one,FourVec::new(1.0,5.0,9.0,13.0));
    /// assert_eq!(element_one_one,1.0);
    /// ```
    pub fn c0(&self) -> FourVec {
        FourVec::new(*self.n0.m0(),*self.n1.m0(),*self.n2.m0(),*self.n3.m0())
    }

    pub fn c1(&self) -> FourVec {
        FourVec::new(*self.n0.m1(),*self.n1.m1(),*self.n2.m1(),*self.n3.m1())
    }

    pub fn c2(&self) -> FourVec {
        FourVec::new(*self.n0.m2(),*self.n1.m2(),*self.n2.m2(),*self.n3.m2())
    }

    pub fn c3(&self) -> FourVec {
        FourVec::new(*self.n0.m3(),*self.n1.m3(),*self.n2.m3(),*self.n3.m3())
    }
}

impl fmt::Display for FourMat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},\n{},\n{},\n{}]", self.n0(), self.n1(), self.n2(), self.n3())
    }
}

impl Add for FourMat {
    type Output = FourMat;

    fn add(self, other: FourMat) -> FourMat {
        FourMat {
            n0: self.n0 + *other.n0(),
            n1: self.n1 + *other.n1(),
            n2: self.n2 + *other.n2(),
            n3: self.n3 + *other.n3(),
        }
    }
}

impl AddAssign for FourMat {
    fn add_assign(&mut self, other: FourMat) {
        self.n0 +=*other.n0();
        self.n1 +=*other.n1();
        self.n2 +=*other.n2();
        self.n3 +=*other.n3();
    }
}

impl Sub for FourMat {
    type Output = FourMat;

    fn sub(self, other: FourMat) -> FourMat {
        FourMat {
            n0: self.n0 -*other.n0(),
            n1: self.n1 -*other.n1(),
            n2: self.n2 -*other.n2(),
            n3: self.n3 -*other.n3(),
        }
    }
}

impl SubAssign for FourMat {
    fn sub_assign(&mut self, other: FourMat) {
        self.n0 -=*other.n0();
        self.n1 -=*other.n1();
        self.n2 -=*other.n2();
        self.n3 -=*other.n3();
    }
}

impl Mul<f64> for FourMat {
    type Output = FourMat;

    fn mul(self, coef: f64) -> FourMat {
        FourMat {
            n0: self.n0 *coef,
            n1: self.n1 *coef,
            n2: self.n2 *coef,
            n3: self.n3 *coef,
        }
    }
}

impl Mul<FourMat> for f64 {
    type Output = FourMat;

    fn mul(self, vec: FourMat) -> FourMat {
        FourMat {
            n0: *vec.n0() * self,
            n1: *vec.n1() * self,
            n2: *vec.n2() * self,
            n3: *vec.n3() * self,
        }
    }
}

impl Mul<FourMat> for FourMat {
    type Output = FourMat;
    /// Matrix multiplication
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::FourMat;
    /// use calcify::FourVec;
    ///
    /// let mat4 = FourMat::new(FourVec::new(1.0,2.0,3.0,4.0),
    ///                             FourVec::new(5.0,6.0,7.0,8.0),
    ///                             FourVec::new(9.0,10.0,11.0,12.0),
    ///                             FourVec::new(13.0,14.0,15.0,16.0));
    ///
    /// assert_eq!(
    ///     mat4*mat4,
    ///     FourMat::new(FourVec::new(90.0,100.0,110.0,120.0),
    ///                 FourVec::new(202.0,228.0,254.0,280.0),
    ///                 FourVec::new(314.0,356.0,398.0,440.0),
    ///                 FourVec::new(426.0,484.0,542.0,600.0)));
    /// ```
    fn mul(self, other: FourMat) -> FourMat {
        let c0 = other.c0();
        let c1 = other.c1();
        let c2 = other.c2();
        let c3 = other.c3();
        FourMat {
            n0: FourVec::new(self.n0*c0, self.n0*c1, self.n0*c2, self.n0*c3),
            n1: FourVec::new(self.n1*c0, self.n1*c1, self.n1*c2, self.n1*c3),
            n2: FourVec::new(self.n2*c0, self.n2*c1, self.n2*c2, self.n2*c3),
            n3: FourVec::new(self.n3*c0, self.n3*c1, self.n3*c2, self.n3*c3),
        }
    }
}

impl Mul<FourVec> for FourMat {
    type Output = FourVec;
    /// Matrix multiplication with vector
    ///
    /// # Note
    ///
    /// Only works in one direction FourMat*FourVec, implying FourVec as a column vector.
    ///
    /// # Example
    ///
    /// ```
    /// use calcify::FourMat;
    /// use calcify::FourVec;
    ///
    /// let mat4 = FourMat::new(FourVec::new(1.0,2.0,3.0,4.0),
    ///                             FourVec::new(1.0,2.0,3.0,4.0),
    ///                             FourVec::new(1.0,2.0,3.0,4.0),
    ///                             FourVec::new(1.0,2.0,3.0,4.0));
    ///
    /// assert_eq!(
    ///     mat4*FourVec::new(2.0,2.0,2.0,2.0),
    ///     FourVec::new(20.0,20.0,20.0,20.0)
    /// );
    /// ```
    fn mul(self, other: FourVec) -> FourVec {
        FourVec::new(self.n0*other,self.n1*other,self.n2*other,self.n3*other)
    }
}

impl Neg for FourMat {
    type Output = FourMat;

    fn neg(self) -> FourMat {
        FourMat {
            n0: -self.n0,
            n1: -self.n1,
            n2: -self.n2,
            n3: -self.n3,
        }
    }
}

/// Returns a FourVec, inside a Result, boosted into a frame of arbitrary velocity **v**.
/// Each componant of **v** must be less than calcify::C_LIGHT.
/// Uses a FourMat Lorentz Transformation tensor.
/// # Arguments
///
/// * `initial` - calcify::FourVec
/// * `v` - calcify::ThreeVec
///
/// # Example
/// ```
/// use calcify::boost;
/// use calcify::FourVec;
/// use calcify::ThreeVec;
///
/// let vv = ThreeVec::random(100.0);
/// let vec4 = FourVec::new(10.0,1.0,1.0,1.0);
/// let bVec = boost(vec4,vv);
///
/// ```
pub fn boost(initial: FourVec, v: ThreeVec) -> Result<FourVec,&'static str> {
    let bx = beta(*v.x0())?;
    let by = beta(*v.x1())?;
    let bz = beta(*v.x2())?;
    let bb = bx*bx + by*by + bz*bz;
    let g = gamma(beta((v*v).sqrt())?);

    let ll = FourMat::new(FourVec::new(g,-g*bx,-g*by,-g*bz),
                          FourVec::new(-g*bx,(g - 1.0)*(bx*bx)/bb + 1.0,(g - 1.0)*(bx*by)/bb,(g - 1.0)*(bx*bz)/bb),
                          FourVec::new(-g*by,(g - 1.0)*(bx*by)/bb,(g - 1.0)*(by*by)/bb + 1.0,(g - 1.0)*(by*bz)/bb),
                          FourVec::new(-g*bz,(g - 1.0)*(bx*bz)/bb,(g - 1.0)*(by*bz)/bb,(g - 1.0)*(bz*bz)/bb + 1.0));

    Ok(ll*initial)
}
