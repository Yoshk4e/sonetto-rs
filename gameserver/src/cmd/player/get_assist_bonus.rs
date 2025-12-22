use crate::packet::ClientPacket;
use crate::{error::AppError, send_push};

use crate::state::ConnectionContext;
use sonettobuf::{
    CmdId, CurrencyChangePush, GetAssistBonusReply, ItemChangePush, MaterialChangePush,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_assist_bonus(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let data = GetAssistBonusReply {
        assist_bonus: Some(0),
        has_receive_assist_bonus: Some(0),
    };

    let should_push = {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard.check_and_mark_state_pushes().await?
    };

    if should_push {
        tracing::info!("Sending state pushes from GetAssistBonus");
        send_push!(
            ctx,
            CmdId::CurrencyChangePushCmd,
            CurrencyChangePush,
            "currency/currency_push_1.json"
        );
        send_push!(
            ctx,
            CmdId::ItemChangePushCmd,
            ItemChangePush,
            "item/item_push_1.json"
        );
        send_push!(
            ctx,
            CmdId::CurrencyChangePushCmd,
            CurrencyChangePush,
            "currency/currency_push_2.json"
        );
        send_push!(
            ctx,
            CmdId::MaterialChangePushCmd,
            MaterialChangePush,
            "material/material_push_1.json"
        );
    } else {
        tracing::warn!("No state pushes from GetAssistBonus");
    }

    {
        let mut ctx_guard = ctx.lock().await;

        ctx_guard
            .send_reply(CmdId::GetAssistBonusCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
