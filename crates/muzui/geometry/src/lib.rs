#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T> From<Point<T>> for Size<T> {
    fn from(value: Point<T>) -> Self {
        Self::new(value.x, value.y)
    }
}

impl<T> From<Size<T>> for Point<T> {
    fn from(value: Size<T>) -> Self {
        Self::new(value.width, value.height)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rect<T> {
    pub origin: Point<T>,
    pub size: Size<T>,
}

impl<T> Rect<T> {
    pub const fn new(origin: Point<T>, size: Size<T>) -> Self {
        Self { origin, size }
    }

    pub fn from_xywh(x: T, y: T, width: T, height: T) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }
}

impl<T: Copy> Rect<T> {
    pub const fn x(&self) -> T {
        self.origin.x
    }

    pub const fn y(&self) -> T {
        self.origin.y
    }

    pub const fn width(&self) -> T {
        self.size.width
    }

    pub const fn height(&self) -> T {
        self.size.height
    }
}

macro_rules! impl_math {
    ($type:ty, $first_prop:ident, $second_prop:ident) => {
        impl<T: Add<Output = T>> Add for $type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self::new(
                    self.$first_prop + rhs.$first_prop,
                    self.$second_prop + rhs.$second_prop,
                )
            }
        }

        impl<T: AddAssign> AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                self.$first_prop += rhs.$first_prop;
                self.$second_prop += rhs.$second_prop;
            }
        }

        impl<T: Add<Output = T> + Copy> Add<T> for $type {
            type Output = Self;

            fn add(self, rhs: T) -> Self::Output {
                Self::new(self.$first_prop + rhs, self.$second_prop + rhs)
            }
        }

        impl<T: Sub<Output = T>> Sub for $type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(
                    self.$first_prop - rhs.$first_prop,
                    self.$second_prop - rhs.$second_prop,
                )
            }
        }

        impl<T: SubAssign> SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                self.$first_prop -= rhs.$first_prop;
                self.$second_prop -= rhs.$second_prop;
            }
        }

        impl<T: Sub<Output = T> + Copy> Sub<T> for $type {
            type Output = Self;

            fn sub(self, rhs: T) -> Self::Output {
                Self::new(self.$first_prop - rhs, self.$second_prop - rhs)
            }
        }

        impl<T: Mul<Output = T>> Mul for $type {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(
                    self.$first_prop * rhs.$first_prop,
                    self.$second_prop * rhs.$second_prop,
                )
            }
        }

        impl<T: Mul<Output = T> + Copy> Mul<T> for $type {
            type Output = Self;

            fn mul(self, rhs: T) -> Self::Output {
                Self::new(self.$first_prop * rhs, self.$second_prop * rhs)
            }
        }

        impl<T: Div<Output = T>> Div for $type {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                Self::new(
                    self.$first_prop / rhs.$first_prop,
                    self.$second_prop / rhs.$second_prop,
                )
            }
        }

        impl<T: Div<Output = T> + Copy> Div<T> for $type {
            type Output = Self;

            fn div(self, rhs: T) -> Self::Output {
                Self::new(self.$first_prop / rhs, self.$second_prop / rhs)
            }
        }
    };
}

impl_math!(Point<T>, x, y);
impl_math!(Size<T>, width, height);
impl_math!(Rect<T>, origin, size);

impl<T: Add<Output = T>> Add<Point<T>> for Rect<T> {
    type Output = Self;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Self::new(self.origin + rhs, self.size)
    }
}

impl<T: AddAssign> AddAssign<Point<T>> for Rect<T> {
    fn add_assign(&mut self, rhs: Point<T>) {
        self.origin += rhs;
    }
}

impl<T: AddAssign> AddAssign<Size<T>> for Rect<T> {
    fn add_assign(&mut self, rhs: Size<T>) {
        self.size += rhs;
    }
}

impl<T: SubAssign> SubAssign<Point<T>> for Rect<T> {
    fn sub_assign(&mut self, rhs: Point<T>) {
        self.origin -= rhs;
    }
}

impl<T: SubAssign> SubAssign<Size<T>> for Rect<T> {
    fn sub_assign(&mut self, rhs: Size<T>) {
        self.size -= rhs;
    }
}

impl<T: Add<Output = T>> Add<Size<T>> for Rect<T> {
    type Output = Self;

    fn add(self, rhs: Size<T>) -> Self::Output {
        Self::new(self.origin, self.size + rhs)
    }
}

impl<T: Sub<Output = T>> Sub<Point<T>> for Rect<T> {
    type Output = Self;

    fn sub(self, rhs: Point<T>) -> Self::Output {
        Self::new(self.origin - rhs, self.size)
    }
}

impl<T: Sub<Output = T>> Sub<Size<T>> for Rect<T> {
    type Output = Self;

    fn sub(self, rhs: Size<T>) -> Self::Output {
        Self::new(self.origin, self.size - rhs)
    }
}

impl<T: Div<Output = T>> Div<Point<T>> for Rect<T> {
    type Output = Self;

    fn div(self, rhs: Point<T>) -> Self::Output {
        Self::new(self.origin / rhs, self.size)
    }
}

impl<T: Div<Output = T>> Div<Size<T>> for Rect<T> {
    type Output = Self;

    fn div(self, rhs: Size<T>) -> Self::Output {
        Self::new(self.origin, self.size / rhs)
    }
}

impl<T: Mul<Output = T>> Mul<Point<T>> for Rect<T> {
    type Output = Self;

    fn mul(self, rhs: Point<T>) -> Self::Output {
        Self::new(self.origin * rhs, self.size)
    }
}

impl<T: Mul<Output = T>> Mul<Size<T>> for Rect<T> {
    type Output = Self;

    fn mul(self, rhs: Size<T>) -> Self::Output {
        Self::new(self.origin, self.size * rhs)
    }
}
