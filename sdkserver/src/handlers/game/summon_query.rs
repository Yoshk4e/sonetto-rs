use crate::AppState;
use crate::handlers::account::helpers::get_summons;
use crate::models::{
    request::SummonQueryReq,
    response::{SummonQueryRsp, SummonQueryRspData},
};
use axum::extract::{Query, State};
use axum::response::Json;

pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<SummonQueryReq>,
) -> Json<SummonQueryRsp> {
    let user_id = query.user_id;
    let token = query.token;

    tracing::trace!("SummonQuery received");

    let rsp = match get_summons(&state, user_id, &token).await {
        Ok(rsp) => rsp,
        Err(err) => {
            tracing::warn!("SummonQuery failed: {err}");

            SummonQueryRsp::summons(SummonQueryRspData {
                page_data: Vec::new(),
            })
        }
    };

    Json(rsp)
}
