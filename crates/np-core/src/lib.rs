pub mod clamp; pub mod collision; pub mod fea; pub mod field; pub mod fill; pub mod geom; pub mod isosurface; pub mod pipeline; pub mod tcp;
pub use geom::{FillPolyline,LayerSurface,TcpPath,ToolpathPoint,Vec3}; pub use fea::{PrincipalStressSample,StressGrid,StressIn}; pub use field::GoverningScalar; pub use pipeline::{CoreInput,CoreOutput,GoverningScalarSummary};
pub fn run(input:&CoreInput)->CoreOutput{pipeline::run(input)}
