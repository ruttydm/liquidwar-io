use std::f32::consts::PI;

pub struct KernelCoeffs {
    pub poly6: f32,
    pub spiky_grad: f32,
    pub visc_lap: f32,
    pub h: f32,
    pub h_sq: f32,
}

impl KernelCoeffs {
    pub fn new(h: f32) -> Self {
        KernelCoeffs {
            poly6: 315.0 / (64.0 * PI * h.powi(9)),
            spiky_grad: -45.0 / (PI * h.powi(6)),
            visc_lap: 45.0 / (PI * h.powi(6)),
            h,
            h_sq: h * h,
        }
    }

    /// Poly6 kernel for density estimation. Takes r^2 to avoid sqrt.
    #[inline(always)]
    pub fn poly6(&self, r_sq: f32) -> f32 {
        if r_sq >= self.h_sq {
            return 0.0;
        }
        self.poly6 * (self.h_sq - r_sq).powi(3)
    }

    /// Spiky gradient kernel for pressure forces. Returns scalar factor.
    #[inline(always)]
    pub fn spiky_gradient(&self, r: f32) -> f32 {
        if r >= self.h || r < 1e-6 {
            return 0.0;
        }
        self.spiky_grad * (self.h - r).powi(2)
    }

    /// Viscosity laplacian kernel.
    #[inline(always)]
    pub fn viscosity_laplacian(&self, r: f32) -> f32 {
        if r >= self.h {
            return 0.0;
        }
        self.visc_lap * (self.h - r)
    }
}
