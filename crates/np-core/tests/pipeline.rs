use np_core::*;
#[test] fn empty_input_yields_empty_output(){let out=run(&CoreInput::default());assert!(out.layers.is_empty());assert!(out.tcp.points.is_empty());}
