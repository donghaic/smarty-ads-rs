pub mod ads_dao;
pub mod dyn_cfg;
pub mod redis_dao;

pub use ads_dao::*;
pub use dyn_cfg::*;
pub use redis_dao::*;

const RedisCfgKey_ExpSignalDailyTotalTemptClick: &str = "cfg:signal:tempclick"; //
const RedisCfgKey_ExpSignalAdIdFillRate: &str = "cfg:signal:adid:fillrate"; //
const RedisCfgKey_ExpSignalAdIdShowRate: &str = "cfg:signal:adid:showrate"; //
const RedisCfgKey_ExpSignalAdIdClickRate: &str = "cfg:signal:adid:clickrate"; //
const RedisCfgKey_ExpTargetCtrAction: &str = "cfg:exp:action:targetctr:{}"; //

const RedisKey_ExpAdidDefalutChoice: &str = "exp:default:adid:choices"; //  默认选择的广告id
const RedisKey_ExpVersionAdids: &str = "expversion:adidlist:{}"; // 各版本的广告id列表
const RedisCfgKey_ExpVersionAdIdCfg: &str = "expversion:cfg:{}:{}"; // 各版本的广告id配置列表
const RedisCfgKey_ExpVersionAdIdScores: &str = "expversion:score:{}:{}"; // 各版本的广告id分数列表

const RedisCfgKey_MasterServer: &str = "cfg:master"; //
const RedisCfgKey_AdidWhitelist: &str = "cfg:whitelist"; //
const RedisCfgKey_MainActionRate: &str = "cfg:mainaction:rate"; //
const RedisCfgKey_ExpBaseCfg: &str = "cfg:exp:base";
const RedisCfgKey_ExpExpAbParams: &str = "cfg:exp:ab"; //
