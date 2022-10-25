use std::f32::consts::{PI, FRAC_PI_2};

pub fn get_angle(cx: f32, cy: f32, ex: f32, ey: f32) -> f32 {
    let dy = ey - cy;
    let dx = ex - cx;

    let angle = match dx != 0.0 {
        // Returns the angle in radians
        true => (dy / dx).atan(),
        false => match dy > 0.0 {
            true => FRAC_PI_2, 
            false => PI,
        },
    };

    match cx < ex {
        true => angle - PI,
        false => angle,

    }
    
}
