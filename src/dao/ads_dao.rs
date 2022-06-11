use std::cmp::Ordering;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;

use chrono::DateTime;
use chrono::Local;
use moka::sync::Cache;

use crate::dao::*;
use crate::model::*;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AdsDB {
    pub dyn_cfg: DyncConfigV2,
    pub redis_dao: RedisDao,
    adid_cache: Cache<String, i64>,
    adid_experiment_cache: Cache<String, AdIdExpCfg>,
}

impl AdsDB {
    pub fn new(redis_client: redis::Client) -> Self {
        let adid_cache = Cache::builder()
            .time_to_live(Duration::from_secs(24 * 60 * 60)) // Time to live (TTL): 30 minutes
            .time_to_idle(Duration::from_secs(24 * 60 * 60)) // Time to idle (TTI):  5 minutes
            .build(); // Create the cache.

        let adid_experiment_cache = Cache::builder()
            .time_to_live(Duration::from_secs(24 * 60 * 60))
            .time_to_idle(Duration::from_secs(24 * 60 * 60))
            .build(); // Create the cache.

        AdsDB {
            dyn_cfg: DyncConfigV2::new(redis_client.clone()),
            redis_dao: RedisDao::new(redis_client.clone()),
            adid_cache,
            adid_experiment_cache,
        }
    }

    pub fn add_adids_to_localcache(&self, version: &str, ad_id: &Vec<i64>) {
        for ad_id in ad_id {
            self.adid_cache
                .insert(format!("{}:{}", version, ad_id.to_string()), *ad_id);
        }
    }

    pub fn get_version_adids_from_localcache(&self, version: &str) -> Vec<i64> {
        let mut ad_ids: Vec<i64> = Vec::new();
        for (key, value) in self.adid_cache.iter() {
            if key.starts_with(&version) {
                ad_ids.push(value);
            }
        }
        ad_ids
    }

    pub fn get_realtime_adids_event(&self, keys: Vec<&str>) -> Vec<AdEvent> {
        let mut ad_events: Vec<AdEvent> = Vec::new();

        match self.redis_dao.get_multi_event_by_keys(keys) {
            Ok(events) => {
                for event_str in events {
                    let fields: Vec<_> = event_str.split("_").collect();
                    if fields.len() < 4 {
                        continue;
                    }
                    let ad_event = AdEvent {
                        request: fields[0].parse().unwrap_or_default(),
                        fill: fields[1].parse().unwrap_or_default(),
                        show: fields[2].parse().unwrap_or_default(),
                        click: fields[3].parse().unwrap_or_default(),
                    };
                    ad_events.push(ad_event);
                }
            }
            Err(e) => {
                log::error!("get_realtime_adids_event error: {}", e);
            }
        }

        ad_events
    }

    pub fn update_adids(&self, version: &str, ad_ids: Vec<i64>) {
        match self.redis_dao.update_adids(version, ad_ids) {
            Ok(_) => {}
            Err(e) => {
                log::error!("update_adids error: {}", e);
            }
        }
    }

    pub fn get_adid_exp_cfg(&self, version: &str, ad_id: i64) -> AdIdExpCfg {
        let key = format!("{}:{}", version, ad_id);

        let cfg = self.adid_experiment_cache.get(&key).unwrap_or_else(|| {
            let r_cfg = match self.redis_dao.get_adid_exp_cfg(version, ad_id) {
                Ok(cfg) => cfg,
                Err(e) => {
                    log::error!("get_adid_exp_cfg error: {}", e);
                    AdIdExpCfg::default()
                }
            };
            if !r_cfg.is_empty() {
                self.adid_experiment_cache.insert(key, r_cfg.clone());
            }
            r_cfg
        });

        cfg
    }

    pub fn set_adid_exp_cfg(&self, version: &str, ad_id: i64, cfg: AdIdExpCfg) {
        let key = format!("{}:{}", version, ad_id);
        self.adid_experiment_cache.insert(key, cfg.clone());
        match self.redis_dao.set_adid_exp_cfg(version, ad_id, &cfg) {
            Ok(_) => {}
            Err(err) => log::error!("update_adids error: {}", err),
        }
    }

    pub(crate) fn get_user_daily_ad_id_event(
        &self,
        _adid: i64,
        _usr: &str,
        _date: &str,
    ) -> AdEvent {
        todo!()
    }

    pub(crate) fn query_temp_click(&self, _usr: &str) -> f64 {
        todo!()
    }

    /// =================================================
    /// 动态配置相关
    pub(crate) fn get_exp_base_cfg(&self) -> ExpBaseCfg {
        let base_cfg_map = self.dyn_cfg.get_hash(super::RedisCfgKey_ExpBaseCfg);

        ExpBaseCfg {
            version: read_parse(base_cfg_map.get("version")),
            base_value: read_parse(base_cfg_map.get("base")),
            score_factor: read_parse(base_cfg_map.get("score_factor")),
            start_time: get_date(base_cfg_map.get("start_time")),
        }
    }

    pub(crate) fn get_exp_ab_params(&self) -> AbParams {
        let base_cfg_map = self.dyn_cfg.get_hash(super::RedisCfgKey_ExpExpAbParams);

        AbParams {
            fill_a: read_parse(base_cfg_map.get("fill_a")),
            fill_b: read_parse(base_cfg_map.get("fill_b")),
            show_a: read_parse(base_cfg_map.get("show_a")),
            show_b: read_parse(base_cfg_map.get("show_b")),
            click_a: read_parse(base_cfg_map.get("click_a")),
            click_b: read_parse(base_cfg_map.get("click_b")),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_adid_whitelist(&self) -> HashSet<u64> {
        HashSet::new()
    }

    pub(crate) fn get_signal_ad_id_fill_rate(&self) -> Vec<RangeValue> {
        self.get_signal_cfg(super::RedisCfgKey_ExpSignalAdIdFillRate)
    }

    pub(crate) fn get_signal_ad_id_show_rate(&self) -> Vec<RangeValue> {
        self.get_signal_cfg(super::RedisCfgKey_ExpSignalAdIdShowRate)
    }

    pub(crate) fn get_signal_ad_id_click_rate(&self) -> Vec<RangeValue> {
        self.get_signal_cfg(super::RedisCfgKey_ExpSignalAdIdClickRate)
    }

    pub(crate) fn get_signal_daily_total_tempt_click(&self) -> Vec<RangeValue> {
        self.get_signal_cfg(super::RedisCfgKey_ExpSignalDailyTotalTemptClick)
    }

    pub(crate) fn get_realtime_ad_id_window_events(&self, _usergroup: &str, _adid: i64) -> AdEvent {
        todo!()
    }

    fn get_signal_cfg(&self, key: &str) -> Vec<RangeValue> {
        let cfgs = self.dyn_cfg.get_hash(key);

        let mut range_cfgs = Vec::new();
        for (k, v) in cfgs {
            if !k.contains("_") {
                continue;
            }

            let mut range_keys = k.split("_");
            let a = range_keys.next();
            let b = range_keys.next();
            range_cfgs.push(RangeValue {
                min: a.unwrap_or_default().parse::<f64>().unwrap_or_default(),
                max: b.unwrap_or_default().parse::<f64>().unwrap_or_default(),
                value: v.parse().unwrap_or_default(),
            });
        }

        range_cfgs.sort_by(|a, b| -> Ordering {
            if a.max > b.max {
                return Ordering::Less;
            } else if a.min > b.min {
                return Ordering::Greater;
            } else {
                return Ordering::Equal;
            }
        });

        range_cfgs
    }
}

fn read_parse<T>(s: Option<&String>) -> T
where
    T: FromStr + Default,
{
    match s {
        Some(s) => s.parse().unwrap_or_default(),
        None => T::default(),
    }
}

fn get_date(s: Option<&String>) -> DateTime<Local> {
    match s {
        Some(s) => s.parse::<DateTime<Local>>().unwrap_or(Local::now()),
        None => Local::now(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use moka::sync::ConcurrentCacheExt;

    use super::*;

    #[tokio::test]
    async fn test_adsdb_create() {
        let redis_client = redis::Client::open("redis://127.0.0.1").unwrap();
        let ads_db = AdsDB::new(redis_client.clone());
        assert_eq!(ads_db.adid_cache.entry_count(), 0);
        ads_db.add_adids_to_localcache("1", &vec![1, 2, 3]);
        ads_db.adid_cache.sync();
        assert_eq!(ads_db.adid_cache.entry_count(), 3);

        ads_db.add_adids_to_localcache("2", &vec![4, 3]);
        for kv in ads_db.adid_cache.iter() {
            println!("{:?}", kv)
        }

        let adids = ads_db.get_version_adids_from_localcache("1");
        println!("version1 ads:  {:?}", adids);
        ads_db.adid_cache.invalidate_all();
    }

    #[test]
    fn test_hashmap() {
        let mut map = HashMap::new();
        map.insert("a", "1");
        map.insert("b", "i2");
        map.insert("c", "3");

        println!("{:?}", map);
        println!("{:?}", map.get("a"));
        println!(
            "{:?}",
            map.get("a").map_or(0.0, |v| v.parse().unwrap_or(0.0))
        );
    }

    #[test]
    fn test_get_exp_base_cfg() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let redis_client = redis::Client::open("redis://127.0.0.1").unwrap();
            let ads_db = AdsDB::new(redis_client.clone());
            let cfg = ads_db.get_exp_base_cfg();
            println!("{:?}", cfg);
        });
    }

    #[test]
    fn test_get_signal_cfg() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let redis_client = redis::Client::open("redis://127.0.0.1").unwrap();
            let ads_db = AdsDB::new(redis_client.clone());
            let cfg = ads_db.get_signal_cfg("cfg:signal:adid:fillrate");

            for c in cfg {
                println!("{:?}", c);
            }
        });
    }
}
