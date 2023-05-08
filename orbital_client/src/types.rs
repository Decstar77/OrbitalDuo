use cgmath::*;

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat4 = Matrix4<f32>;

pub type Vec2i = Vector2<i32>;

pub fn normalized_f64_to_i16(v: f64) -> i16 {
    (v * i16::max_value() as f64).round() as i16
}

// Should be safe to use accross the network, let's hope...
pub fn magnitude_i32(v: Vec2i) -> i32 {
    ((v.x * v.x + v.y * v.y) as f64).sqrt().round() as i32
}

pub fn ivec_to_vec(v: Vec2i) -> Vec2 {
    Vec2::new(v.x as f32, v.y as f32)
}

pub fn max_vec(a : Vec2, b : Vec2) -> Vec2 {
    Vec2::new(a.x.max(b.x), a.y.max(b.y))
}

pub fn is_point_on_circle(center : Vec2, radius : f32, point : Vec2) -> bool {
    let dist_x = center.x - point.x;
    let dist_y = center.y - point.y;

    let distance = (dist_x * dist_x + dist_y * dist_y).sqrt();

    distance < radius
}

pub fn is_box_overlapping_circle(min : Vec2, max : Vec2, center : Vec2, radius : f32) -> bool {
    let mut closest_x = center.x;
    let mut closest_y = center.y;

    if center.x < min.x {
        closest_x = min.x;
    } else if center.x > max.x {
        closest_x = max.x;
    }

    if center.y < min.y {
        closest_y = min.y;
    } else if center.y > max.y {
        closest_y = max.y;
    }

    let dist_x = center.x - closest_x;
    let dist_y = center.y - closest_y;

    let distance = (dist_x * dist_x + dist_y * dist_y).sqrt();

    distance < radius
}

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub min : Vec2,
    pub max : Vec2,
}

impl Bounds {
    pub fn new(min : Vec2, max : Vec2) -> Bounds {
        Bounds {
            min,
            max,
        }
    }

    pub fn contains(&self, point : Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }
}
