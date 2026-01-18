use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::items::{InsightItem, Item, PowerItem, UserItemModel};
use sonettobuf::{CmdId, GetItemListReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_item_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (items_data, power_items_data, insight_items_data) = {
        let conn = ctx.lock().await;
        let user_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();

        let item = UserItemModel::new(user_id, pool);

        let items: Vec<Item> = item.get_all_items().await?;
        let power_items: Vec<PowerItem> = item.get_all_power_items().await?;
        let insight_items: Vec<InsightItem> = item.get_all_insight_items().await?;

        (items, power_items, insight_items)
    };

    let reply = GetItemListReply {
        items: items_data.into_iter().map(Into::into).collect(),
        power_items: power_items_data.into_iter().map(Into::into).collect(),
        insight_items: insight_items_data.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetItemListCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
