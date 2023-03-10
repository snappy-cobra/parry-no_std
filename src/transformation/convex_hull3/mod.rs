pub use self::error::ConvexHullError;
pub(self) use self::initial_mesh::{try_get_initial_mesh, InitialMesh};
pub(self) use self::triangle_facet::TriangleFacet;
pub(self) use self::validation::check_facet_links;
pub use convex_hull::{convex_hull, try_convex_hull};
pub use validation::check_convex_hull;

mod convex_hull;
mod error;
mod initial_mesh;
mod triangle_facet;
mod validation;
