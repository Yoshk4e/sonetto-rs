use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::UserHeroModel;
use sonettobuf::{CmdId, HeroBirthdayInfo, HeroInfoListReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_info_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (heroes_data, touch_count, all_skins, birthday_infos) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        let hero = UserHeroModel::new(player_id, pool.clone());

        let heroes = hero.get_all_heroes().await?;

        let touch_count = hero.get_touch_count().await?;

        let all_skins = hero.get_skins().await?;

        let birthday_infos = hero.get_birthdays().await?;

        (heroes, touch_count, all_skins, birthday_infos)
    };

    let reply = HeroInfoListReply {
        heros: heroes_data.into_iter().map(Into::into).collect(),
        touch_count_left: touch_count,
        all_hero_skin: all_skins,
        birthday_infos: birthday_infos
            .into_iter()
            .map(|(hero_id, count)| HeroBirthdayInfo {
                hero_id: Some(hero_id),
                birthday_count: Some(count),
            })
            .collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::HeroInfoListCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
