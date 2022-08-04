use crate::consts::*;
use rand::prelude::*;
use std::iter::FromIterator;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Float3([f64; 3]);

pub type Color = Float3;
pub type Vec3 = Float3;
pub type Point3 = Float3;

// 生成
impl Float3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }

    pub const fn zero() -> Self {
        Self([0.0; 3])
    }

    pub const fn one() -> Self {
        Self([1.0; 3])
    }

    pub const fn full(value: f64) -> Self {
        Self([value; 3])
    }

    pub const fn xaxis() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    pub const fn yaxis() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub const fn zaxis() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
}

// 算術系
impl Float3 {
    pub fn sqrt(&self) -> Self {
        Self::from_iter(self.0.iter().map(|x| x.sqrt()))
    }

    pub fn near_zero(&self) -> bool {
        self.0.iter().all(|x| x.abs() < EPS)
    }

    pub fn saturate(&self) -> Self {
        Self::from_iter(self.0.iter().map(|x| x.min(1.0).max(0.0)))
    }
}

// アクセス
impl Float3 {
    pub fn x(&self) -> f64 {
        self.0[0]
    }

    pub fn y(&self) -> f64 {
        self.0[1]
    }

    pub fn z(&self) -> f64 {
        self.0[2]
    }

    pub fn to_array(&self) -> [f64; 3] {
        self.0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, f64> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, f64> {
        self.0.iter_mut()
    }
}

// ベクトル演算
impl Float3 {
    // 内積
    pub fn dot(&self, rhs: Self) -> f64 {
        self.0
            .iter()
            .zip(rhs.0.iter())
            .fold(0.0, |acc, (l, r)| acc + l * r)
    }

    // 外積
    pub fn cross(&self, rhs: Self) -> Self {
        Self([
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
        ])
    }

    // 長さ
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    // 長さの2乗
    pub fn length_squared(&self) -> f64 {
        self.0.iter().fold(0.0, |acc, x| acc + x * x)
    }

    // 正規化
    pub fn normalize(&self) -> Self {
        *self / self.length()
    }

    // 線形補間
    pub fn lerp(&self, v: Self, t: f64) -> Self {
        *self + (v - *self) * t
    }
}

// カラー演算
impl Float3 {
    pub fn r(&self) -> u8 {
        // TODO 255.99 ?
        (255.99 * self.0[0].min(1.0).max(0.0)) as u8
    }
    pub fn g(&self) -> u8 {
        (255.99 * self.0[1].min(1.0).max(0.0)) as u8
    }
    pub fn b(&self) -> u8 {
        (255.99 * self.0[2].min(1.0).max(0.0)) as u8
    }

    pub fn from_hex(hex: &[u8; 6]) -> Self {
        use std::str::from_utf8;

        if let Ok(hex_str) = from_utf8(hex) {
            let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap();
            Self::from_rgb(r, g, b)
        } else {
            panic!();
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
    }

    pub fn to_rgb(&self) -> [u8; 3] {
        [self.r(), self.g(), self.b()]
    }
}

impl Float3 {
    // リニア空間からsRGB空間へ
    pub fn gamma(&self, factor: f64) -> Self {
        let recip = factor.recip();
        Self::from_iter(self.0.iter().map(|x| x.powf(recip)))
    }
    // sRGB空間からリニア空間へ
    pub fn degamma(&self, factor: f64) -> Self {
        Self::from_iter(self.0.iter().map(|x| x.powf(factor)))
    }
}

impl Float3 {
    pub fn random() -> Self {
        Self::new(random::<f64>(), random::<f64>(), random::<f64>())
    }

    pub fn random_full() -> Self {
        Self::full(random::<f64>())
    }

    pub fn random_limit(min: f64, max: f64) -> Self {
        Self::from_iter(Self::random().0.iter().map(|x| min + x * (max - min)))
    }

    // 単位球の中の任意の点を生成
    pub fn random_in_unit_sphere() -> Self {
        loop {
            let point = Self::random_limit(-1.0, 1.0);
            if point.length_squared() < 1.0 {
                return point;
            }
        }
    }
}

impl FromIterator<f64> for Float3 {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        let mut initer = iter.into_iter();
        Float3([
            initer.next().unwrap(),
            initer.next().unwrap(),
            initer.next().unwrap(),
        ])
    }
}

impl std::ops::Neg for Float3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::from_iter(self.0.iter().map(|x| -x))
    }
}
impl std::ops::AddAssign<Float3> for Float3 {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..3 {
            self.0[i] += rhs.0[i]
        }
    }
}
impl std::ops::Add<Float3> for Float3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from_iter(self.0.iter().zip(rhs.0.iter()).map(|(l, r)| l + r))
    }
}
impl std::ops::SubAssign<Float3> for Float3 {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..3 {
            self.0[i] -= rhs.0[i]
        }
    }
}
impl std::ops::Sub<Float3> for Float3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::from_iter(self.0.iter().zip(rhs.0.iter()).map(|(l, r)| l - r))
    }
}
impl std::ops::Mul<f64> for Float3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::from_iter(self.0.map(|x| x * rhs))
    }
}
impl std::ops::Mul<Float3> for f64 {
    type Output = Float3;
    fn mul(self, rhs: Float3) -> Float3 {
        Float3::from_iter(rhs.0.map(|x| x * self))
    }
}
impl std::ops::MulAssign<f64> for Float3 {
    fn mul_assign(&mut self, rhs: f64) {
        for i in 0..3 {
            self.0[i] *= rhs
        }
    }
}
impl std::ops::Mul<Float3> for Float3 {
    type Output = Float3;
    fn mul(self, rhs: Float3) -> Float3 {
        Float3::from_iter(self.0.iter().zip(rhs.0.iter()).map(|(l, r)| l * r))
    }
}

impl std::ops::DivAssign<f64> for Float3 {
    fn div_assign(&mut self, rhs: f64) {
        for i in 0..3 {
            self.0[i] /= rhs
        }
    }
}
impl std::ops::Div<f64> for Float3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Float3::from_iter(self.0.iter().map(|x| x / rhs))
    }
}

#[cfg(test)]
mod tests {
    use crate::Float3;

    #[test]
    fn test_create() {
        assert_eq!(Float3::new(0.0, 0.0, 0.0), Float3::zero());
        assert_eq!(Float3::new(1.0, 1.0, 1.0), Float3::one());
        assert_eq!(Float3::new(2.0, 2.0, 2.0), Float3::full(2.0));
        assert_eq!(Float3::new(1.0, 0.0, 0.0), Float3::xaxis());
        assert_eq!(Float3::new(0.0, 1.0, 0.0), Float3::yaxis());
        assert_eq!(Float3::new(0.0, 0.0, 1.0), Float3::zaxis());
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(
            Float3::new(1.0, 2.0, 3.0),
            Float3::new(1.0, 4.0, 9.0).sqrt()
        );
    }

    #[test]
    fn test_near_zero() {
        assert!(!Float3::new(1.0, 2.0, 3.0).near_zero());
        assert!(!Float3::new(1e-6, 1e-6, 01e-6).near_zero());
        assert!(!Float3::new(1.0, 1e-7, 1.0).near_zero());

        assert!(Float3::new(1e-7, 1e-7, 01e-7).near_zero());
        assert!(Float3::zero().near_zero());
    }

    #[test]
    fn test_saturate() {
        assert_eq!(
            Float3::new(0.5, 0.0, 1.0),
            Float3::new(0.5, -1.0, 2.0).saturate()
        );
    }

    #[test]
    fn test_length() {
        assert_eq!(3.0, Float3::new(1.0, 2.0, 2.0).length());
    }

    #[test]
    fn test_dot() {
        let a = Float3::new(3.0, 4.0, 1.0);
        let b = Float3::new(3.0, 7.0, 5.0);
        assert_eq!(42.0, a.dot(b));
    }

    #[test]
    fn test_cross() {
        let a = Float3::new(1.0, 2.0, 3.0);
        let b = Float3::new(4.0, 5.0, 6.0);
        let expect = Float3::new(-3.0, 6.0, -3.0);
        assert_eq!(expect, a.cross(b));
    }

    #[test]
    fn test_ope() {
        let a = Float3::new(2.0, 4.0, 6.0);
        let b = Float3::new(2.0, 2.0, 2.0);
        assert_eq!(Float3::new(4.0, 6.0, 8.0), a + b);
        assert_eq!(Float3::new(0.0, 2.0, 4.0), a - b);
        assert_eq!(Float3::new(4.0, 8.0, 12.0), a * b);

        assert_eq!(Float3::new(4.0, 8.0, 12.0), a * 2.0);
        assert_eq!(Float3::new(1.0, 2.0, 3.0), a / 2.0);

        // TODO assign 系のテスト
    }
}
