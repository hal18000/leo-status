use leo_status_driver::{GpsdoConfig, GpsdoStatus};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub(crate) struct LockStatusResponse {
    /// The number of times that the GPS lock has been lost since reboot
    loss_count: u8,

    /// Whether the GPSDO has a lock on a GPS signal
    sat_lock: bool,

    /// Whether the PLL is locked to the configured frequencies
    pll_lock: bool,

    /// Whether the system is locked overall
    locked: bool,
}

impl From<GpsdoStatus> for LockStatusResponse {
    fn from(value: GpsdoStatus) -> Self {
        LockStatusResponse {
            loss_count: value.loss_count(),
            sat_lock: value.sat_locked(),
            pll_lock: value.pll_locked(),
            locked: value.locked(),
        }
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct PllParamsResponse {
    /// The frequency produced by the GPSDO TCXO
    fin: u32,

    /// The divisor of fin before it enters the PLL
    n3: u32,
    /// The first divisor on the feedback loop
    n2_hs: u8,

    /// The second divisor on the feedback loop
    n2_ls: u32,

    /// The shared divisor on the output from the PLL
    n1_hs: u8,

    /// The divisor after n1_hs, heading to port one
    nc1_ls: u32,

    /// The divisor after n1_hs, heading to port two
    nc2_ls: u32,

    /// The skew between port one and port two, 0 - 255
    skew: u8,

    /// The PLL bandwidth mode
    bw: u8,

    /// The frequency of fin after division by n3
    f3: u32,

    /// The frequency of the output of the PLL
    fosc: u64,
}

impl From<GpsdoConfig> for PllParamsResponse {
    fn from(value: GpsdoConfig) -> Self {
        PllParamsResponse {
            fin: value.fin(),
            n3: value.n3(),
            n2_hs: value.n2_hs(),
            n2_ls: value.n2_ls(),
            n1_hs: value.n1_hs(),
            nc1_ls: value.nc1_ls(),
            nc2_ls: value.nc2_ls(),
            skew: value.skew(),
            bw: value.bw(),
            f3: value.f3(),
            fosc: value.fosc(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ConfigResponse {
    /// Whether the output1 port of the GPSDO is active
    output1: bool,

    /// Whether the output2 port of the GPSDO is active
    output2: bool,

    /// The drive level of the signal in milliamps
    level: u8,

    pll_params: PllParamsResponse,

    /// The frequency output on output1
    fout1: u64,

    /// The frequency output on output2
    fout2: u64,
}

impl From<GpsdoConfig> for ConfigResponse {
    fn from(value: GpsdoConfig) -> Self {
        ConfigResponse {
            output1: value.output1(),
            output2: value.output2(),
            level: match value.level() {
                0 => 8,
                1 => 16,
                2 => 24,
                3 => 32,
                _ => 0,
            },
            fout1: value.fout1(),
            fout2: value.fout2(),
            pll_params: value.into(),
        }
    }
}
