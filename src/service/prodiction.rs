// #![allow(dead_code)]
// #![allow(unused_variables)]

use rand::prelude::*;

use crate::dao::*;
use crate::model::*;
use md5;

#[derive(Clone)]
pub struct ProdictionService {
    ads_dao: AdsDB,
}

impl ProdictionService {
    pub fn new(ads_dao: AdsDB) -> Self {
        Self { ads_dao }
    }

    pub fn predict(&self, request: &Request) -> Response {
        let usrhash = md5::compute(&request.usr);
        let usr_md5 = format!("{:x}", usrhash);
        let usergroup = &usr_md5[usr_md5.len() - 1..];
        let exp_base_cfg: ExpBaseCfg = self.ads_dao.get_exp_base_cfg();
        let ab_params: AbParams = self.ads_dao.get_exp_ab_params();
        self.ads_dao
            .add_adids_to_localcache(&exp_base_cfg.version, &request.ad_id);
        let user_daily_total_tempt_click = self.ads_dao.query_temp_click(&request.usr);

        //let adid_whitelist: HashSet<u64> = self.ads_dao.get_adid_whitelist();
        let tempt_click_cfg: Vec<RangeValue> = self.ads_dao.get_signal_daily_total_tempt_click();
        let adid_fill_rate_cfg: Vec<RangeValue> = self.ads_dao.get_signal_ad_id_fill_rate();
        let adid_show_rate_cfg: Vec<RangeValue> = self.ads_dao.get_signal_ad_id_show_rate();
        let adid_click_rate_cfg: Vec<RangeValue> = self.ads_dao.get_signal_ad_id_click_rate();

        let rate_a = super::find_target_val(&tempt_click_cfg, user_daily_total_tempt_click as f64);

        let date = chrono::Local::now().to_rfc2822();

        let mut predictions = Vec::new();
        for adid in request.ad_id.iter() {
            let ad_id_realtime_event: AdEvent = self
                .ads_dao
                .get_realtime_ad_id_window_events(usergroup, *adid);
            let user_daily_ad_id_event: AdEvent =
                self.ads_dao
                    .get_user_daily_ad_id_event(*adid, &request.usr, &date);

            let fill_rate = user_daily_ad_id_event.get_fill_rate(&ab_params);
            let show_rate = user_daily_ad_id_event.get_show_rate(&ab_params);
            let click_rate = user_daily_ad_id_event.get_click_rate(&ab_params);
            let rate_b = super::find_target_val(&adid_fill_rate_cfg, fill_rate);
            let rate_c = super::find_target_val(&adid_show_rate_cfg, show_rate);
            let rate_d = super::find_target_val(&adid_click_rate_cfg, click_rate);
            let window_ctr = ad_id_realtime_event.get_click_rate_without_ab();

            let ad_exp_cfg = self.ads_dao.get_adid_exp_cfg(&exp_base_cfg.version, *adid);

            // 目标CTR
            let target_ctr = if ad_exp_cfg.is_exp_group(usergroup) {
                ad_exp_cfg.exp_action_value // 如果是试验组
            } else {
                ad_exp_cfg.main_action_value // 对照与主版本
            };

            let mut total_rate = rate_a * rate_b * rate_c * rate_d;
            // 区间判断 [-N,-30, 30,+N]
            if window_ctr < target_ctr {
                if (window_ctr + target_ctr * 0.3) < target_ctr {
                    // 策略A
                    total_rate = total_rate * 2 as f64
                }
            } else {
                if (target_ctr + target_ctr * 0.3) < window_ctr {
                    // 策略C
                    total_rate = total_rate * 0.5
                }
            }

            // 随机预估
            let prediction = if total_rate >= exp_base_cfg.base_value {
                let mut rng = rand::thread_rng();
                let probability: f64 = rng.gen();
                let state = if total_rate >= probability { 1 } else { 0 };
                state
            } else {
                0
            };

            predictions.push(AdItem::new(*adid, prediction));
        }

        Response {
            code: 0,
            msg: "".to_string(),
            items: predictions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5() {
        let usrhash = md5::compute("1234");
        let usr_md5 = format!("{:x}", usrhash);
        let usergroup = &usr_md5[usr_md5.len() - 1..];
        println!("{}", usr_md5);
        println!("{}", usergroup);
    }
}
