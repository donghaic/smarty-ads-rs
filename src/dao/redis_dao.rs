use std::collections::HashMap;

use crate::model::*;
use anyhow::{Ok, Result};
use redis::{self, Commands};

const CFG_EXPIRE_TIME: usize = 5 * 3600 * 24;

#[derive(Clone)]
pub struct RedisDao {
    pub redis_client: redis::Client,
}

impl RedisDao {
    pub(crate) fn new(redis_client: redis::Client) -> RedisDao {
        RedisDao {
            redis_client: redis_client,
        }
    }

    pub(crate) fn set_adid_exp_cfg(
        &self,
        version: &str,
        ad_id: i64,
        cfg: &crate::model::AdIdExpCfg,
    ) -> Result<()> {
        let key = format!("expversion:cfg:{}:{}", version, ad_id);
        let value = serde_json::to_string(cfg)?;
        let mut conn = self.redis_client.get_connection()?;
        let _: () = conn.set_ex(key, value, CFG_EXPIRE_TIME)?;
        Ok(())
    }

    pub(crate) fn get_adid_exp_cfg(&self, version: &str, ad_id: i64) -> Result<AdIdExpCfg> {
        let key = format!("expversion:cfg:{}:{}", version, ad_id);
        let mut conn = self.redis_client.get_connection()?;
        let cfg_json: String = conn.get(key)?;
        let cfg: AdIdExpCfg = serde_json::from_str(cfg_json.as_str())?;

        Ok(cfg)
    }

    pub(crate) fn get_multi_event_by_keys(&self, _keys: Vec<&str>) -> Result<Vec<String>> {
        todo!()
    }

    pub(crate) fn get_ad_exp_action_score(
        &self,
        version: &str,
        ad_id: i64,
    ) -> Result<HashMap<String, i64>> {
        let mut conn = self.redis_client.get_connection()?;
        let key = format!("expversion:score:{}:{}", version, ad_id);
        let action_scores: HashMap<String, i64> = conn.hgetall(key)?;
        Ok(action_scores)
    }

    pub(crate) fn set_ad_exp_action_score(
        &self,
        version: &str,
        ad_id: i64,
        scores: HashMap<String, i64>,
    ) -> Result<()> {
        let mut conn = self.redis_client.get_connection()?;
        let key = format!("expversion:score:{}:{}", version, ad_id);
        for kv in scores {
            let _: () = conn.hset(&key, kv.0, kv.1)?;
        }
        Ok(())
    }

    pub(crate) fn get_exp_adid_default_choice(
        &self,
        _version: &str,
        _ad_id: i64,
    ) -> Result<HashMap<String, i64>> {
        let mut conn = self.redis_client.get_connection()?;
        let default_choice: HashMap<String, i64> =
            conn.hgetall(super::RedisKey_ExpAdidDefalutChoice)?;
        Ok(default_choice)
    }

    pub(crate) fn update_exp_base_cfg(&self, cfg: &ExpBaseCfg) -> Result<()> {
        let mut conn = self.redis_client.get_connection()?;
        let now = chrono::Local::now();
        let start_time = now.format("%Y-%m-%d %H:%M:%S%z").to_string();
        let values = [("version", &cfg.version), ("start_time", &start_time)];
        conn.hset_multiple(super::RedisCfgKey_ExpBaseCfg, &values)?;
        Ok(())
    }

    pub(crate) fn update_adids(&self, _version: &str, _ad_ids: Vec<i64>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local};

    use super::*;

    #[test]
    fn test_datetime_format() {
        let now = chrono::Local::now();
        let start_time = now.format("%Y-%m-%d %H:%M:%S%z").to_string();
        println!("{}", start_time);

        let date = "2022-05-30 10:27:47+0800"
            .parse::<DateTime<Local>>()
            .unwrap();

        println!("{}", date);
    }
    #[test]
    fn test_update_exp_base_cfg() {
        let redis_client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let redis_dao = RedisDao::new(redis_client);
        redis_dao.update_exp_base_cfg(&ExpBaseCfg {
            version: "1.0.0".to_string(),
            base_value: 0.0,
            score_factor: 0.0,
            start_time: chrono::Local::now(),
        });
    }
}
