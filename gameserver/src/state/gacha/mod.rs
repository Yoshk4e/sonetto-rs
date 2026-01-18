use anyhow::Result;
use data::exceldb;
use database::models::game::summon::SpPoolInfo;

mod helpers;
mod result;
mod rewards;
mod state;

pub use helpers::{
    get_rewards, parse_dupe_rewards, parse_id_list, parse_item, parse_store_product,
    parse_up_heroes,
};
pub use result::{GachaPool, GachaResult};
pub use rewards::grant_dupe_rewards;
pub use state::{BannerType, GachaState, load_gacha_state, save_gacha_state};

use crate::state::gacha::helpers::parse_weighted_id_list;

pub async fn build_gacha(pool_id: i32, sp_pool_info: Option<&SpPoolInfo>) -> Result<GachaPool> {
    let game_data = exceldb::get();

    let pool_cfg = game_data
        .summon_pool
        .iter()
        .find(|p| p.id == pool_id)
        .expect("Summon pool not found");

    let banner_type = match &sp_pool_info {
        Some(sp) => BannerType::from(sp.sp_type),
        None => BannerType::RateUp,
    };

    let (six_up, five_up, six_up_weighted) = match banner_type {
        BannerType::Yearning => {
            let (six_up, five_up) = parse_up_heroes(pool_cfg.up_weight.as_str());

            let weighted = parse_weighted_id_list(pool_cfg.double_ssr_up_rates.as_str());

            (six_up, five_up, weighted)
        }

        BannerType::RateUp => {
            let (six_up, five_up) = parse_up_heroes(pool_cfg.up_weight.as_str());

            (six_up, five_up, Vec::new())
        }

        BannerType::Ripple => {
            let sp = sp_pool_info.as_ref().unwrap();
            (sp.up_hero_ids.clone(), Vec::new(), Vec::new())
        }

        BannerType::Standard => (Vec::new(), Vec::new(), Vec::new()),
    };

    let summons = game_data.summon.iter().filter(|p| p.id == pool_id);

    let mut six_all = Vec::new();
    let mut five_all = Vec::new();
    let mut four = Vec::new();
    let mut three = Vec::new();
    let mut two = Vec::new();

    for summon in summons {
        let ids = parse_id_list(&summon.summon_id);

        match summon.rare {
            5 => six_all.extend(ids),
            4 => five_all.extend(ids),
            3 => four.extend(ids),
            2 => three.extend(ids),
            1 => two.extend(ids),
            _ => {}
        }
    }

    let six_normal = six_all
        .into_iter()
        .filter(|id| !six_up.contains(id))
        .collect();

    let five_normal = five_all
        .into_iter()
        .filter(|id| !five_up.contains(id))
        .collect();

    Ok(GachaPool {
        six_up,
        six_up_weighted,
        six_normal,
        five_up,
        five_normal,
        four,
        three,
        two,
    })
}
