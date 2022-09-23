#![allow(dead_code)]

mod consts;
mod rayt;

use consts::*;
use rayt::camera::*;
use rayt::float3::*;
use rayt::onb::*;
use rayt::ray::*;
use rayt::render::*;
use std::sync::Arc;

trait Texture: Sync + Send {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

struct HitInfo {
    t: f64,               // 光線のパラメーター
    p: Point3,            // 衝突位置
    n: Vec3,              // 衝突した位置の法線
    m: Arc<dyn Material>, // 材質
    u: f64,               // テキスチャ座標
    v: f64,               // テキスチャ座標
}

impl HitInfo {
    const fn new(t: f64, p: Point3, n: Vec3, m: Arc<dyn Material>, u: f64, v: f64) -> Self {
        Self { t, p, n, m, u, v }
    }
}

struct ColorTexture {
    color: Color,
}

impl ColorTexture {
    const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Texture for ColorTexture {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.color
    }
}

struct CheckerTexture {
    odd: Box<dyn Texture>,
    even: Box<dyn Texture>,
    freq: f64,
}

impl CheckerTexture {
    const fn new(odd: Box<dyn Texture>, even: Box<dyn Texture>, freq: f64) -> Self {
        Self { odd, even, freq }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        let sines = p.iter().fold(1.0, |acc, x| acc * (x * self.freq).sin());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

trait Material: Sync + Send {
    // 散乱をシミュレート
    fn scatter(&self, ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo>;
    fn emitted(&self, _ray: &Ray, _hit: &HitInfo) -> Color {
        Color::zero()
    }
    fn scattering_pdf(&self, _ray: &Ray, _hit: &HitInfo) -> f64 {
        0.0
    }
}

trait Pdf {
    fn value(&self, hit: &HitInfo, direction: Vec3) -> f64;
    fn generate(&self, hit: &HitInfo) -> Vec3;
}

struct CosinePdf {}
impl CosinePdf {
    const fn new() -> Self {
        Self {}
    }
}

impl Pdf for CosinePdf {
    fn value(&self, hit: &HitInfo, direction: Vec3) -> f64 {
        let cosine = direction.normalize().dot(hit.n);
        if cosine > 0.0 {
            cosine * FRAC_1_PI
        } else {
            0.0
        }
    }

    fn generate(&self, hit: &HitInfo) -> Vec3 {
        ONB::new(hit.n).local(Vec3::random_cosine_direction())
    }
}

struct ScatterInfo {
    ray: Ray,       // 散乱後の光の向き
    albedo: Color,  // 反射率(アルベド)
    pdf_value: f64, // 確率の重み
}

impl ScatterInfo {
    fn new(ray: Ray, albedo: Color, pdf_value: f64) -> Self {
        Self {
            ray,
            albedo,
            pdf_value,
        }
    }
}

struct DiffuseLight {
    emit: Box<dyn Texture>,
}

impl DiffuseLight {
    fn new(emit: Box<dyn Texture>) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray: &Ray, _hit: &HitInfo) -> Option<ScatterInfo> {
        None
    }

    fn emitted(&self, ray: &Ray, hit: &HitInfo) -> Color {
        if ray.direction.dot(hit.n) < 0.0 {
            self.emit.value(hit.u, hit.v, hit.p)
        } else {
            Color::zero()
        }
    }
}

// 拡散反射するような材質
struct Lambertian {
    albedo: Box<dyn Texture>,
}

impl Lambertian {
    fn new(albedo: Box<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo> {
        let direction = ONB::new(hit.n).local(Vec3::random_cosine_direction());
        let new_ray = Ray::new(hit.p, direction.normalize());
        let albedo = self.albedo.value(hit.u, hit.v, hit.p);
        let pdf_value = new_ray.direction.dot(hit.n) * FRAC_1_PI;
        Some(ScatterInfo::new(new_ray, albedo, pdf_value))
    }
    fn scattering_pdf(&self, ray: &Ray, hit: &HitInfo) -> f64 {
        ray.direction.normalize().dot(hit.n).max(0.0) * FRAC_1_PI
    }
}

struct Metal {
    albedo: Box<dyn Texture>,
    fuzz: f64, // 反射のずれ度合い
}

impl Metal {
    fn new(albedo: Box<dyn Texture>, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo> {
        let mut reflected = ray.direction.normalize().reflect(hit.n);
        reflected = reflected + self.fuzz * Vec3::random_in_unit_sphere();
        if reflected.dot(hit.n) > 0.0 {
            let albedo = self.albedo.value(hit.u, hit.v, hit.p);
            Some(ScatterInfo::new(Ray::new(hit.p, reflected), albedo, 0.0))
        } else {
            None
        }
    }
}

struct Dielectric {
    ri: f64, // 屈折率
}

impl Dielectric {
    fn new(ri: f64) -> Self {
        Self { ri }
    }

    fn schlick(cosine: f64, ri: f64) -> f64 {
        let r0 = ((1.0 - ri) / (1.0 + ri)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitInfo) -> Option<ScatterInfo> {
        let reflected = ray.direction.reflect(hit.n);
        let (outward_normal, ni_over_nt, cosine) = {
            let dot = ray.direction.dot(hit.n);
            if dot > 0.0 {
                (-hit.n, self.ri, self.ri * dot / ray.direction.length())
            } else {
                (hit.n, self.ri.recip(), -dot / ray.direction.length())
            }
        };

        if let Some(refracted) = (-ray.direction).refract(outward_normal, ni_over_nt) {
            if Vec3::random_full().x() > Self::schlick(cosine, self.ri) {
                return Some(ScatterInfo::new(
                    Ray::new(hit.p, refracted),
                    Color::one(),
                    0.0,
                ));
            }
        }
        Some(ScatterInfo::new(
            Ray::new(hit.p, reflected),
            Color::one(),
            0.0,
        ))
    }
}

trait Shape: Sync {
    fn hit(
        &self,
        ray: &Ray,
        t0: f64, //t0 ~ t1 は衝突範囲
        t1: f64,
    ) -> Option<HitInfo>;
}

struct Sphere {
    center: Point3,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    const fn new(center: Point3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Shape for Sphere {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * ray.direction.dot(oc);
        let c = oc.dot(oc) - self.radius.powi(2);
        let d = b * b - 4.0 * a * c;
        if d > 0.0 {
            let root = d.sqrt();
            let temp = (-b - root) / (2.0 * a);
            if t0 < temp && temp < t1 {
                let p = ray.at(temp);
                return Some(HitInfo::new(
                    temp,
                    p,
                    (p - self.center) / self.radius,
                    Arc::clone(&self.material),
                    0.0,
                    0.0,
                ));
            }
            let temp = (-b + root) / (2.0 * a);
            if t0 < temp && temp < t1 {
                let p = ray.at(temp);
                return Some(HitInfo::new(
                    temp,
                    p,
                    (p - self.center) / self.radius,
                    Arc::clone(&self.material),
                    0.0,
                    0.0,
                ));
            }
        }
        None
    }
}

enum RectAxisType {
    XY,
    XZ,
    YZ,
}

struct Rect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
    axis: RectAxisType,
    material: Arc<dyn Material>,
}

impl Rect {
    fn new(
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
        k: f64,
        axis: RectAxisType,
        material: Arc<dyn Material>,
    ) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k,
            axis,
            material,
        }
    }
}

impl Shape for Rect {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;
        let mut axis = Vec3::zaxis();
        match self.axis {
            RectAxisType::XY => {}
            RectAxisType::XZ => {
                origin = Point3::new(origin.x(), origin.z(), origin.y());
                direction = Vec3::new(direction.x(), direction.z(), direction.y());
                axis = Vec3::yaxis();
            }
            RectAxisType::YZ => {
                origin = Point3::new(origin.y(), origin.z(), origin.x());
                direction = Vec3::new(direction.y(), direction.z(), direction.x());
                axis = Vec3::xaxis();
            }
        }

        let t = (self.k - origin.z()) / direction.z();
        if t < t0 || t > t1 {
            return None;
        }

        let x = origin.x() + t * direction.x();
        let y = origin.y() + t * direction.y();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        Some(HitInfo::new(
            t,
            ray.at(t),
            axis,
            Arc::clone(&self.material),
            (x - self.x0) / (self.x1 - self.x0),
            (y - self.y0) / (self.y1 - self.y0),
        ))
    }
}

struct FlipFace {
    shape: Box<dyn Shape>,
}

impl FlipFace {
    fn new(shape: Box<dyn Shape>) -> Self {
        Self { shape }
    }
}

impl Shape for FlipFace {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        if let Some(hit) = self.shape.hit(ray, t0, t1) {
            Some(HitInfo { n: -hit.n, ..hit })
        } else {
            None
        }
    }
}

struct Box3D {
    p0: Point3,
    p1: Point3,
    shapes: ShapeList,
}

impl Box3D {
    fn new(p0: Point3, p1: Point3, material: Arc<dyn Material>) -> Self {
        let mut shapes = ShapeList::new();

        shapes.push(
            ShapeBuilder::new()
                .material(Arc::clone(&material))
                .rect_xy(p0.x(), p1.x(), p0.y(), p1.y(), p1.z())
                .build(),
        );
        shapes.push(
            ShapeBuilder::new()
                .material(Arc::clone(&material))
                .rect_xy(p0.x(), p1.x(), p0.y(), p1.y(), p0.z())
                .flip_face()
                .build(),
        );
        shapes.push(
            ShapeBuilder::new()
                .material(Arc::clone(&material))
                .rect_xz(p0.x(), p1.x(), p0.z(), p1.z(), p1.y())
                .build(),
        );
        shapes.push(
            ShapeBuilder::new()
                .material(Arc::clone(&material))
                .rect_xz(p0.x(), p1.x(), p0.z(), p1.z(), p0.y())
                .flip_face()
                .build(),
        );
        shapes.push(
            ShapeBuilder::new()
                .material(Arc::clone(&material))
                .rect_yz(p0.y(), p1.y(), p0.z(), p1.z(), p1.x())
                .build(),
        );
        shapes.push(
            ShapeBuilder::new()
                .material(Arc::clone(&material))
                .rect_yz(p0.y(), p1.y(), p0.z(), p1.z(), p0.x())
                .flip_face()
                .build(),
        );

        Self { p0, p1, shapes }
    }
}

impl Shape for Box3D {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        self.shapes.hit(ray, t0, t1)
    }
}

struct ShapeList {
    pub objects: Vec<Box<dyn Shape>>,
}

impl ShapeList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn push(&mut self, object: Box<dyn Shape>) {
        self.objects.push(object);
    }
}

impl Shape for ShapeList {
    fn hit(&self, ray: &Ray, t0: f64, t1: f64) -> Option<HitInfo> {
        let mut hit_info: Option<HitInfo> = None;
        let mut closest_so_far = t1;
        for object in &self.objects {
            if let Some(info) = object.hit(ray, t0, closest_so_far) {
                closest_so_far = info.t;
                hit_info = Some(info);
            }
        }

        hit_info
    }
}

struct ShapeBuilder {
    texture: Option<Box<dyn Texture>>,
    material: Option<Arc<dyn Material>>,
    shape: Option<Box<dyn Shape>>,
}

impl ShapeBuilder {
    fn new() -> Self {
        Self {
            texture: None,
            material: None,
            shape: None,
        }
    }

    fn color_texture(mut self, color: Color) -> Self {
        self.texture = Some(Box::new(ColorTexture::new(color)));
        self
    }

    fn checker_texture(mut self, odd_color: Color, even_color: Color, freq: f64) -> Self {
        self.texture = Some(Box::new(CheckerTexture::new(
            Box::new(ColorTexture::new(odd_color)),
            Box::new(ColorTexture::new(even_color)),
            freq,
        )));
        self
    }

    // material

    fn material(mut self, material: Arc<dyn Material>) -> Self {
        self.material = Some(material);
        self
    }

    fn lambertian(mut self) -> Self {
        self.material = Some(Arc::new(Lambertian::new(self.texture.unwrap())));
        // none にしないと所有権チェックでエラーになる
        self.texture = None;
        self
    }

    fn metal(mut self, fuzz: f64) -> Self {
        self.material = Some(Arc::new(Metal::new(self.texture.unwrap(), fuzz)));
        self.texture = None;
        self
    }

    fn dielectric(mut self, ri: f64) -> Self {
        self.material = Some(Arc::new(Dielectric::new(ri)));
        self
    }

    fn diffuse_light(mut self) -> Self {
        self.material = Some(Arc::new(DiffuseLight::new(self.texture.unwrap())));
        self.texture = None;
        self
    }

    // shapes
    fn sphere(mut self, center: Point3, radius: f64) -> Self {
        self.shape = Some(Box::new(Sphere::new(
            center,
            radius,
            self.material.unwrap(),
        )));
        self.material = None;
        self
    }

    fn rect_xy(mut self, x0: f64, x1: f64, y0: f64, y1: f64, k: f64) -> Self {
        self.shape = Some(Box::new(Rect::new(
            x0,
            x1,
            y0,
            y1,
            k,
            RectAxisType::XY,
            self.material.unwrap(),
        )));
        self.material = None;
        self
    }

    fn rect_xz(mut self, x0: f64, x1: f64, y0: f64, y1: f64, k: f64) -> Self {
        self.shape = Some(Box::new(Rect::new(
            x0,
            x1,
            y0,
            y1,
            k,
            RectAxisType::XZ,
            self.material.unwrap(),
        )));
        self.material = None;
        self
    }

    fn rect_yz(mut self, x0: f64, x1: f64, y0: f64, y1: f64, k: f64) -> Self {
        self.shape = Some(Box::new(Rect::new(
            x0,
            x1,
            y0,
            y1,
            k,
            RectAxisType::YZ,
            self.material.unwrap(),
        )));
        self.material = None;
        self
    }

    fn box3d(mut self, p0: Point3, p1: Point3) -> Self {
        self.shape = Some(Box::new(Box3D::new(p0, p1, self.material.unwrap())));
        self.material = None;
        self
    }

    fn flip_face(mut self) -> Self {
        self.shape = Some(Box::new(FlipFace::new(self.shape.unwrap())));
        self
    }

    fn build(self) -> Box<dyn Shape> {
        self.shape.unwrap()
    }
}

struct CornelBoxScene {
    world: ShapeList,
}

impl CornelBoxScene {
    fn new() -> Self {
        let mut world = ShapeList::new();

        let red = Color::new(0.64, 0.05, 0.05);
        let white = Color::full(0.73);
        let green = Color::new(0.12, 0.45, 0.15);

        world.push(
            ShapeBuilder::new()
                .color_texture(green)
                .lambertian()
                .rect_yz(0.0, 555.0, 0.0, 555.0, 555.0)
                .flip_face()
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(red)
                .lambertian()
                .rect_yz(0.0, 555.0, 0.0, 555.0, 0.0)
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(Color::full(15.0))
                .diffuse_light()
                .rect_xz(213.0, 343.0, 227.0, 332.0, 554.0)
                .flip_face()
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(white)
                .lambertian()
                .rect_xz(0.0, 555.0, 0.0, 555.0, 555.0)
                .flip_face()
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(white)
                .lambertian()
                .rect_xz(0.0, 555.0, 0.0, 555.0, 0.0)
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(white)
                .lambertian()
                .rect_xy(0.0, 555.0, 0.0, 555.0, 555.0)
                .flip_face()
                .build(),
        );

        world.push(
            ShapeBuilder::new()
                .color_texture(white)
                .lambertian()
                .box3d(
                    Point3::new(130.0, 0.0, 65.0),
                    Point3::new(295.0, 165.0, 230.0),
                )
                .build(),
        );
        world.push(
            ShapeBuilder::new()
                .color_texture(white)
                .lambertian()
                .box3d(
                    Point3::new(265.0, 0.0, 295.0),
                    Point3::new(430.0, 330.0, 460.0),
                )
                .build(),
        );

        Self { world }
    }

    fn background(&self, _d: Vec3) -> Color {
        Color::full(0.0)
    }
}

impl SceneWithDepth for CornelBoxScene {
    fn camera(&self) -> Camera {
        Camera::from_lookat(
            Vec3::new(278.0, 278.0, -800.0),
            Vec3::new(278.0, 278.0, 0.0),
            Vec3::yaxis(),
            40.0,
            self.aspect(),
        )
    }

    fn trace(&self, ray: Ray, depth: usize) -> Color {
        let hit_info = self.world.hit(&ray, 0.001, f64::MAX);

        if let Some(hit) = hit_info {
            let emitted = hit.m.emitted(&ray, &hit);
            let scatter_info = if depth > 0 {
                hit.m.scatter(&ray, &hit)
            } else {
                None
            };
            if let Some(scatter) = scatter_info {
                let pdf = CosinePdf::new();
                let new_ray = Ray::new(hit.p, pdf.generate(&hit));

                let spdf_value = pdf.value(&hit, new_ray.direction);
                if spdf_value > 0.0 {
                    let pdf_value = hit.m.scattering_pdf(&new_ray, &hit);
                    let albedo = scatter.albedo * pdf_value;
                    return emitted + albedo * self.trace(new_ray, depth - 1) / spdf_value;
                } else {
                    return emitted;
                }
            } else {
                return emitted;
            }
        }
        self.background(ray.direction)
    }
}

fn main() {
    render_aa_with_depth(CornelBoxScene::new());
}
