use bevy::{
    math::{bounding::RayCast3d, Vec3A},
    prelude::Triangle3d,
};

pub trait RaycastExt {
    fn intersect_triangle(&self, triangle: Triangle3d) -> Option<Vec3A>;
}

impl RaycastExt for RayCast3d {
    // From https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm#Rust_implementation
    fn intersect_triangle(&self, triangle: Triangle3d) -> Option<Vec3A> {
        let e1 = Vec3A::from(triangle.vertices[1] - triangle.vertices[0]);
        let e2 = Vec3A::from(triangle.vertices[2] - triangle.vertices[0]);

        let ray_cross_e2 = self.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None; // This ray is parallel to this triangle.
        }

        let inv_det = 1.0 / det;
        let s = self.origin - Vec3A::from(triangle.vertices[0]);
        let u = inv_det * s.dot(ray_cross_e2);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let s_cross_e1 = s.cross(e1);
        let v = inv_det * self.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);

        if t > f32::EPSILON {
            // ray intersection
            let intersection_point = self.origin + self.direction * t;
            Some(intersection_point)
        } else {
            // This means that there is a line intersection but not a ray intersection.
            None
        }
    }
}
