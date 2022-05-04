use pyo3::{exceptions::PyTypeError, prelude::*, types::PyFloat};

macro_rules! impl_vec {
    ($Vec: ident, $Inner: ty/*$($component: ident), **/) => {
        #[pyclass(module = "hikari")]
        #[derive(Copy, Clone)]
        pub struct $Vec(pub(crate) $Inner);
    
        impl $Vec {
            fn to_vec<'a>(any: &'a PyAny) -> PyResult<$Inner> {
                if let Ok(pyfloat) = any.cast_as::<PyFloat>() {
                    // If it is a float in python land
                    let float = pyfloat.value() as f32;
                    Ok(<$Inner>::ONE * float)
                } 
                else if let Ok(vec) = any.extract::<$Vec>() {
                    // If it is a vec in python land
                    Ok(vec.0)
                } else {
                    let typename = any.get_type().name().unwrap();
                    let ourname = stringify!($Vec);
                    PyResult::Err(PyTypeError::new_err(format!(
                        "Can't convert {typename} to {ourname}"
                    )))
                }
            }
        }

        #[pymethods]
        impl $Vec {
            /// All zeroes
            #[classattr]
            const ZERO: $Vec = Self(<$Inner>::ZERO);
            /// All ones
            #[classattr]
            const ONE: $Vec = Self(<$Inner>::ONE);
            /// A unit-length vector pointing along the positive X axis.
            #[classattr]
            const X: $Vec =Self(<$Inner>::X);
            /// A unit-length vector pointing along the positive Y axis.
            #[classattr]
            const Y: $Vec =Self(<$Inner>::Y);
            
            #[inline]
            fn __add__(&self, other: &PyAny) -> PyResult<Self> {
                Ok(Self(self.0 - Self::to_vec(other)?))
            }
            #[inline]
            fn __sub__(&self, other: &PyAny) -> PyResult<Self> {
                Ok(Self(self.0 - Self::to_vec(other)?))
            }
            #[inline]
            fn __mul__(&self, other: &PyAny) -> PyResult<Self> {
                Ok(Self(self.0 * Self::to_vec(other)?))
            }
            #[inline]
            fn __rmul__(&self, other: &PyAny) -> PyResult<Self> {
                Ok(Self(self.0 * Self::to_vec(other)?))
            }
            #[inline]
            fn __div__(&self, other: &PyAny) -> PyResult<Self> {
                Ok(Self(self.0 / Self::to_vec(other)?))
            }
            #[inline]
            fn __iadd__(&mut self, other: &PyAny) -> PyResult<()> {
                self.0 += Self::to_vec(other)?;
                Ok(())
            }
            #[inline]
            fn __isub__(&mut self, other: &PyAny) -> PyResult<()>{
                self.0 -= Self::to_vec(other)?;
                Ok(())
            }
            #[inline]
            fn __imul__(&mut self, other: &PyAny) -> PyResult<()>{
                self.0 *= Self::to_vec(other)?;
                Ok(())
            }
            #[inline]
            fn __neg__(&self) -> Self {
                Self(-self.0)
            }
            #[inline]
            fn __pos__(&self) -> Self {
                Self(self.0)
            }
            #[inline]
            fn __eq__(&self, other: &Self) -> bool {
                self.0 == other.0
            }
            #[inline]
            fn __repr__(&self) -> String {
               self.0.to_string()
            }
            #[inline]
            fn __str__(&self) -> String {
                self.__repr__()
            }
            /// Returns the length of self
            #[inline]
            pub fn length(&self) -> f32 {
                self.0.length()
            }

            /// Returns the squared length of self
            #[inline]
            fn length_squared(&self) -> f32 {
                self.0.length_squared()
            }

            /// Computes the Euclidean distance of `other` from `self`.
            #[inline]
            fn distance(&self, other: &Self) -> f32 {
                self.0.distance(other.0)
            }

            /// Compute the squared euclidean distance of `other` from `self`.
            #[inline]
            fn distance_squared(&self, other: &Self) -> f32 {
                self.0.distance_squared(other.0)
            }
        
            /// Returns `self` normalized to length 1.0 if possible, else returns zero.
            ///
            /// In particular, if the input is zero (or very close to zero), or non-finite,
            /// the result of this operation will be zero.
            #[inline]
            fn normalize(&self) -> Self {
                Self(self.0.normalize_or_zero())
            }

            /// Returns a vector with a length no less than `min` and no more than `max`
            #[inline]
            fn clamp(&self, min: &Self, max: &Self) -> Self {
                Self(self.0.clamp(min.0, max.0))
            }

            /// Computes the dot product of `self` and `other`.
            #[inline]
            fn dot(&self, b: &Self) -> f32 {
                self.0.dot(b.0)
            }

            /// Performs a linear interpolation between `self` and `other` based on the value `s`.
            ///
            /// When `s` is `0.0`, the result will be equal to `self`.  When `s` is `1.0`, the result
            /// will be equal to `other`.
            #[staticmethod]
            #[inline]
            fn lerp(a: &Self, b: &Self, t: f32) -> Self {
                Self(a.0.lerp(b.0, t))
            }

            /// Returns a vector containing the minimum values for each element of `self` and `other`.
            ///
            /// In other words this computes `[self.x.max(other.x), self.y.max(other.y), ..]`.
            #[staticmethod]
            #[inline]
            fn min(a: &Self, b: &Self) -> Self {
                Self(a.0.min(b.0))
            }

            /// Returns a vector containing the maximum values for each element of `self` and `other`.
            ///
            /// In other words this computes `[self.x.max(other.x), self.y.max(other.y), ..]`.
            #[staticmethod]
            #[inline]
            fn max(a: &Self, b: &Self) -> Self {
                Self(a.0.max(b.0))
            }


    }
    };
}

impl_vec!(Vec2, glam::Vec2);
impl_vec!(Vec3, glam::Vec3A);
impl_vec!(Vec4, glam::Vec4);

#[pymethods]
impl Vec2 {
    #[new]
    fn __new__(x: f32, y: f32) -> Self {
        Self(glam::vec2(x, y))
    }
    #[getter]
    fn x(&self) -> f32 {
        self.0.x
    }
    #[setter]
    fn set_x(&mut self, value: f32) {
        self.0.x = value;
    }
    #[getter]
    fn y(&self) -> f32 {
        self.0.y
    }
    #[setter]
    fn set_y(&mut self, value: f32) {
        self.0.y = value;
    }
}

#[pymethods]
impl Vec3 {
    /// A unit-length vector pointing along the positive Z axis.
    #[classattr]
    const Z: Vec3 =Self(glam::Vec3A::Z);

    #[new]
    fn __new__(x: f32, y: f32, z: f32) -> Self {
        Self(glam::vec3a(x, y, z))
    }
    #[getter]
    fn x(&self) -> f32 {
        self.0.x
    }
    #[setter]
    fn set_x(&mut self, value: f32) {
        self.0.x = value;
    }
    #[getter]
    fn y(&self) -> f32 {
        self.0.y
    }
    #[setter]
    fn set_y(&mut self, value: f32) {
        self.0.y = value;
    }
    #[getter]
    fn z(&self) -> f32 {
        self.0.z
    }
    #[setter]
    fn set_z(&mut self, value: f32) {
        self.0.z = value;
    }

    #[inline]
    fn cross(&self, other: &Self) -> Self {
        Self(self.0.cross(other.0))
    }
}

#[pymethods]
impl Vec4 {
    /// A unit-length vector pointing along the positive Z axis.
    #[classattr]
    const Z: Vec4 =Self(glam::Vec4::Z);
    /// A unit-length vector pointing along the positive W axis.
    #[classattr]
    const W: Vec4 =Self(glam::Vec4::W);

    #[new]
    fn __new__(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(glam::vec4(x, y, z, w))
    }
    #[getter]
    fn x(&self) -> f32 {
        self.0.x
    }
    #[setter]
    fn set_x(&mut self, value: f32) {
        self.0.x = value;
    }
    #[getter]
    fn y(&self) -> f32 {
        self.0.y
    }
    #[setter]
    fn set_y(&mut self, value: f32) {
        self.0.y = value;
    }
    #[getter]
    fn z(&self) -> f32 {
        self.0.z
    }
    #[setter]
    fn set_z(&mut self, value: f32) {
        self.0.z = value;
    }
    #[getter]
    fn w(&self) -> f32 {
        self.0.w
    }
    #[setter]
    fn set_w(&mut self, value: f32) {
        self.0.w = value;
    }
}