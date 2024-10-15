use bevy::{
    math::{Vec3, Vec3A},
    prelude::{GlobalTransform, Transform, TransformPoint},
    render::primitives::Aabb,
};

pub trait AabbExt {
    fn compute_global_transform(&self, transform: GlobalTransform) -> GlobalTransform;
    fn rotate(&self, transform: impl TransformPoint) -> Self;
}

impl AabbExt for Aabb {
    // Copied from bevy_gizmos/src/aabb.rs
    fn compute_global_transform(&self, transform: GlobalTransform) -> GlobalTransform {
        transform
            * GlobalTransform::from(
                Transform::from_translation(self.center.into())
                    .with_scale((self.half_extents * 2.).into()),
            )
    }

    fn rotate(&self, transform: impl TransformPoint) -> Self {
        #[allow(clippy::expect_used)]
        Aabb::enclosing(
            [
                self.center + (Vec3A::new(1., 1., 1.) * self.half_extents),
                self.center + (Vec3A::new(1., 1., -1.) * self.half_extents),
                self.center + (Vec3A::new(1., -1., 1.) * self.half_extents),
                self.center + (Vec3A::new(-1., 1., 1.) * self.half_extents),
                self.center + (Vec3A::new(1., -1., -1.) * self.half_extents),
                self.center + (Vec3A::new(-1., 1., -1.) * self.half_extents),
                self.center + (Vec3A::new(-1., -1., 1.) * self.half_extents),
                self.center + (Vec3A::new(-1., -1., -1.) * self.half_extents),
            ]
            .into_iter()
            .map(|point| transform.transform_point(point))
            .map(Vec3::from),
        )
        .expect("Aabb is calculated from rotated vertices.")
    }
}
