use serde::{Deserialize,Serialize}; use crate::geom::{FillPolyline,LayerSurface};
#[derive(Clone,Debug,Default,Serialize,Deserialize)] pub struct CollisionReport{pub clear:bool,pub notes:Vec<String>}
pub fn check_collision_stub(_layers:&[LayerSurface],_fills:&[FillPolyline])->CollisionReport{CollisionReport{clear:true,notes:vec!["collision stub: always clear; pair with #16/#68 for production sequencing".into()]}}
