use crate::consts::*;
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

    // TODO 修正
    //
    // 正規化
    // pub fn normalize(&self) -> Self {
    //     *self / self.length()
    // }
    //
    // 線形補間
    // pub fn lerp(&self, v: Self, t: f64) -> Self {
    //     *self + (v - *self) * t
    // }
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
