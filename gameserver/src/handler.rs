use crate::cmd::*;
use crate::error::{AppError, CmdError};
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::CmdId;
use std::sync::Arc;
use tokio::sync::Mutex;

macro_rules! dispatch {
    ($cmd_id:expr, $ctx:expr, $packet:expr, {
        $($variant:path => $handler:expr),* $(,)?
    }) => {
        match $cmd_id {
            $(
                $variant => $handler($ctx, $packet).await?,
            )*
            v => return Err(AppError::Cmd(CmdError::UnhandledCmd(v))),
        }
    };
}

pub async fn dispatch_command(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: &[u8],
) -> Result<(), AppError> {
    let req = ClientPacket::decode(req)?;
    let cmd_id = TryInto::<CmdId>::try_into(req.cmd_id as i32)
        .map_err(|_| AppError::Cmd(CmdError::UnregisteredCmd(req.cmd_id)))?;

    tracing::info!("Received Cmd: {:?}", cmd_id);

    dispatch!(cmd_id, ctx, req, {
        CmdId::LoginRequestCmd => system::on_login,
        CmdId::ReconnectRequestCmd => system::on_reconnect,
        CmdId::UpdateClientStatBaseInfoCmd => stat::on_update_client_stat_base_info,

        CmdId::GetServerTimeCmd => common::on_get_server_time,
        CmdId::GetPlayerInfoCmd => player::on_get_player_info,
        CmdId::GetCurrencyListCmd => currency::on_get_currency_list,
        CmdId::GetGuideInfoCmd => guide::on_get_guide_info,
        CmdId::DiceHeroGetInfoCmd => dice::on_dice_hero_get_info,
        CmdId::ClientStatBaseInfoCmd => stat::on_client_stat_base_info,
        CmdId::GetSimplePropertyCmd => property::on_get_simple_property,
        CmdId::GetClothInfoCmd => player::on_get_cloth_info,
        CmdId::HeroInfoListCmd => hero::on_hero_info_list,
        CmdId::GetHeroGroupCommonListCmd => hero_group::on_get_hero_group_common_list,
        CmdId::GetHeroGroupListCmd => hero_group::on_get_hero_group_list,
        CmdId::GetHeroGroupSnapshotListCmd => hero_group::on_get_hero_group_snapshot_list,
        CmdId::GetItemListCmd => item::on_get_item_list,
        CmdId::GetDungeonCmd => dungeon::on_get_dungeon,
        CmdId::ReconnectFightCmd => fight::on_reconnect_fight,
        CmdId::GetBuyPowerInfoCmd => currency::on_get_buy_power_info,
        CmdId::GetEquipInfoCmd => equip::on_get_equip_info,
        CmdId::GetStoryCmd => story::on_get_story,
        CmdId::GetChargeInfoCmd => charge::on_get_charge_info,
        CmdId::GetMonthCardInfoCmd => charge::on_get_month_card_info,
        CmdId::GetBlockPackageInfoRequsetCmd => room::on_get_block_package_info,
        CmdId::GetBuildingInfoCmd => room::on_get_building_info,
        CmdId::GetCharacterInteractionInfoCmd => room::on_get_character_interaction_info,
        CmdId::GetSummonInfoCmd => summon::on_get_summon_info,
        CmdId::GetAchievementInfoCmd => achievement::on_get_achievement_info,
        CmdId::GetDialogInfoCmd => dialog::on_get_dialog_info,
        CmdId::GetAntiqueInfoCmd => antique::on_get_antique_info,
        CmdId::GetUnlockVoucherInfoCmd => voucher::on_get_unlock_voucher_info,
        CmdId::GetWeekwalkInfoCmd => weekwalk::on_get_weekwalk_info,
        CmdId::WeekwalkVer2GetInfoCmd => weekwalk::on_weekwalk_ver2_get_info,
        CmdId::GetExploreSimpleInfoCmd => explore::on_get_explore_simple_info,
        CmdId::GetTowerInfoCmd => tower::on_get_tower_info,
        CmdId::GetPlayerCardInfoCmd => player_card::on_get_player_card_info,
        CmdId::GetCommandPostInfoCmd => command_post::on_get_command_post_info,
        CmdId::GetRougeOutsideInfoCmd => rouge::on_get_rouge_outside_info, // need to implement / static data for now
        CmdId::LoadFriendInfosCmd => friend::on_load_friend_infos,
        CmdId::GetSignInInfoCmd => sign_in::on_get_sign_in_info,
        CmdId::GetStoreInfosCmd => store::on_get_store_infos, // keep this static for now it controlls the items in shop

       //bgm
        CmdId::GetBgmInfoCmd => bgm::on_get_bgm_info, // we're loading all the bgm from the excel table for starter data
        CmdId::SetUseBgmCmd => bgm::on_set_use_bgm,
        CmdId::SetFavoriteBgmCmd => bgm::on_set_favorite_bgm,

        CmdId::GetRoomObInfoCmd => room::on_get_room_ob_info,
        CmdId::CritterGetInfoCmd => critter::on_critter_get_info,
        CmdId::GetAllMailsCmd => mail::on_get_all_mails,
        CmdId::GetActivityInfosCmd => activity::on_get_activity_infos,
        CmdId::GetHandbookInfoCmd => handbook::on_get_handbook_info,
        CmdId::GetRedDotInfosCmd => red_dot::on_get_red_dot_infos,
        CmdId::DungeonInstructionDungeonInfoCmd => dungeon::on_instruction_dungeon_info,
        CmdId::GetBpInfoCmd => bp::on_get_bp_info,
        CmdId::GetTurnbackInfoCmd => turnback::on_get_turnback_info,
        CmdId::GetSettingInfosCmd => user_setting::on_get_setting_infos,
        CmdId::GetPowerMakerInfoCmd => power_maker::on_get_power_maker_info,
        CmdId::GetTaskInfoCmd => task::on_get_task_info,
        CmdId::GetNecrologistStoryCmd => necrologist_story::on_get_necrologist_story,
        CmdId::GetHeroStoryCmd => hero_story::on_get_hero_story,
        CmdId::GetRoomPlanInfoCmd => room::on_get_room_plan_info,

        // Controls the ui for the latest euphoria not implemented yet tho
        CmdId::GetAct125InfosCmd => activity125::on_get_act125_infos,

        CmdId::Act160GetInfoCmd => activity160::on_act160_get_info,
        CmdId::Act165GetInfoCmd => activity165::on_act165_get_info,

        // controls ui for bonus currency at the start usually for 7 days
        // state 0 = not started state 1 = not completed, state 2 = completed
        CmdId::Get101InfosCmd => activity101::on_get101_infos,

        CmdId::GetAct208InfoCmd => activity208::on_get_act208_info,
        CmdId::SetSimplePropertyCmd => property::on_set_simple_property,
        CmdId::MarkMainThumbnailCmd => player::on_mark_main_thumbnail,
        CmdId::AutoUseExpirePowerItemCmd => item::on_auto_use_expire_power_item,
        CmdId::GetAssistBonusCmd => player::on_get_assist_bonus,
        CmdId::SignInCmd => sign_in::on_sign_in,
        CmdId::GetAct209InfoCmd => activity209::on_get_act209_info,
        CmdId::Get101BonusCmd => activity101::on_get101_bonus,
        CmdId::GetChargePushInfoCmd => charge::on_get_charge_push_info,
        CmdId::HeroRedDotReadCmd => hero::on_hero_red_dot_read,
        CmdId::ReadChargeNewCmd => charge::on_read_charge_new,
        CmdId::DestinyStoneUseCmd => destiny_stone::on_destiny_stone_use,
        CmdId::HeroTouchCmd => hero::on_hero_touch,
        CmdId::UseSkinCmd => skin::on_use_skin,
        CmdId::HeroDefaultEquipCmd => hero::on_hero_default_equip,

        CmdId::MarkHeroFavorCmd => hero::on_mark_hero_favor,

        // special equipment for ezio
        CmdId::ChoiceHero3123WeaponCmd => hero::on_choice_hero_3123_weapon,

        //wilderness
        CmdId::GetManufactureInfoCmd => manufacture::on_get_manufacture_info,
        CmdId::GetRoomLogCmd => room::on_get_room_log,

        //battle
        CmdId::SetHeroGroupEquipCmd => hero_group::on_set_hero_group_equip,
        CmdId::SetHeroGroupSnapshotCmd => hero_group::on_set_hero_group_snapshot,
        CmdId::StartTowerBattleCmd => tower::on_start_tower_battle,
        CmdId::StartDungeonCmd => dungeon::on_start_dungeon,
        CmdId::BeginRoundCmd => dungeon::on_begin_round,
        CmdId::FightEndFightCmd => dungeon::on_fight_end_fight,
        CmdId::GetFightRecordGroupCmd => dungeon::on_get_fight_record_group,
        CmdId::GetFightOperCmd => dungeon::on_get_fight_oper,
        CmdId::ChangeHeroGroupSelectCmd => dungeon::on_change_hero_group_select,

        CmdId::SignInTotalRewardAllCmd => sign_in::on_sign_in_total_reward_all,

        CmdId::UpdateStoryCmd => story::on_update_story,

        CmdId::RenameCmd => system::on_rename,

        CmdId::SetShowHeroUniqueIdsCmd => hero::on_set_show_hero_unique_ids,

        CmdId::GetHeroBirthdayCmd => hero::on_get_hero_birthday,

        //Todo add option for talent upgrades
        CmdId::TalentStyleReadCmd => talent::on_talent_style_read, // just echos back the hero id

        //summons
        CmdId::SummonQueryTokenCmd => summon::on_summon_query_token,
        CmdId::SummonCmd => summon::on_summon,
        CmdId::ChooseEnhancedPoolHeroCmd => summon::on_choose_enhanced_pool_hero,

    });

    Ok(())
}
