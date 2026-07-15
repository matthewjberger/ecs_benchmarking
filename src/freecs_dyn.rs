// The dynamic tier (freecs::dynamic::DynWorld) with runtime-registered
// components, typed reference queries, and staged systems. This is the tier
// Nightshade actually runs on, so it is the one worth measuring.
//
// Component fields are payload the ECS stores and moves; they are written on
// spawn and exercised through storage rather than read via field access.
#![allow(dead_code)]

pub mod add_remove;
pub mod frag_iter;
pub mod heavy_compute;
pub mod schedule;
pub mod simple_insert;
pub mod simple_iter;
