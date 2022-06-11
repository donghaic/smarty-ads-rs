use crate::model::RangeValue;

pub mod exp_driver;
pub mod prodiction;

pub use prodiction::*;

pub fn find_target_val(cfgs: &Vec<RangeValue>, target: f64) -> f64 {
    for cfg in cfgs {
        if cfg.min >= target && target < cfg.max {
            return cfg.value;
        }
    }

    return 0.0;
}
