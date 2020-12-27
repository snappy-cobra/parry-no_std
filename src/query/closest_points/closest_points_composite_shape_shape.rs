use crate::bounding_volume::SimdAABB;
use crate::math::{Isometry, Real, SimdBool, SimdReal, Vector, SIMD_WIDTH};
use crate::partitioning::{SimdBestFirstVisitStatus, SimdBestFirstVisitor};
use crate::query::{ClosestPoints, QueryDispatcher};
use crate::shape::{Shape, SimdCompositeShape};
use na;
use simba::simd::{SimdBool as _, SimdPartialOrd, SimdValue};

/// Closest points between a composite shape and any other shape.
pub fn closest_points_composite_shape_shape<D: ?Sized, G1: ?Sized>(
    dispatcher: &D,
    pos12: &Isometry<Real>,
    g1: &G1,
    g2: &dyn Shape,
    margin: Real,
) -> ClosestPoints
where
    D: QueryDispatcher,
    G1: SimdCompositeShape,
{
    let mut visitor =
        CompositeShapeAgainstShapeClosestPointsVisitor::new(dispatcher, pos12, g1, g2, margin);

    g1.quadtree()
        .traverse_best_first(&mut visitor)
        .expect("The composite shape must not be empty.")
        .1
}

/// Closest points between a shape and a composite shape.
pub fn closest_points_shape_composite_shape<D: ?Sized, G2: ?Sized>(
    dispatcher: &D,
    pos12: &Isometry<Real>,
    g1: &dyn Shape,
    g2: &G2,
    margin: Real,
) -> ClosestPoints
where
    D: QueryDispatcher,
    G2: SimdCompositeShape,
{
    closest_points_composite_shape_shape(dispatcher, &pos12.inverse(), g2, g1, margin).flipped()
}

struct CompositeShapeAgainstShapeClosestPointsVisitor<'a, D: ?Sized, G1: ?Sized + 'a> {
    msum_shift: Vector<SimdReal>,
    msum_margin: Vector<SimdReal>,
    margin: Real,

    dispatcher: &'a D,
    pos12: &'a Isometry<Real>,
    g1: &'a G1,
    g2: &'a dyn Shape,
}

impl<'a, D: ?Sized, G1: ?Sized> CompositeShapeAgainstShapeClosestPointsVisitor<'a, D, G1>
where
    D: QueryDispatcher,
    G1: SimdCompositeShape,
{
    pub fn new(
        dispatcher: &'a D,
        pos12: &'a Isometry<Real>,
        g1: &'a G1,
        g2: &'a dyn Shape,
        margin: Real,
    ) -> CompositeShapeAgainstShapeClosestPointsVisitor<'a, D, G1> {
        let ls_aabb2 = g2.compute_aabb(pos12);

        CompositeShapeAgainstShapeClosestPointsVisitor {
            msum_shift: Vector::splat(-ls_aabb2.center().coords),
            msum_margin: Vector::splat(ls_aabb2.half_extents()),
            margin,
            dispatcher,
            pos12,
            g1,
            g2,
        }
    }
}

impl<'a, D: ?Sized, G1: ?Sized> SimdBestFirstVisitor<u32, SimdAABB>
    for CompositeShapeAgainstShapeClosestPointsVisitor<'a, D, G1>
where
    D: QueryDispatcher,
    G1: SimdCompositeShape,
{
    type Result = ClosestPoints;

    fn visit(
        &mut self,
        best: Real,
        bv: &SimdAABB,
        data: Option<[Option<&u32>; SIMD_WIDTH]>,
    ) -> SimdBestFirstVisitStatus<Self::Result> {
        // Compute the minkowski sum of the two AABBs.
        let msum = SimdAABB {
            mins: bv.mins + self.msum_shift + (-self.msum_margin),
            maxs: bv.maxs + self.msum_shift + self.msum_margin,
        };
        let dist = msum.distance_to_origin();
        let mask = dist.simd_lt(SimdReal::splat(best));

        if let Some(data) = data {
            let bitmask = mask.bitmask();
            let mut weights = [0.0; SIMD_WIDTH];
            let mut mask = [false; SIMD_WIDTH];
            let mut results = [None; SIMD_WIDTH];
            let mut found_intersection = false;

            for ii in 0..SIMD_WIDTH {
                if (bitmask & (1 << ii)) != 0 && data[ii].is_some() {
                    self.g1.map_part_at(*data[ii].unwrap(), &mut |g1| {
                        let pts =
                            self.dispatcher
                                .closest_points(&self.pos12, g1, self.g2, self.margin);
                        match pts {
                            Ok(ClosestPoints::WithinMargin(ref p1, ref p2)) => {
                                let p2_1 = self.pos12 * p2;
                                weights[ii] = na::distance(p1, &p2_1);
                                results[ii] = Some(ClosestPoints::WithinMargin(*p1, *p2));
                                mask[ii] = true;
                            }
                            Ok(ClosestPoints::Intersecting) => {
                                found_intersection = true;
                            }
                            Err(_) | Ok(ClosestPoints::Disjoint) => {}
                        };
                    });

                    if found_intersection {
                        return SimdBestFirstVisitStatus::ExitEarly(Some(
                            ClosestPoints::Intersecting,
                        ));
                    }
                }
            }

            SimdBestFirstVisitStatus::MaybeContinue {
                weights: SimdReal::from(weights),
                mask: SimdBool::from(mask),
                results,
            }
        } else {
            SimdBestFirstVisitStatus::MaybeContinue {
                weights: dist,
                mask,
                results: [None; SIMD_WIDTH],
            }
        }
    }
}