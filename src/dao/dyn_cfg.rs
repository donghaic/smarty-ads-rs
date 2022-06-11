use std::{
    collections::{BTreeMap, HashMap},
    str,
    sync::{Arc, RwLock},
};

use redis::Commands;
use tokio_cron_scheduler::{Job, JobScheduler};

/// 使用enum来实现多种配置管理
#[derive(Clone, Debug)]
pub enum CfgFieldField {
    Str(String),
    Int64(i64),
    Float64(f64),
    Hash(BTreeMap<String, String>),
}

#[derive(Clone)]
pub struct DyncConfigV2 {
    redis_client: redis::Client,
    fields: Arc<RwLock<HashMap<String, CfgFieldField>>>,
}

impl DyncConfigV2 {
    pub fn new(redis_client: redis::Client) -> Self {
        let dyn_cfg = Self {
            redis_client: redis_client,
            fields: Arc::new(RwLock::new(HashMap::new())),
        };

        dyn_cfg.add_str_field(super::RedisCfgKey_MasterServer.to_string());
        dyn_cfg.add_i64_field(super::RedisCfgKey_MainActionRate.to_string());
        dyn_cfg.add_hash_field(super::RedisCfgKey_ExpBaseCfg.to_string());
        dyn_cfg.add_hash_field(super::RedisCfgKey_ExpSignalDailyTotalTemptClick.to_string());
        dyn_cfg.add_hash_field(super::RedisCfgKey_ExpSignalAdIdFillRate.to_string());
        dyn_cfg.add_hash_field(super::RedisCfgKey_ExpSignalAdIdClickRate.to_string());
        dyn_cfg.add_hash_field(super::RedisCfgKey_ExpSignalAdIdShowRate.to_string());

        // 启动定时任务
        let monitor = Monitor::new(dyn_cfg.clone());
        monitor.start();

        dyn_cfg
    }

    pub fn add_i64_field(&self, key: String) {
        self.add_field(key, CfgFieldField::Int64(0));
    }

    pub fn add_hash_field(&self, key: String) {
        self.add_field(key, CfgFieldField::Hash(BTreeMap::new()));
    }

    pub fn add_str_field(&self, key: String) {
        let mut fields = self.fields.write().unwrap();
        fields.insert(key, CfgFieldField::Str("".to_string()));
    }

    pub fn add_field(&self, key: String, value: CfgFieldField) {
        let mut fields = self.fields.write().unwrap();
        fields.insert(key, value);
    }

    pub fn get_string(&self, key: &str) -> String {
        let fields = self.fields.read().unwrap();
        match fields.get(key) {
            Some(CfgFieldField::Str(val)) => val.clone(),
            _ => "".to_string(),
        }
    }

    pub fn get_i64(&self, key: &str) -> i64 {
        let fields = self.fields.read().unwrap();
        match fields.get(key) {
            Some(CfgFieldField::Int64(val)) => *val,
            _ => 0,
        }
    }

    pub fn get_f64(&self, key: &str) -> f64 {
        let fields = self.fields.read().unwrap();
        match fields.get(key) {
            Some(CfgFieldField::Float64(val)) => *val,
            _ => 0.0,
        }
    }

    pub fn get_hash(&self, key: &str) -> BTreeMap<String, String> {
        let fields = self.fields.read().unwrap();
        match fields.get(key) {
            Some(CfgFieldField::Hash(val)) => val.clone(),
            _ => BTreeMap::default(),
        }
    }

    fn sync_redis(&mut self) {
        let mut fields = self.fields.write().unwrap();

        log::info!("DyncConfigV2 Monitor sync redis keys={}", fields.len());
        for (key, val) in fields.iter_mut() {
            match val {
                CfgFieldField::Str(val) => {
                    let v = self.redis_client.get(key).unwrap_or_default();
                    if v != *val {
                        log::info!(
                            "sync redis Str key={} preval={:?}, newval={:?}",
                            key,
                            val,
                            v
                        );
                    }
                    *val = v;
                }
                CfgFieldField::Int64(val) => {
                    let v: i64 = self.redis_client.get(key).unwrap_or_default();
                    if v != *val {
                        log::info!(
                            "sync redis Int64 key={} preval={:?}, newval={:?}",
                            key,
                            val,
                            v
                        );
                    }
                    *val = v;
                }
                CfgFieldField::Hash(val) => {
                    let v: BTreeMap<String, String> =
                        self.redis_client.hgetall(key).unwrap_or_default();
                    let old = format!("{:?}", *val);
                    let new = format!("{:?}", v);

                    if old != new {
                        log::info!(
                            "sync redis Hash key={} preval={:?}, newval={:?}",
                            key,
                            val,
                            v
                        );
                    }
                    *val = v;
                }
                CfgFieldField::Float64(val) => {
                    let v: f64 = self.redis_client.get(key).unwrap_or_default();
                    if v != *val {
                        log::info!(
                            "sync redis Float64 key={} preval={:?}, newval={:?}",
                            key,
                            val,
                            v
                        );
                    }
                    *val = v;
                }
            }
        }
    }
}

struct Monitor {
    scheduler: JobScheduler,
    dync_cfg: DyncConfigV2,
}

impl Monitor {
    pub fn new(dync_cfg: DyncConfigV2) -> Self {
        let sched = JobScheduler::new().unwrap();

        Self {
            scheduler: sched,
            dync_cfg: dync_cfg,
        }
    }

    pub fn start(&self) {
        log::info!("starting dyn config monitor");
        let mut cfg = self.dync_cfg.clone();

        let _ = self
            .scheduler
            .add(Job::new("*/10 * * * * *", move |_, _| cfg.sync_redis()).unwrap());
        self.scheduler.start().unwrap();

        let mut cfg = self.dync_cfg.clone();
        cfg.sync_redis();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_hash() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let redis_client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
            let cfg = DyncConfigV2::new(redis_client);
            let hash = cfg.get_hash("hash_key1");
            println!("{:?}", hash);

            let data = BTreeMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]);

            cfg.add_field("hash_key1".to_string(), CfgFieldField::Hash(data));

            let hash = cfg.get_hash("hash_key1");
            println!("{:?}", hash);
        });
    }

    #[test]
    fn get_string() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let redis_client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
            let cfg = DyncConfigV2::new(redis_client);

            let val1 = cfg.get_string("key1");
            println!("val1={}", val1);

            cfg.add_field("key2".to_string(), CfgFieldField::Str("value2".to_string()));
            let val2 = cfg.get_string("key2");
            println!("val2={}", val2);
        });
    }

    #[test]
    fn get_i32() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let redis_client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
            let cfg = DyncConfigV2::new(redis_client);
            let val1 = cfg.get_i64("key1");
            println!("val1={}", val1);

            cfg.add_field("key2".to_string(), CfgFieldField::Int64(2));
            let val1 = cfg.get_i64("key2");
            println!("val2={}", val1);
        });
    }
}
