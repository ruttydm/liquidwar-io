use crate::constants::*;
use crate::grid::SpatialGrid;
use crate::kernels::KernelCoeffs;
use crate::particle::Particles;

pub struct SphSimulation {
    pub particles: Particles,
    grid: SpatialGrid,
    kernels: KernelCoeffs,
    pub position_buffer: Vec<f32>,
    pub speed_buffer: Vec<f32>,
}

impl SphSimulation {
    pub fn new(target_count: usize) -> Self {
        let particles = Particles::new_dam_break(target_count);
        let count = particles.count;
        let grid = SpatialGrid::new(count);
        let kernels = KernelCoeffs::new(SMOOTHING_RADIUS);

        let mut sim = SphSimulation {
            particles,
            grid,
            kernels,
            position_buffer: vec![0.0; count * 3],
            speed_buffer: vec![0.0; count],
        };
        sim.fill_buffers();
        sim
    }

    pub fn step(&mut self) {
        self.grid.rebuild(
            &self.particles.pos_x,
            &self.particles.pos_y,
            &self.particles.pos_z,
            self.particles.count,
        );
        self.compute_density_pressure();
        self.compute_forces();
        self.integrate();
        self.enforce_boundaries();
    }

    fn compute_density_pressure(&mut self) {
        let h_sq = self.kernels.h_sq;
        let count = self.particles.count;

        for i in 0..count {
            let mut density = 0.0_f32;
            let px = self.particles.pos_x[i];
            let py = self.particles.pos_y[i];
            let pz = self.particles.pos_z[i];

            self.grid.for_each_neighbor(px, py, pz, |j| {
                let dx = px - self.particles.pos_x[j];
                let dy = py - self.particles.pos_y[j];
                let dz = pz - self.particles.pos_z[j];
                let r_sq = dx * dx + dy * dy + dz * dz;

                if r_sq < h_sq {
                    density += PARTICLE_MASS * self.kernels.poly6(r_sq);
                }
            });

            self.particles.density[i] = density;
            self.particles.pressure[i] = (GAS_CONSTANT * (density - REST_DENSITY)).max(0.0);
        }
    }

    fn compute_forces(&mut self) {
        let h = self.kernels.h;
        let count = self.particles.count;

        for i in 0..count {
            let mut fx = 0.0_f32;
            let mut fy = GRAVITY * self.particles.density[i];
            let mut fz = 0.0_f32;

            let px = self.particles.pos_x[i];
            let py = self.particles.pos_y[i];
            let pz = self.particles.pos_z[i];
            let pi_pressure = self.particles.pressure[i];
            let _pi_density = self.particles.density[i];

            self.grid.for_each_neighbor(px, py, pz, |j| {
                if i == j {
                    return;
                }

                let dx = px - self.particles.pos_x[j];
                let dy = py - self.particles.pos_y[j];
                let dz = pz - self.particles.pos_z[j];
                let r_sq = dx * dx + dy * dy + dz * dz;
                let r = r_sq.sqrt();

                if r < h && r > 1e-6 {
                    let pj_density = self.particles.density[j];
                    let pj_pressure = self.particles.pressure[j];

                    // Pressure force
                    let pressure_term = -PARTICLE_MASS
                        * (pi_pressure + pj_pressure)
                        / (2.0 * pj_density)
                        * self.kernels.spiky_gradient(r);

                    let inv_r = 1.0 / r;
                    fx += pressure_term * dx * inv_r;
                    fy += pressure_term * dy * inv_r;
                    fz += pressure_term * dz * inv_r;

                    // Viscosity force
                    let visc_term = VISCOSITY * PARTICLE_MASS / pj_density
                        * self.kernels.viscosity_laplacian(r);

                    fx += visc_term * (self.particles.vel_x[j] - self.particles.vel_x[i]);
                    fy += visc_term * (self.particles.vel_y[j] - self.particles.vel_y[i]);
                    fz += visc_term * (self.particles.vel_z[j] - self.particles.vel_z[i]);
                }
            });

            self.particles.force_x[i] = fx;
            self.particles.force_y[i] = fy;
            self.particles.force_z[i] = fz;
        }
    }

    fn integrate(&mut self) {
        for i in 0..self.particles.count {
            let inv_density = 1.0 / self.particles.density[i].max(1e-6);

            let ax = self.particles.force_x[i] * inv_density;
            let ay = self.particles.force_y[i] * inv_density;
            let az = self.particles.force_z[i] * inv_density;

            self.particles.vel_x[i] += ax * DT;
            self.particles.vel_y[i] += ay * DT;
            self.particles.vel_z[i] += az * DT;

            self.particles.pos_x[i] += self.particles.vel_x[i] * DT;
            self.particles.pos_y[i] += self.particles.vel_y[i] * DT;
            self.particles.pos_z[i] += self.particles.vel_z[i] * DT;
        }
    }

    fn enforce_boundaries(&mut self) {
        for i in 0..self.particles.count {
            // Floor
            if self.particles.pos_y[i] < BOUNDARY_EPSILON {
                self.particles.pos_y[i] = BOUNDARY_EPSILON;
                self.particles.vel_y[i] *= WALL_DAMPING;
            }
            // Ceiling
            if self.particles.pos_y[i] > DOMAIN_HEIGHT - BOUNDARY_EPSILON {
                self.particles.pos_y[i] = DOMAIN_HEIGHT - BOUNDARY_EPSILON;
                self.particles.vel_y[i] *= WALL_DAMPING;
            }
            // X walls
            if self.particles.pos_x[i] < BOUNDARY_EPSILON {
                self.particles.pos_x[i] = BOUNDARY_EPSILON;
                self.particles.vel_x[i] *= WALL_DAMPING;
            }
            if self.particles.pos_x[i] > DOMAIN_WIDTH - BOUNDARY_EPSILON {
                self.particles.pos_x[i] = DOMAIN_WIDTH - BOUNDARY_EPSILON;
                self.particles.vel_x[i] *= WALL_DAMPING;
            }
            // Z walls
            if self.particles.pos_z[i] < BOUNDARY_EPSILON {
                self.particles.pos_z[i] = BOUNDARY_EPSILON;
                self.particles.vel_z[i] *= WALL_DAMPING;
            }
            if self.particles.pos_z[i] > DOMAIN_DEPTH - BOUNDARY_EPSILON {
                self.particles.pos_z[i] = DOMAIN_DEPTH - BOUNDARY_EPSILON;
                self.particles.vel_z[i] *= WALL_DAMPING;
            }
        }
    }

    pub fn fill_buffers(&mut self) {
        for i in 0..self.particles.count {
            let idx = i * 3;
            self.position_buffer[idx] = self.particles.pos_x[i];
            self.position_buffer[idx + 1] = self.particles.pos_y[i];
            self.position_buffer[idx + 2] = self.particles.pos_z[i];

            let vx = self.particles.vel_x[i];
            let vy = self.particles.vel_y[i];
            let vz = self.particles.vel_z[i];
            self.speed_buffer[i] = (vx * vx + vy * vy + vz * vz).sqrt();
        }
    }
}
