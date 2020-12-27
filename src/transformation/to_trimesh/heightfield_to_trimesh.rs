use crate::shape::HeightField;
use na::Point3;

impl HeightField {
    /// Converts this height-field to a triangle-mesh.
    pub fn to_trimesh(&self) -> (Vec<Point3<f32>>, Vec<Point3<u32>>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for (i, tri) in self.triangles().enumerate() {
            vertices.push(tri.a);
            vertices.push(tri.b);
            vertices.push(tri.c);

            let i = i as u32;
            indices.push(Point3::new(i * 3, i * 3 + 1, i * 3 + 2))
        }

        (vertices, indices)
    }
}