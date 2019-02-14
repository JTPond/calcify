use std::f64;

/// Re-export Rust's PI
pub const PI: f64 = std::f64::consts::PI;

/// The speed of light in a vacuum in SI (299_792_458.0 m/s)
pub const C_LIGHT: f64 = 299_792_458.0;

/// Boltzmann constant in SI (1.380_650_4e-23 J/K)
pub const K_BOLTZ: f64 = 1.380_650_4e-23;

/// Elementary charge in SI (1.602_176_487e-19 C)
pub const E_CHARGE: f64 = 1.602_176_487e-19;

/// Fine structure constant (7.297_352_537_6e-3 unitless)
pub const Q_ALPHA: f64 = 7.297_352_537_6e-3;

/// Gravitational coupling constant (1.752e-45 unitless)
pub const G_ALPHA: f64 = 1.752e-45;

/// **Reduced** Planck constant in SI (6.626_068_96e-34/2π J*S)
pub const H_BAR: f64 = 6.626_068_96e-34/(2.0*PI);

/// Permittivity of free space in SI (8.854_187_817e-12 F/m)
pub const EP_NAUGHT: f64 = 8.854_187_817e-12;

/// Permeability of free space in SI (4π*10^-7 N/A^2)
pub const MU_NAUGHT: f64 = 4.0*PI*10.0e7;

/// Gravitational constant in SI (6.674_28e-11 m^3/(kg*s^2))
pub const BIG_G: f64 = 6.674_28e-11;

/// Proton mass in SI (1.672_621_637e-27 kg)
pub const M_PROTON: f64 = 1.672_621_637e-27;

/// Electron mass in SI (9.109_382_15e-31 kg)
pub const M_ELECTRON: f64 = 9.109_382_15e-31;

/// Calcify consts in different unit systems
pub struct Consts {
    pub c_light: f64,
    pub k_boltz: f64,
    pub e_charge: f64,
    pub q_alpha: f64,
    pub g_alpha: f64,
    pub h_bar: f64,
    pub ep_naught: f64,
    pub mu_naught: f64,
    pub big_g: f64,
    pub m_proton: f64,
    pub m_electron: f64,
}

impl Consts {
    /// Return all calcify constants in a Consts struct in Planck Lorentz–Heaviside units
    ///
    /// # Example
    /// ```
    /// use calcify::consts::Consts;
    ///
    /// let consts_planck = Consts::planck();
    /// let c = consts_planck.c_light;
    /// let hb = consts_planck.h_bar;
    /// assert_eq!(c,1.0);
    /// assert_eq!(hb,1.0);
    /// ```
    pub fn planck() -> Consts {
        Consts {
            c_light : 1.0,
            k_boltz : 1.0,
            e_charge : (4.0*PI*Q_ALPHA).sqrt(),
            q_alpha : Q_ALPHA,
            g_alpha : G_ALPHA,
            h_bar : 1.0,
            ep_naught : 1.0,
            mu_naught : 1.0,
            big_g : 1.0/(4.0*PI),
            m_proton : 1_836.152_672_47*(4.0*PI*G_ALPHA).sqrt(),
            m_electron : (4.0*PI*G_ALPHA).sqrt(),
        }
    }

    /// Return all calcify constants in a Consts struct in natural Lorentz–Heaviside units
    pub fn natural() -> Consts {
        Consts {
            c_light : 1.0,
            k_boltz : 1.0,
            e_charge : (4.0*PI*Q_ALPHA).sqrt(),
            q_alpha : Q_ALPHA,
            g_alpha : G_ALPHA,
            h_bar : 1.0,
            ep_naught : 1.0,
            mu_naught : 1.0,
            big_g : G_ALPHA/(511_000.0*511_000.0),
            m_proton : 938_000_000.0,
            m_electron : 511_000.0,
        }
    }
}
