// Simulation domain
pub const DOMAIN_WIDTH: f32 = 4.0;
pub const DOMAIN_HEIGHT: f32 = 6.0;
pub const DOMAIN_DEPTH: f32 = 4.0;

// SPH parameters
pub const PARTICLE_MASS: f32 = 1.0;
pub const REST_DENSITY: f32 = 1000.0;
pub const GAS_CONSTANT: f32 = 2000.0;
pub const SMOOTHING_RADIUS: f32 = 0.1;
pub const VISCOSITY: f32 = 250.0;
pub const GRAVITY: f32 = -9.81;
pub const DT: f32 = 0.0008;
pub const SUBSTEPS: u32 = 4;

// Boundary
pub const WALL_DAMPING: f32 = -0.5;
pub const BOUNDARY_EPSILON: f32 = 0.001;
