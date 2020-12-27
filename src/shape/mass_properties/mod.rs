pub use self::mass_properties::MassProperties;

mod mass_properties;
mod mass_properties_ball;
mod mass_properties_capsule;
#[cfg(feature = "dim3")]
mod mass_properties_cone;
#[cfg(featuer = "dim2")]
mod mass_properties_convex_polygon;
#[cfg(feature = "dim2")]
mod mass_properties_convex_polygon;
#[cfg(feature = "dim3")]
mod mass_properties_convex_polyhedron;
mod mass_properties_cuboid;
mod mass_properties_cylinder;