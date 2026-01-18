use crate::{
    error::AppError, handlers::item::apply_insight_item, network::packet::ClientPacket,
    state::ConnectionContext, util::push::send_item_change_push,
};
use database::models::game::heros::UserHeroModel;
use prost::Message;
use sonettobuf::{CmdId, UseInsightItemReply, UseInsightItemRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_insight_item(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseInsightItemRequest::decode(&req.data[..])?;
    tracing::info!("Received UseInsightItemRequest: {:?}", request);

    let uid = request.uid.ok_or(AppError::InvalidRequest)?;
    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let hero = UserHeroModel::new(player_id, pool.clone());

    let item_id = apply_insight_item(&pool, player_id, uid, hero_id).await?;

    send_item_change_push(ctx.clone(), player_id, vec![], vec![], vec![item_id as u32]).await?;

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(
            CmdId::UseInsightItemCmd,
            UseInsightItemReply {
                hero_id: Some(hero_id),
                uid: Some(uid),
            },
            0,
            req.up_tag,
        )
        .await?;
    }

    if let Ok(hero) = hero.get_hero(hero_id).await {
        let mut conn = ctx.lock().await;
        conn.notify(
            CmdId::HeroHeroUpdatePushCmd,
            sonettobuf::HeroUpdatePush {
                hero_updates: vec![hero.into()],
            },
        )
        .await?;
    }

    Ok(())
}
