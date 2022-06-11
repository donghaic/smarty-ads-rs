use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub usr: String,
    pub ad_id: Vec<i64>,
    pub service_type: i64,
    pub model: String,
    pub is_debug: bool,
}

impl Request {
    pub fn check(&self) -> (bool, String) {
        if self.usr.is_empty() {
            return (false, "用户账号不能为空".to_string());
        } else if self.ad_id.is_empty() {
            return (false, "广告ID列表不能为空".to_string());
        } else if self.service_type == 0 {
            return (false, "ServiceType只能为1或2".to_string());
        }
        return (true, "".to_string());
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub code: i32,
    pub msg: String,
    pub items: Vec<AdItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdItem {
    pub ad_id: i64,
    pub value: u8,
}

impl AdItem {
    pub fn new(ad_id: i64, value: u8) -> Self {
        Self { ad_id, value }
    }
}

pub struct AdEvent {
    pub request: i64,
    pub fill: i64,
    pub show: i64,
    pub click: i64,
}

impl AdEvent {
    pub fn get_fill_rate(&self, ab_params: &AbParams) -> f64 {
        (self.fill as f64 + ab_params.fill_a) / ((self.request + 1) as f64 + ab_params.fill_b)
    }

    pub fn get_show_rate(&self, ab_params: &AbParams) -> f64 {
        (self.show as f64 + ab_params.show_a) / ((self.fill + 1) as f64 + ab_params.show_b)
    }

    pub fn get_click_rate(&self, ab_params: &AbParams) -> f64 {
        (self.click as f64 + ab_params.click_a) / ((self.show + 1) as f64 + ab_params.click_b)
    }

    pub fn get_click_rate_without_ab(&self) -> f64 {
        (self.click as f64) / (self.show as f64 + 1.0)
    }
}

pub struct AbParams {
    pub fill_a: f64,
    pub fill_b: f64,
    pub show_a: f64,
    pub show_b: f64,
    pub click_a: f64,
    pub click_b: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpBaseCfg {
    pub version: String,
    pub base_value: f64,
    pub score_factor: f64,
    pub start_time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeValue {
    pub min: f64,
    pub max: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdIdExpCfg {
    pub ad_id: i64,
    pub version: String,
    pub cg_user: String,
    pub eg_user: String,
    pub eg_action_id: String,
    pub main_action_id: String,
    pub exp_action_value: f64,
    pub main_action_value: f64,
}

impl AdIdExpCfg {
    pub fn is_empty(&self) -> bool {
        self.ad_id == 0 || self.version.is_empty()
    }

    pub fn is_control_group(&self, ug: &str) -> bool {
        ug == self.cg_user
    }

    pub fn is_exp_group(&self, ug: &str) -> bool {
        ug == self.eg_user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_new_request() {
        let req = Request {
            usr: "i123".to_string(),
            ad_id: vec![1, 2, 3],
            service_type: 1,
            model: "".to_string(),
            is_debug: false,
        };

        println!("Creating new request {:?}", req);

        let (ok, msg) = req.check();
        println!(" check-{}, msg={} ", ok, msg)
    }

    #[test]
    fn test_exp_config() {
        let cfg = ExpBaseCfg {
            version: "1".to_string(),
            base_value: 0.123,
            score_factor: 0.90,
            start_time: Local::now(),
        };
        let cfg_str = serde_json::to_string(&cfg).unwrap();

        println!("{}", cfg_str);
        println!("{:?}", cfg);
    }

    #[test]
    fn test_adid_exp_cfg() {
        let cfg = AdIdExpCfg {
            ad_id: 123,
            version: "1".to_string(),
            cg_user: "1".to_string(),
            eg_user: "f".to_string(),
            eg_action_id: "1".to_string(),
            main_action_id: "12".to_string(),
            exp_action_value: 0.3,
            main_action_value: 0.54,
        };

        let cfg_str = serde_json::to_string(&cfg).unwrap();
        println!("{}", cfg_str);

        let cfg1: AdIdExpCfg = serde_json::from_str(cfg_str.as_str()).unwrap();
        println!("deserde json => {:?}", cfg1)
    }
}
