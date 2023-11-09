use bevy::prelude::*;
// use bevy_prototype_debug_lines::DebugLines;

pub trait DebugLinesExt {
    fn cross(&mut self, p: Vec3, duration: f32);
}

// impl DebugLinesExt for DebugLines {
//     fn cross(&mut self, p: Vec3, duration: f32) {
//         let s = 16.0;
//         let c0 = Vec3::new(-s, s, 0.0);
//         let c1 = Vec3::new(s, s, 0.0);

//         let start = p + c0;
//         let end = p - c0;
//         // let zoff = 5.0;
//         // start.z = zoff;
//         // end.z = zoff;
//         self.line(start, end, duration);

//         let start = p + c1;
//         let end = p - c1;
//         // start.z = zoff;
//         // end.z = zoff;
//         self.line(start, end, duration);
//     }
// }
