use std::collections::BTreeMap;

use crate::model::*;
use axum::http::StatusCode;
use axum::{Extension, Json};

use crate::dao::*;
use crate::service::*;

pub async fn predict(
    Extension(prediction_service): Extension<ProdictionService>,
    Json(req): Json<Request>,
) -> Result<Json<Response>, StatusCode> {
    let (status, msg) = req.check();
    if status {
        let response = prediction_service.predict(&req);
        Ok(Json(response))
    } else {
        Ok(Json(Response {
            code: 400,
            msg: msg,
            items: vec![],
        }))
    }
}

pub async fn test(
    Extension(ads_db): Extension<AdsDB>,
) -> Result<Json<BTreeMap<String, String>>, StatusCode> {
    let data = ads_db.dyn_cfg.get_hash("cfg:exp:base");
    Ok(Json(data))
}
