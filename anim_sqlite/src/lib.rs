#![warn(bare_trait_objects)]

extern crate flo_animation;
extern crate flo_canvas;

extern crate itertools;
extern crate rusqlite;
extern crate futures;
extern crate desync;

mod db;
mod animation_sqlite;

pub use self::animation_sqlite::*;
