use crate::constants::*;

pub struct Particles {
    pub count: usize,
    pub pos_x: Vec<f32>,
    pub pos_y: Vec<f32>,
    pub pos_z: Vec<f32>,
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    pub vel_z: Vec<f32>,
    pub force_x: Vec<f32>,
    pub force_y: Vec<f32>,
    pub force_z: Vec<f32>,
    pub density: Vec<f32>,
    pub pressure: Vec<f32>,
}

impl Particles {
    pub fn new_dam_break(target_count: usize) -> Self {
        let spacing = 0.8 * SMOOTHING_RADIUS;

        // Fill the left-bottom portion of the domain
        let fill_x = DOMAIN_WIDTH * 0.45;
        let fill_y = DOMAIN_HEIGHT * 0.5;
        let fill_z = DOMAIN_DEPTH * 0.45;

        let nx = (fill_x / spacing) as usize;
        let ny = (fill_y / spacing) as usize;
        let nz = (fill_z / spacing) as usize;

        let max_particles = nx * ny * nz;
        let count = target_count.min(max_particles);

        let mut pos_x = Vec::with_capacity(count);
        let mut pos_y = Vec::with_capacity(count);
        let mut pos_z = Vec::with_capacity(count);

        let offset = BOUNDARY_EPSILON + spacing * 0.5;
        let mut placed = 0;

        'outer: for iy in 0..ny {
            for ix in 0..nx {
                for iz in 0..nz {
                    if placed >= count {
                        break 'outer;
                    }
                    pos_x.push(offset + ix as f32 * spacing);
                    pos_y.push(offset + iy as f32 * spacing);
                    pos_z.push(offset + iz as f32 * spacing);
                    placed += 1;
                }
            }
        }

        let count = placed;

        Particles {
            count,
            pos_x,
            pos_y,
            pos_z,
            vel_x: vec![0.0; count],
            vel_y: vec![0.0; count],
            vel_z: vec![0.0; count],
            force_x: vec![0.0; count],
            force_y: vec![0.0; count],
            force_z: vec![0.0; count],
            density: vec![0.0; count],
            pressure: vec![0.0; count],
        }
    }
}
