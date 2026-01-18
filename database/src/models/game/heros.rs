use anyhow::Result;
use data::exceldb::character_level::CharacterLevel;
use serde::{Deserialize, Serialize};
use sonettobuf;
use sqlx::{FromRow, SqlitePool};

#[allow(async_fn_in_trait)]
pub trait HeroModel<T> {
    async fn get(&self, hero_id: i32) -> Result<T>;
    async fn get_uid(&self, hero_uid: i32) -> Result<T>;
    async fn get_all(&self) -> Result<Vec<T>>;
    async fn has_hero(&self, hero_id: i32) -> Result<bool>;
    async fn hero_duplicate(&self, hero_id: i32) -> Result<i32>;
    async fn create_hero(&self, hero_id: i32) -> Result<i64>;
    async fn hero_count(&self, rarity: usize, now: i64) -> Result<()>;
    async fn special_equipped_gear(&self, hero_id: i32, extra_str: String) -> Result<()>;
    async fn equipped_gear(&self, hero_id: i32, equip_uid: i64) -> Result<()>;
    async fn touch_count(&self) -> Result<Option<i32>>;
    async fn use_touch(&self) -> Result<Option<i32>>;
    async fn skin(&self, hero_id: i32, skin_id: i32) -> Result<()>;
    async fn skins(&self) -> Result<Vec<i32>>;
    async fn birthdays(&self) -> Result<Vec<(i32, i32)>>;
    async fn destiny_stone(&self, hero_id: i32, stone_id: i32) -> Result<()>;
    async fn level_up(&self, hero_id: i32, new_level: i32, stats: &CharacterLevel) -> Result<()>;
    async fn rank_up(&self, hero_id: i32, new_rank: i32) -> Result<()>;
    async fn unlock_insight_skin(&self, hero_id: i32, target_rank: i32) -> Result<bool>;
    async fn read_hero_red_dot(&self, hero_id: i32, red_dot: i32) -> Result<()>;
    async fn upgrade_ex_skill(&self, hero_id: i32, levels: i32) -> Result<()>;
    async fn set_favor(&self, hero_id: i32, is_favor: bool) -> Result<()>;
    async fn unmark_new(&self, hero_id: i32) -> Result<()>;
    async fn set_show_hero(&self, hero_uids: &[i64]) -> Result<()>;
    async fn talent_style_read(&self, hero_id: i32) -> Result<()>;
    async fn update_talent(&self, hero_id: i32, talent_id: i32) -> Result<()>;
    async fn remove_talent_cube(
        &self,
        hero_id: i32,
        template_id: i32,
        pos_x: i32,
        pos_y: i32,
    ) -> Result<()>;
    async fn place_talent_cube(
        &self,
        hero_id: i32,
        template_id: i32,
        cube_id: i32,
        direction: i32,
        pos_x: i32,
        pos_y: i32,
    ) -> Result<()>;
    async fn sync_active_talent_cubes(
        &self,
        hero_id: i32,
        template_id: i32,
        get_cube: Option<(i32, i32)>,
        put_cube: Option<(i32, i32, i32, i32)>,
    ) -> Result<()>;
    async fn get_template_info(
        &self,
        hero_id: i32,
        template_id: i32,
    ) -> Result<sonettobuf::TalentTemplateInfo>;
    async fn load_talent_scheme(
        &self,
        hero_id: i32,
        talent_id: i32,
        talent_mould: i32,
        template_id: i32,
    ) -> Result<sonettobuf::TalentTemplateInfo>;
    async fn has_talent_style(&self, hero_id: i32, style: i32) -> Result<bool>;
    async fn unlock_talent_style(
        &self,
        hero_id: i32,
        style: i32,
    ) -> Result<(Vec<(u32, i32)>, Vec<(i32, i32)>)>;
    async fn apply_talent_style(&self, hero_id: i32, template_id: i32, style: i32) -> Result<()>;
    async fn switch_talent_template(
        &self,
        hero_id: i32,
        template_id: i32,
    ) -> Result<sonettobuf::TalentTemplateInfo>;
}

pub struct UserHeroModel {
    user_id: i64,
    pool: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Hero {
    pub uid: i64,
    pub user_id: i64,
    pub hero_id: i32,
    pub create_time: i64,
    pub level: i32,
    pub exp: i32,
    pub rank: i32,
    pub breakthrough: i32,
    pub skin: i32,
    pub faith: i32,
    pub active_skill_level: i32,
    pub ex_skill_level: i32,
    pub is_new: bool,
    pub talent: i32,
    pub default_equip_uid: i64,
    pub duplicate_count: i32,
    pub use_talent_template_id: i32,
    pub talent_style_unlock: i32,
    pub talent_style_red: i32,
    pub is_favor: bool,
    pub destiny_rank: i32,
    pub destiny_level: i32,
    pub destiny_stone: i32,
    pub red_dot: i32,
    pub extra_str: String,
    // Base attributes
    pub base_hp: i32,
    pub base_attack: i32,
    pub base_defense: i32,
    pub base_mdefense: i32,
    pub base_technic: i32,
    pub base_multi_hp_idx: i32,
    pub base_multi_hp_num: i32,
    // Ex attributes
    pub ex_cri: i32,
    pub ex_recri: i32,
    pub ex_cri_dmg: i32,
    pub ex_cri_def: i32,
    pub ex_add_dmg: i32,
    pub ex_drop_dmg: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct HeroSpAttribute {
    pub hero_uid: i64,
    pub revive: i32,
    pub heal: i32,
    pub absorb: i32,
    pub defense_ignore: i32,
    pub clutch: i32,
    pub final_add_dmg: i32,
    pub final_drop_dmg: i32,
    pub normal_skill_rate: i32,
    pub play_add_rate: i32,
    pub play_drop_rate: i32,
    pub dizzy_resistances: i32,
    pub sleep_resistances: i32,
    pub petrified_resistances: i32,
    pub frozen_resistances: i32,
    pub disarm_resistances: i32,
    pub forbid_resistances: i32,
    pub seal_resistances: i32,
    pub cant_get_exskill_resistances: i32,
    pub del_ex_point_resistances: i32,
    pub stress_up_resistances: i32,
    pub control_resilience: i32,
    pub del_ex_point_resilience: i32,
    pub stress_up_resilience: i32,
    pub charm_resistances: i32,
    pub rebound_dmg: i32,
    pub extra_dmg: i32,
    pub reuse_dmg: i32,
    pub big_skill_rate: i32,
    pub clutch_dmg: i32,
}

impl From<HeroSpAttribute> for sonettobuf::HeroSpAttribute {
    fn from(sp: HeroSpAttribute) -> Self {
        sonettobuf::HeroSpAttribute {
            revive: Some(sp.revive),
            heal: Some(sp.heal),
            absorb: Some(sp.absorb),
            defense_ignore: Some(sp.defense_ignore),
            clutch: Some(sp.clutch),
            final_add_dmg: Some(sp.final_add_dmg),
            final_drop_dmg: Some(sp.final_drop_dmg),
            normal_skill_rate: Some(sp.normal_skill_rate),
            play_add_rate: Some(sp.play_add_rate),
            play_drop_rate: Some(sp.play_drop_rate),
            dizzy_resistances: Some(sp.dizzy_resistances),
            sleep_resistances: Some(sp.sleep_resistances),
            petrified_resistances: Some(sp.petrified_resistances),
            frozen_resistances: Some(sp.frozen_resistances),
            disarm_resistances: Some(sp.disarm_resistances),
            forbid_resistances: Some(sp.forbid_resistances),
            seal_resistances: Some(sp.seal_resistances),
            cant_get_exskill_resistances: Some(sp.cant_get_exskill_resistances),
            del_ex_point_resistances: Some(sp.del_ex_point_resistances),
            stress_up_resistances: Some(sp.stress_up_resistances),
            control_resilience: Some(sp.control_resilience),
            del_ex_point_resilience: Some(sp.del_ex_point_resilience),
            stress_up_resilience: Some(sp.stress_up_resilience),
            charm_resistances: Some(sp.charm_resistances),
            rebound_dmg: Some(sp.rebound_dmg),
            extra_dmg: Some(sp.extra_dmg),
            reuse_dmg: Some(sp.reuse_dmg),
            big_skill_rate: Some(sp.big_skill_rate),
            clutch_dmg: Some(sp.clutch_dmg),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct HeroEquipAttribute {
    pub id: i64,
    pub hero_uid: i64,
    pub equip_id: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub mdefense: i32,
    pub technic: i32,
    pub multi_hp_idx: i32,
    pub multi_hp_num: i32,
}

impl From<HeroEquipAttribute> for sonettobuf::HeroEquipAttribute {
    fn from(e: HeroEquipAttribute) -> Self {
        sonettobuf::HeroEquipAttribute {
            id: Some(e.equip_id),
            equip_attr: Some(sonettobuf::HeroAttribute {
                hp: Some(e.hp),
                attack: Some(e.attack),
                defense: Some(e.defense),
                mdefense: Some(e.mdefense),
                technic: Some(e.technic),
                multi_hp_idx: Some(e.multi_hp_idx),
                multi_hp_num: Some(e.multi_hp_num),
            }),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct HeroSkin {
    pub hero_uid: i64,
    pub skin: i32,
    pub expire_sec: i32,
}

impl From<HeroSkin> for sonettobuf::SkinInfo {
    fn from(s: HeroSkin) -> Self {
        sonettobuf::SkinInfo {
            skin: Some(s.skin),
            expire_sec: Some(s.expire_sec),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct HeroTalentCube {
    pub hero_uid: i64,
    pub cube_id: i32,
    pub direction: i32,
    pub pos_x: i32,
    pub pos_y: i32,
}

impl From<HeroTalentCube> for sonettobuf::TalentCubeInfo {
    fn from(c: HeroTalentCube) -> Self {
        sonettobuf::TalentCubeInfo {
            cube_id: Some(c.cube_id),
            direction: Some(c.direction),
            pos_x: Some(c.pos_x),
            pos_y: Some(c.pos_y),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct HeroTalentTemplate {
    pub id: i64,
    pub hero_uid: i64,
    pub template_id: i32,
    pub name: String,
    pub style: i32,
}

#[derive(Debug, Clone)]
pub struct HeroData {
    pub record: Hero,
    pub passive_skill_levels: Vec<i32>,
    pub voices: Vec<i32>,
    pub voices_heard: Vec<i32>,
    pub skin_list: Vec<HeroSkin>,
    pub sp_attr: Option<HeroSpAttribute>,
    pub equip_attrs: Vec<HeroEquipAttribute>,
    pub item_unlocks: Vec<i32>,
    pub talent_cubes: Vec<HeroTalentCube>,
    pub talent_templates: Vec<(HeroTalentTemplate, Vec<HeroTalentCube>)>,
    pub destiny_stone_unlocks: Vec<i32>,
}

impl From<HeroData> for sonettobuf::HeroInfo {
    fn from(h: HeroData) -> Self {
        sonettobuf::HeroInfo {
            uid: h.record.uid,
            user_id: h.record.user_id,
            hero_id: h.record.hero_id,
            create_time: Some(h.record.create_time),
            level: Some(h.record.level),
            exp: Some(h.record.exp),
            rank: Some(h.record.rank),
            breakthrough: Some(h.record.breakthrough),
            skin: Some(h.record.skin),
            faith: Some(h.record.faith),
            active_skill_level: Some(h.record.active_skill_level),
            passive_skill_level: h.passive_skill_levels,
            ex_skill_level: Some(h.record.ex_skill_level),
            voice: h.voices,
            voice_heard: h.voices_heard,
            skin_info_list: h.skin_list.into_iter().map(Into::into).collect(),
            base_attr: Some(sonettobuf::HeroAttribute {
                hp: Some(h.record.base_hp),
                attack: Some(h.record.base_attack),
                defense: Some(h.record.base_defense),
                mdefense: Some(h.record.base_mdefense),
                technic: Some(h.record.base_technic),
                multi_hp_idx: Some(h.record.base_multi_hp_idx),
                multi_hp_num: Some(h.record.base_multi_hp_num),
            }),
            ex_attr: Some(sonettobuf::HeroExAttribute {
                cri: Some(h.record.ex_cri),
                recri: Some(h.record.ex_recri),
                cri_dmg: Some(h.record.ex_cri_dmg),
                cri_def: Some(h.record.ex_cri_def),
                add_dmg: Some(h.record.ex_add_dmg),
                drop_dmg: Some(h.record.ex_drop_dmg),
            }),
            sp_attr: h.sp_attr.map(Into::into),
            equip_attr_list: h.equip_attrs.into_iter().map(Into::into).collect(),
            is_new: Some(h.record.is_new),
            item_unlock: h.item_unlocks,
            talent: Some(h.record.talent),
            talent_cube_infos: h.talent_cubes.into_iter().map(Into::into).collect(),
            default_equip_uid: Some(h.record.default_equip_uid),
            duplicate_count: Some(h.record.duplicate_count),
            talent_templates: h
                .talent_templates
                .into_iter()
                .map(|(template, cubes)| sonettobuf::TalentTemplateInfo {
                    id: Some(template.template_id),
                    talent_cube_infos: cubes.into_iter().map(Into::into).collect(),
                    name: Some(template.name),
                    style: Some(template.style),
                })
                .collect(),
            use_talent_template_id: Some(h.record.use_talent_template_id),
            talent_style_unlock: Some(h.record.talent_style_unlock),
            talent_style_red: Some(h.record.talent_style_red),
            is_favor: Some(h.record.is_favor),
            destiny_rank: Some(h.record.destiny_rank),
            destiny_level: Some(h.record.destiny_level),
            destiny_stone: Some(h.record.destiny_stone),
            destiny_stone_unlock: h.destiny_stone_unlocks,
            red_dot: Some(h.record.red_dot),
            extra_str: Some(h.record.extra_str),
        }
    }
}

impl UserHeroModel {
    pub fn new(user_id: i64, pool: SqlitePool) -> Self {
        Self { user_id, pool }
    }

    pub async fn get_hero(&self, hero_id: i32) -> Result<HeroData> {
        HeroModel::<HeroData>::get(self, hero_id).await
    }

    pub async fn get_all_heroes(&self) -> Result<Vec<HeroData>> {
        HeroModel::<HeroData>::get_all(self).await
    }

    pub async fn has_hero(&self, hero_id: i32) -> Result<bool> {
        HeroModel::<HeroData>::has_hero(self, hero_id).await
    }

    pub async fn player_hero_count(&self, rarity: usize, now: i64) -> Result<()> {
        HeroModel::<HeroData>::hero_count(self, rarity, now).await
    }

    pub async fn add_hero_duplicate(&self, hero_id: i32) -> Result<i32> {
        HeroModel::<HeroData>::hero_duplicate(self, hero_id).await
    }

    pub async fn create_hero(&self, hero_id: i32) -> Result<i64> {
        HeroModel::<HeroData>::create_hero(self, hero_id).await
    }

    pub async fn update_special_equipped_gear(
        &self,
        hero_id: i32,
        extra_str: String,
    ) -> Result<()> {
        HeroModel::<HeroData>::special_equipped_gear(self, hero_id, extra_str).await
    }

    pub async fn update_equipped_gear(&self, hero_id: i32, equip_uid: i64) -> Result<()> {
        HeroModel::<HeroData>::equipped_gear(self, hero_id, equip_uid).await
    }

    pub async fn get_touch_count(&self) -> Result<Option<i32>> {
        HeroModel::<HeroData>::touch_count(self).await
    }

    pub async fn update_skin(&self, hero_uid: i32, skin_id: i32) -> Result<()> {
        HeroModel::<HeroData>::skin(self, hero_uid, skin_id).await
    }

    pub async fn get_skins(&self) -> Result<Vec<i32>> {
        HeroModel::<HeroData>::skins(self).await
    }

    pub async fn get_birthdays(&self) -> Result<Vec<(i32, i32)>> {
        HeroModel::<HeroData>::birthdays(self).await
    }

    pub async fn update_destiny_stone(&self, hero_id: i32, stone_id: i32) -> Result<()> {
        HeroModel::<HeroData>::destiny_stone(self, hero_id, stone_id).await
    }
}

impl HeroModel<HeroData> for UserHeroModel {
    async fn get(&self, hero_id: i32) -> Result<HeroData> {
        let hero_record =
            sqlx::query_as::<_, Hero>("SELECT * FROM heroes WHERE user_id = ? AND hero_id = ?")
                .bind(self.user_id)
                .bind(hero_id)
                .fetch_one(&self.pool)
                .await?;

        let hero_uid = hero_record.uid;

        let passive_skill_levels: Vec<i32> = sqlx::query_scalar(
            "SELECT level FROM hero_passive_skill_levels WHERE hero_uid = ? ORDER BY skill_index",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let voices: Vec<i32> =
            sqlx::query_scalar("SELECT voice_id FROM hero_voices WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_all(&self.pool)
                .await?;

        let voices_heard: Vec<i32> =
            sqlx::query_scalar("SELECT voice_id FROM hero_voices_heard WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_all(&self.pool)
                .await?;

        let skin_list = sqlx::query_as::<_, HeroSkin>(
            "SELECT hero_uid, skin, expire_sec FROM hero_skins WHERE hero_uid = ?",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let sp_attr =
            sqlx::query_as::<_, HeroSpAttribute>("SELECT * FROM hero_sp_attrs WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_optional(&self.pool)
                .await?;

        let equip_attrs = sqlx::query_as::<_, HeroEquipAttribute>(
            "SELECT * FROM hero_equip_attributes WHERE hero_uid = ?",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let item_unlocks: Vec<i32> =
            sqlx::query_scalar("SELECT item_id FROM hero_item_unlocks WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_all(&self.pool)
                .await?;

        let talent_cubes = sqlx::query_as::<_, HeroTalentCube>(
            "SELECT hero_uid, cube_id, direction, pos_x, pos_y FROM hero_talent_cubes WHERE hero_uid = ?"
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let templates = sqlx::query_as::<_, HeroTalentTemplate>(
            "SELECT id, hero_uid, template_id, name, style FROM hero_talent_templates WHERE hero_uid = ?"
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let mut talent_templates = Vec::new();
        for template in templates {
            let template_cubes = sqlx::query_as::<_, HeroTalentCube>(
                "SELECT 0 as hero_uid, cube_id, direction, pos_x, pos_y
                 FROM hero_talent_template_cubes WHERE template_row_id = ?",
            )
            .bind(template.id)
            .fetch_all(&self.pool)
            .await?;

            talent_templates.push((template, template_cubes));
        }

        let destiny_stone_unlocks: Vec<i32> = sqlx::query_scalar(
            "SELECT stone_id FROM hero_destiny_stone_unlocks WHERE hero_uid = ?",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        Ok(HeroData {
            record: hero_record,
            passive_skill_levels,
            voices,
            voices_heard,
            skin_list,
            sp_attr,
            equip_attrs,
            item_unlocks,
            talent_cubes,
            talent_templates,
            destiny_stone_unlocks,
        })
    }

    async fn get_uid(&self, hero_uid: i32) -> Result<HeroData> {
        let hero_record =
            sqlx::query_as::<_, Hero>("SELECT * FROM heroes WHERE user_id = ? AND uid = ?")
                .bind(self.user_id)
                .bind(hero_uid)
                .fetch_one(&self.pool)
                .await?;

        let hero_uid = hero_record.uid;

        let passive_skill_levels: Vec<i32> = sqlx::query_scalar(
            "SELECT level FROM hero_passive_skill_levels WHERE hero_uid = ? ORDER BY skill_index",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let voices: Vec<i32> =
            sqlx::query_scalar("SELECT voice_id FROM hero_voices WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_all(&self.pool)
                .await?;

        let voices_heard: Vec<i32> =
            sqlx::query_scalar("SELECT voice_id FROM hero_voices_heard WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_all(&self.pool)
                .await?;

        let skin_list = sqlx::query_as::<_, HeroSkin>(
            "SELECT hero_uid, skin, expire_sec FROM hero_skins WHERE hero_uid = ?",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let sp_attr =
            sqlx::query_as::<_, HeroSpAttribute>("SELECT * FROM hero_sp_attrs WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_optional(&self.pool)
                .await?;

        let equip_attrs = sqlx::query_as::<_, HeroEquipAttribute>(
            "SELECT * FROM hero_equip_attributes WHERE hero_uid = ?",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let item_unlocks: Vec<i32> =
            sqlx::query_scalar("SELECT item_id FROM hero_item_unlocks WHERE hero_uid = ?")
                .bind(hero_uid)
                .fetch_all(&self.pool)
                .await?;

        let talent_cubes = sqlx::query_as::<_, HeroTalentCube>(
            "SELECT hero_uid, cube_id, direction, pos_x, pos_y FROM hero_talent_cubes WHERE hero_uid = ?"
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let templates = sqlx::query_as::<_, HeroTalentTemplate>(
            "SELECT id, hero_uid, template_id, name, style FROM hero_talent_templates WHERE hero_uid = ?"
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        let mut talent_templates = Vec::new();
        for template in templates {
            let template_cubes = sqlx::query_as::<_, HeroTalentCube>(
                "SELECT 0 as hero_uid, cube_id, direction, pos_x, pos_y
                 FROM hero_talent_template_cubes WHERE template_row_id = ?",
            )
            .bind(template.id)
            .fetch_all(&self.pool)
            .await?;

            talent_templates.push((template, template_cubes));
        }

        let destiny_stone_unlocks: Vec<i32> = sqlx::query_scalar(
            "SELECT stone_id FROM hero_destiny_stone_unlocks WHERE hero_uid = ?",
        )
        .bind(hero_uid)
        .fetch_all(&self.pool)
        .await?;

        Ok(HeroData {
            record: hero_record,
            passive_skill_levels,
            voices,
            voices_heard,
            skin_list,
            sp_attr,
            equip_attrs,
            item_unlocks,
            talent_cubes,
            talent_templates,
            destiny_stone_unlocks,
        })
    }

    async fn get_all(&self) -> Result<Vec<HeroData>> {
        let heroes =
            sqlx::query_as::<_, Hero>("SELECT * FROM heroes WHERE user_id = ?1 ORDER BY uid")
                .bind(self.user_id)
                .fetch_all(&self.pool)
                .await?;

        let mut result = Vec::new();

        for hero_record in heroes {
            let hero_uid = hero_record.uid;

            let passive_skill_levels: Vec<i32> = sqlx::query_scalar(
                "SELECT level FROM hero_passive_skill_levels WHERE hero_uid = ?1 ORDER BY skill_index",
            )
            .bind(hero_uid)
            .fetch_all(&self.pool)
            .await?;

            let voices: Vec<i32> =
                sqlx::query_scalar("SELECT voice_id FROM hero_voices WHERE hero_uid = ?1")
                    .bind(hero_uid)
                    .fetch_all(&self.pool)
                    .await?;

            let voices_heard: Vec<i32> =
                sqlx::query_scalar("SELECT voice_id FROM hero_voices_heard WHERE hero_uid = ?1")
                    .bind(hero_uid)
                    .fetch_all(&self.pool)
                    .await?;

            let skin_list = sqlx::query_as::<_, HeroSkin>(
                "SELECT hero_uid, skin, expire_sec FROM hero_skins WHERE hero_uid = ?1",
            )
            .bind(hero_uid)
            .fetch_all(&self.pool)
            .await?;

            let sp_attr = sqlx::query_as::<_, HeroSpAttribute>(
                "SELECT * FROM hero_sp_attrs WHERE hero_uid = ?1",
            )
            .bind(hero_uid)
            .fetch_optional(&self.pool)
            .await?;

            let equip_attrs = sqlx::query_as::<_, HeroEquipAttribute>(
                "SELECT * FROM hero_equip_attributes WHERE hero_uid = ?1",
            )
            .bind(hero_uid)
            .fetch_all(&self.pool)
            .await?;

            let item_unlocks: Vec<i32> =
                sqlx::query_scalar("SELECT item_id FROM hero_item_unlocks WHERE hero_uid = ?1")
                    .bind(hero_uid)
                    .fetch_all(&self.pool)
                    .await?;

            let talent_cubes = sqlx::query_as::<_, HeroTalentCube>(
                "SELECT hero_uid, cube_id, direction, pos_x, pos_y FROM hero_talent_cubes WHERE hero_uid = ?1"
            )
            .bind(hero_uid)
            .fetch_all(&self.pool)
            .await?;

            let templates = sqlx::query_as::<_, HeroTalentTemplate>(
                "SELECT id, hero_uid, template_id, name, style FROM hero_talent_templates WHERE hero_uid = ?1"
            )
            .bind(hero_uid)
            .fetch_all(&self.pool)
            .await?;

            let mut talent_templates = Vec::new();
            for template in templates {
                let template_cubes = sqlx::query_as::<_, HeroTalentCube>(
                    "SELECT 0 as hero_uid, cube_id, direction, pos_x, pos_y
                     FROM hero_talent_template_cubes WHERE template_row_id = ?1",
                )
                .bind(template.id)
                .fetch_all(&self.pool)
                .await?;

                talent_templates.push((template, template_cubes));
            }

            let destiny_stone_unlocks: Vec<i32> = sqlx::query_scalar(
                "SELECT stone_id FROM hero_destiny_stone_unlocks WHERE hero_uid = ?1",
            )
            .bind(hero_uid)
            .fetch_all(&self.pool)
            .await?;

            result.push(HeroData {
                record: hero_record,
                passive_skill_levels,
                voices,
                voices_heard,
                skin_list,
                sp_attr,
                equip_attrs,
                item_unlocks,
                talent_cubes,
                talent_templates,
                destiny_stone_unlocks,
            });
        }

        Ok(result)
    }

    async fn has_hero(&self, hero_id: i32) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM heroes WHERE user_id = ? AND hero_id = ?",
        )
        .bind(self.user_id)
        .bind(hero_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists > 0)
    }

    async fn hero_duplicate(&self, hero_id: i32) -> Result<i32> {
        sqlx::query(
            r#"
            UPDATE heroes
            SET duplicate_count = duplicate_count + 1
            WHERE user_id = ? AND hero_id = ?
            "#,
        )
        .bind(self.user_id)
        .bind(hero_id)
        .execute(&self.pool)
        .await?;

        let new_count = sqlx::query_scalar::<_, i32>(
            "SELECT duplicate_count FROM heroes WHERE user_id = ? AND hero_id = ?",
        )
        .bind(self.user_id)
        .bind(hero_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(new_count)
    }

    async fn create_hero(&self, hero_id: i32) -> Result<i64> {
        let game_data = data::exceldb::get();
        let now = common::time::ServerTime::now_ms();

        let last_hero_uid: Option<i64> =
            sqlx::query_scalar("SELECT uid FROM heroes ORDER BY uid DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;

        let hero_uid = match last_hero_uid {
            Some(uid) => uid + 1,
            None => 20000001,
        };

        let character = game_data
            .character
            .iter()
            .find(|c| c.id == hero_id && c.id != 3029 && c.id != 9998) //npc
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let hero_skin = character.skin_id;
        let rare = character.rare as usize;

        let level1_stats = game_data
            .character_level
            .iter()
            .filter(|s| s.hero_id == hero_id)
            .min_by_key(|s| s.level);

        let (level, hp, atk, def, mdef, technic, cri, recri, cri_dmg, cri_def, add_dmg, drop_dmg) =
            if let Some(stats) = level1_stats {
                (
                    stats.level,
                    stats.hp,
                    stats.atk,
                    stats.def,
                    stats.mdef,
                    stats.technic,
                    stats.cri,
                    stats.recri,
                    stats.cri_dmg,
                    stats.cri_def,
                    stats.add_dmg,
                    stats.drop_dmg,
                )
            } else {
                (1, 1000, 100, 100, 100, 100, 0, 0, 1300, 0, 0, 0)
            };

        let min_ranks = game_data
            .character_rank
            .iter()
            .filter(|s| s.hero_id == hero_id)
            .min_by_key(|s| s.rank);

        let min_rank = if let Some(min) = min_ranks {
            min.rank
        } else {
            1
        };

        let default_skin = game_data
            .skin
            .iter()
            .filter(|s| s.character_id != 0)
            .filter(|s| s.character_id == hero_id)
            .min_by_key(|s| s.id)
            .map(|s| s.id)
            .unwrap_or(hero_skin);

        let destiny_data = game_data
            .character_destiny
            .iter()
            .find(|d| d.hero_id == hero_id);

        let (destiny_rank, destiny_level, destiny_stone, red_dot_type) =
            if let Some(d) = destiny_data {
                // Hero has destiny - start at 1
                let rank = min_rank;
                let level = 1;
                let stone = d
                    .facets_id
                    .split('#')
                    .next()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                let red_dot_type = 6;
                (rank, level, stone, red_dot_type)
            } else {
                // Hero doesn't have destiny system
                (0, 0, 0, 0)
            };

        let equip_id = character
            .equip_rec
            .split('#')
            .next()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1501);

        let strengthen_stats = game_data
            .equip_strengthen
            .iter()
            .find(|s| s.strength_type == equip_id);

        let (
            final_hp,
            final_atk,
            final_def,
            final_mdef,
            final_technic,
            final_cri,
            final_recri,
            final_cri_dmg,
            final_cri_def,
            final_add_dmg,
            final_drop_dmg,
        ) = if let Some(_) = strengthen_stats {
            // For new heroes, don't add equipment bonuses
            (
                hp,   // No + s.hp
                atk,  // No + s.atk
                def,  // No + s.def
                mdef, // No + s.mdef
                technic, cri, recri, cri_dmg, cri_def, add_dmg, drop_dmg,
            )
        } else {
            (
                hp, atk, def, mdef, technic, cri, recri, cri_dmg, cri_def, add_dmg, drop_dmg,
            )
        };

        let extra_str = if hero_id == 3123 {
            "1003#2003"
        } else if hero_id == 3124 {
            "2#21,22|3#32,33,31"
        } else {
            ""
        };

        let starting_talent = game_data
            .character_talent
            .iter()
            .filter(|t| t.hero_id == hero_id && t.talent_id == 1)
            .map(|t| t.talent_id)
            .next()
            .unwrap_or(1);

        sqlx::query(
            r#"
            INSERT INTO heroes (
                uid, user_id, hero_id, create_time,
                level, exp, rank, breakthrough, skin, faith,
                active_skill_level, ex_skill_level, is_new, talent,
                default_equip_uid, duplicate_count, use_talent_template_id,
                talent_style_unlock, talent_style_red, is_favor,
                destiny_rank, destiny_level, destiny_stone, red_dot, extra_str,
                base_hp, base_attack, base_defense, base_mdefense, base_technic,
                base_multi_hp_idx, base_multi_hp_num,
                ex_cri, ex_recri, ex_cri_dmg, ex_cri_def, ex_add_dmg, ex_drop_dmg
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
                ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
                ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38
            )
            "#,
        )
        .bind(hero_uid)
        .bind(self.user_id)
        .bind(hero_id)
        .bind(now)
        .bind(level) // Level 1
        .bind(0) // Starting exp
        .bind(min_rank) // Starting rank
        .bind(0) // No breakthrough
        .bind(default_skin)
        .bind(10400) // Starting faith
        .bind(1) // Active skill level 1
        .bind(0) // Ex skill level 0
        .bind(true) // is_new (true for new heroes)
        .bind(starting_talent) // talent
        .bind(0) // default_equip_uid = 0
        .bind(0) // duplicate_count
        .bind(1) // use_talent_template_id
        .bind(1) // talent_style_unlock
        .bind(0) // talent_style_red
        .bind(false) // is_favor
        .bind(destiny_rank) // destiny_rank (0)
        .bind(destiny_level) // destiny_level (0)
        .bind(destiny_stone) // destiny_stone
        .bind(red_dot_type) // red_dot
        .bind(extra_str) // extra_str
        // Base attributes (level 1 stats without equipment bonuses)
        .bind(final_hp)
        .bind(final_atk)
        .bind(final_def)
        .bind(final_mdef)
        .bind(final_technic)
        .bind(0) // base_multi_hp_idx
        .bind(0) // base_multi_hp_num
        // Ex attributes
        .bind(final_cri)
        .bind(final_recri)
        .bind(final_cri_dmg)
        .bind(final_cri_def)
        .bind(final_add_dmg)
        .bind(final_drop_dmg)
        .execute(&self.pool)
        .await?;

        let max_skill_group = game_data
            .skill_passive_level
            .iter()
            .filter(|s| s.hero_id == hero_id)
            .map(|s| s.skill_group)
            .max()
            .unwrap_or(0);

        for skill_group in 1..=max_skill_group {
            let min_level = game_data
                .skill_passive_level
                .iter()
                .filter(|s| s.hero_id == hero_id && s.skill_group == skill_group)
                .map(|s| s.skill_level)
                .min()
                .unwrap_or(1);

            sqlx::query(
                "INSERT INTO hero_passive_skill_levels (hero_uid, skill_index, level) VALUES (?, ?, ?)",
            )
            .bind(hero_uid)
            .bind(skill_group - 1)
            .bind(min_level)
            .execute(&self.pool)
            .await?;
        }

        let character_voices: Vec<&data::exceldb::character_voice::CharacterVoice> = game_data
            .character_voice
            .iter()
            .filter(|v| v.hero_id == hero_id)
            .filter(|t| t.r#type == 9 || t.r#type == 11)
            .collect();

        for voice in &character_voices {
            sqlx::query("INSERT INTO hero_voices (hero_uid, voice_id) VALUES (?, ?)")
                .bind(hero_uid)
                .bind(voice.audio)
                .execute(&self.pool)
                .await?;
        }

        for item_id in [6, 3, 7, 4] {
            sqlx::query("INSERT INTO hero_item_unlocks (hero_uid, item_id) VALUES (?, ?)")
                .bind(hero_uid)
                .bind(item_id)
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO hero_sp_attrs (
                hero_uid, revive, heal, absorb, defense_ignore, clutch,
                final_add_dmg, final_drop_dmg, normal_skill_rate, play_add_rate, play_drop_rate,
                dizzy_resistances, sleep_resistances, petrified_resistances, frozen_resistances,
                disarm_resistances, forbid_resistances, seal_resistances, cant_get_exskill_resistances,
                del_ex_point_resistances, stress_up_resistances, control_resilience,
                del_ex_point_resilience, stress_up_resilience, charm_resistances,
                rebound_dmg, extra_dmg, reuse_dmg, big_skill_rate, clutch_dmg
            ) VALUES (
                ?1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            )
            "#,
        )
        .bind(hero_uid)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO hero_birthday_info (user_id, hero_id, birthday_count) VALUES (?, ?, ?)",
        )
        .bind(self.user_id)
        .bind(hero_id)
        .bind(0) // Starting at 0 birthday celebrations
        .execute(&self.pool)
        .await?;

        if let Some(destiny_data) = destiny_data {
            for stone_str in destiny_data.facets_id.split('#') {
                if let Ok(stone_id) = stone_str.parse::<i32>() {
                    sqlx::query(
                        "INSERT INTO hero_destiny_stone_unlocks (hero_uid, stone_id) VALUES (?, ?)",
                    )
                    .bind(hero_uid)
                    .bind(stone_id)
                    .execute(&self.pool)
                    .await?;
                }
            }
        }

        let talent_config = game_data
            .character_talent
            .iter()
            .find(|t| t.hero_id == hero_id && t.talent_id == 1);

        if let Some(talent) = talent_config {
            let talent_scheme = game_data
                .talent_scheme
                .iter()
                .find(|s| s.talent_id == talent.talent_id && s.talent_mould == talent.talent_mould);

            if let Some(scheme) = talent_scheme {
                let cubes: Vec<(i32, i32, i32, i32)> = scheme
                    .talen_scheme
                    .split('#')
                    .filter_map(|cube_str| {
                        let parts: Vec<&str> = cube_str.split(',').collect();
                        if parts.len() == 4 {
                            let cube_id = parts[0].parse::<i32>().ok()?;
                            let direction = parts[1].parse::<i32>().ok()?;
                            let pos_x = parts[2].parse::<i32>().ok()?;
                            let pos_y = parts[3].parse::<i32>().ok()?;
                            Some((cube_id, direction, pos_x, pos_y))
                        } else {
                            None
                        }
                    })
                    .collect();

                for (cube_id, direction, pos_x, pos_y) in &cubes {
                    sqlx::query(
                        "INSERT INTO hero_talent_cubes (hero_uid, cube_id, direction, pos_x, pos_y) VALUES (?, ?, ?, ?, ?)"
                    )
                    .bind(hero_uid)
                    .bind(cube_id)
                    .bind(direction)
                    .bind(pos_x)
                    .bind(pos_y)
                    .execute(&self.pool)
                    .await?;
                }

                tracing::info!(
                    "Inserted {} talent cubes for hero {} talent 1",
                    cubes.len(),
                    hero_id
                );
            }
        }

        // Insert talent templates
        for template_id in 1..=4 {
            let result = sqlx::query(
                "INSERT INTO hero_talent_templates (hero_uid, template_id, name, style) VALUES (?, ?, ?, ?)"
            )
            .bind(hero_uid)
            .bind(template_id)
            .bind("")
            .bind(0)
            .execute(&self.pool)
            .await?;

            let template_row_id = result.last_insert_rowid();

            // Template #1 gets the same cubes as active (saved preset)
            if template_id == 1 && talent_config.is_some() {
                if let Some(talent) = talent_config {
                    let talent_scheme = game_data.talent_scheme.iter().find(|s| {
                        s.talent_id == talent.talent_id && s.talent_mould == talent.talent_mould
                    });

                    if let Some(scheme) = talent_scheme {
                        let cubes: Vec<(i32, i32, i32, i32)> = scheme
                            .talen_scheme
                            .split('#')
                            .filter_map(|cube_str| {
                                let parts: Vec<&str> = cube_str.split(',').collect();
                                if parts.len() == 4 {
                                    Some((
                                        parts[0].parse().ok()?,
                                        parts[1].parse().ok()?,
                                        parts[2].parse().ok()?,
                                        parts[3].parse().ok()?,
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for (cube_id, direction, pos_x, pos_y) in &cubes {
                            sqlx::query(
                                "INSERT INTO hero_talent_template_cubes (template_row_id, cube_id, direction, pos_x, pos_y) VALUES (?, ?, ?, ?, ?)"
                            )
                            .bind(template_row_id)
                            .bind(cube_id)
                            .bind(direction)
                            .bind(pos_x)
                            .bind(pos_y)
                            .execute(&self.pool)
                            .await?;
                        }
                    }
                }
            }
        }

        self.player_hero_count(rare, now).await?;

        tracing::info!(
            "Created hero {} (uid {}) for user {}",
            hero_id,
            hero_uid,
            self.user_id
        );

        Ok(hero_uid)
    }

    async fn hero_count(&self, rarity: usize, now: i64) -> Result<()> {
        let rarity_column = match rarity {
            1 => "hero_rare_nn_count",
            2 => "hero_rare_n_count",
            3 => "hero_rare_r_count",
            4 => "hero_rare_sr_count",
            5 => "hero_rare_ssr_count",
            _ => return Ok(()),
        };

        sqlx::query(&format!(
            r#"
            UPDATE player_info
            SET {} = {} + 1,
                updated_at = ?
            WHERE player_id = ?
            "#,
            rarity_column, rarity_column
        ))
        .bind(now)
        .bind(self.user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn special_equipped_gear(&self, hero_id: i32, extra_str: String) -> Result<()> {
        let hero_data = self.get(hero_id).await?;
        sqlx::query("UPDATE heroes SET extra_str = ? WHERE uid = ?")
            .bind(&extra_str)
            .bind(hero_data.record.uid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn equipped_gear(&self, hero_id: i32, equip_uid: i64) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET default_equip_uid = ? WHERE uid = ? AND user_id = ?")
            .bind(equip_uid)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn use_touch(&self) -> Result<Option<i32>> {
        let current = self.get_touch_count().await?;
        let current = current.unwrap_or(5);

        if current <= 0 {
            return Ok(None);
        }

        let new_count = current - 1;
        sqlx::query("UPDATE hero_touch_count SET touch_count_left = ? WHERE user_id = ?")
            .bind(new_count)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(Some(new_count))
    }

    async fn touch_count(&self) -> Result<Option<i32>> {
        let count: Option<i32> =
            sqlx::query_scalar("SELECT touch_count_left FROM hero_touch_count WHERE user_id = ?1")
                .bind(self.user_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(count)
    }

    async fn skin(&self, hero_id: i32, skin_id: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET skin = ? WHERE uid = ?")
            .bind(skin_id)
            .bind(hero_data.record.uid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn skins(&self) -> Result<Vec<i32>> {
        let skins: Vec<i32> =
            sqlx::query_scalar("SELECT skin_id FROM hero_all_skins WHERE user_id = ?1")
                .bind(self.user_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(skins)
    }

    async fn birthdays(&self) -> Result<Vec<(i32, i32)>> {
        let info: Vec<(i32, i32)> = sqlx::query_as(
            "SELECT hero_id, birthday_count FROM hero_birthday_info WHERE user_id = ?1",
        )
        .bind(self.user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(info)
    }

    async fn destiny_stone(&self, hero_id: i32, stone_id: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;
        sqlx::query("UPDATE heroes SET destiny_stone = ? WHERE uid = ?")
            .bind(stone_id)
            .bind(hero_data.record.uid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn level_up(&self, hero_id: i32, new_level: i32, stats: &CharacterLevel) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query(
            r#"UPDATE heroes
                   SET level = ?,
                       base_hp = ?,
                       base_attack = ?,
                       base_defense = ?,
                       base_mdefense = ?,
                       base_technic = ?,
                       ex_cri = ?,
                       ex_recri = ?,
                       ex_cri_dmg = ?,
                       ex_cri_def = ?,
                       ex_add_dmg = ?,
                       ex_drop_dmg = ?
                   WHERE uid = ?"#,
        )
        .bind(new_level)
        .bind(stats.hp)
        .bind(stats.atk)
        .bind(stats.def)
        .bind(stats.mdef)
        .bind(stats.technic)
        .bind(stats.cri)
        .bind(stats.recri)
        .bind(stats.cri_dmg)
        .bind(stats.cri_def)
        .bind(stats.add_dmg)
        .bind(stats.drop_dmg)
        .bind(hero_data.record.uid)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn rank_up(&self, hero_id: i32, new_rank: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET rank = ?, level = 1 WHERE uid = ? AND user_id = ?")
            .bind(new_rank)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn unlock_insight_skin(&self, hero_id: i32, target_rank: i32) -> Result<bool> {
        if target_rank < 3 {
            return Ok(false);
        }

        let game_data = data::exceldb::get();
        let insight_skin = game_data
            .skin
            .iter()
            .find(|s| s.character_id == hero_id && s.id % 100 == 2 && s.gain_approach == 1);

        let Some(skin) = insight_skin else {
            return Ok(false);
        };

        let has_skin: Option<i32> =
            sqlx::query_scalar("SELECT 1 FROM hero_all_skins WHERE user_id = ? AND skin_id = ?")
                .bind(self.user_id)
                .bind(skin.id)
                .fetch_optional(&self.pool)
                .await?;

        if has_skin.is_some() {
            return Ok(false);
        }

        let hero_data = self.get(hero_id).await?;

        sqlx::query("INSERT INTO hero_all_skins (user_id, skin_id) VALUES (?, ?)")
            .bind(self.user_id)
            .bind(skin.id)
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO hero_skins (hero_uid, skin, expire_sec) VALUES (?, ?, ?)")
            .bind(hero_data.record.uid)
            .bind(skin.id)
            .bind(0)
            .execute(&self.pool)
            .await?;

        sqlx::query("UPDATE heroes SET skin = ? WHERE uid = ? AND user_id = ?")
            .bind(skin.id)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!(
            "User {} unlocked and equipped Insight II skin {} for hero {}",
            self.user_id,
            skin.id,
            hero_id
        );

        Ok(true)
    }

    async fn read_hero_red_dot(&self, hero_id: i32, red_dot: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;
        sqlx::query("UPDATE heroes SET red_dot = ? WHERE uid = ? AND user_id = ?")
            .bind(red_dot)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn upgrade_ex_skill(&self, hero_id: i32, levels: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;
        let new_level = (hero_data.record.ex_skill_level + levels).min(5);

        sqlx::query("UPDATE heroes SET ex_skill_level = ? WHERE uid = ? AND user_id = ?")
            .bind(new_level)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn set_favor(&self, hero_id: i32, is_favor: bool) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET is_favor = ? WHERE uid = ? AND user_id = ?")
            .bind(is_favor)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn unmark_new(&self, hero_id: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET is_new = 0 WHERE uid = ? AND user_id = ?")
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn set_show_hero(&self, hero_uids: &[i64]) -> Result<()> {
        for (slot_idx, uid) in hero_uids.iter().enumerate() {
            let display_order = slot_idx as i32;

            if *uid == 0 {
                sqlx::query(
                    r#"
                    DELETE FROM player_show_heroes
                    WHERE player_id = ? AND display_order = ?
                    "#,
                )
                .bind(self.user_id)
                .bind(display_order)
                .execute(&self.pool)
                .await?;

                continue;
            }

            #[derive(FromRow)]
            struct HeroRow {
                hero_id: i32,
                level: i32,
                rank: i32,
                ex_skill_level: i32,
                skin: i32,
            }

            let hero = sqlx::query_as::<_, HeroRow>(
                "
                SELECT
                    hero_id,
                    level,
                    rank,
                    ex_skill_level,
                    skin
                FROM heroes
                WHERE uid = ? AND user_id = ?
                ",
            )
            .bind(uid)
            .bind(self.user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid hero uid {} for user {}", uid, self.user_id))?;

            sqlx::query(
                r#"
                INSERT INTO player_show_heroes (
                    player_id,
                    hero_id,
                    level,
                    rank,
                    ex_skill_level,
                    skin,
                    display_order
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(player_id, display_order)
                DO UPDATE SET
                    hero_id = excluded.hero_id,
                    level = excluded.level,
                    rank = excluded.rank,
                    ex_skill_level = excluded.ex_skill_level,
                    skin = excluded.skin
                "#,
            )
            .bind(self.user_id)
            .bind(hero.hero_id)
            .bind(hero.level)
            .bind(hero.rank)
            .bind(hero.ex_skill_level)
            .bind(hero.skin)
            .bind(display_order)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn talent_style_read(&self, hero_id: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET talent_style_red = 0 WHERE uid = ? AND user_id = ?")
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update_talent(&self, hero_id: i32, talent_id: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        sqlx::query("UPDATE heroes SET talent = ? WHERE uid = ? AND user_id = ?")
            .bind(talent_id)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn remove_talent_cube(
        &self,
        hero_id: i32,
        template_id: i32,
        pos_x: i32,
        pos_y: i32,
    ) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            "DELETE FROM hero_talent_template_cubes
                 WHERE template_row_id = ? AND pos_x = ? AND pos_y = ?",
        )
        .bind(template_row_id)
        .bind(pos_x)
        .bind(pos_y)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn place_talent_cube(
        &self,
        hero_id: i32,
        template_id: i32,
        cube_id: i32,
        direction: i32,
        pos_x: i32,
        pos_y: i32,
    ) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            "DELETE FROM hero_talent_template_cubes
                 WHERE template_row_id = ? AND pos_x = ? AND pos_y = ?",
        )
        .bind(template_row_id)
        .bind(pos_x)
        .bind(pos_y)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO hero_talent_template_cubes
                 (template_row_id, cube_id, direction, pos_x, pos_y)
                 VALUES (?, ?, ?, ?, ?)",
        )
        .bind(template_row_id)
        .bind(cube_id)
        .bind(direction)
        .bind(pos_x)
        .bind(pos_y)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn sync_active_talent_cubes(
        &self,
        hero_id: i32,
        template_id: i32,
        get_cube: Option<(i32, i32)>,
        put_cube: Option<(i32, i32, i32, i32)>,
    ) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        if template_id != hero_data.record.use_talent_template_id {
            return Ok(());
        }

        if let Some((pos_x, pos_y)) = get_cube {
            sqlx::query(
                "DELETE FROM hero_talent_cubes
                     WHERE hero_uid = ? AND pos_x = ? AND pos_y = ?",
            )
            .bind(hero_data.record.uid)
            .bind(pos_x)
            .bind(pos_y)
            .execute(&self.pool)
            .await?;
        }

        if let Some((cube_id, direction, pos_x, pos_y)) = put_cube {
            sqlx::query(
                "DELETE FROM hero_talent_cubes
                     WHERE hero_uid = ? AND pos_x = ? AND pos_y = ?",
            )
            .bind(hero_data.record.uid)
            .bind(pos_x)
            .bind(pos_y)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                "INSERT INTO hero_talent_cubes
                     (hero_uid, cube_id, direction, pos_x, pos_y)
                     VALUES (?, ?, ?, ?, ?)",
            )
            .bind(hero_data.record.uid)
            .bind(cube_id)
            .bind(direction)
            .bind(pos_x)
            .bind(pos_y)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_template_info(
        &self,
        hero_id: i32,
        template_id: i32,
    ) -> Result<sonettobuf::TalentTemplateInfo> {
        let hero_data = self.get(hero_id).await?;

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let template_data: (String, i32) =
            sqlx::query_as("SELECT name, style FROM hero_talent_templates WHERE id = ?")
                .bind(template_row_id)
                .fetch_one(&self.pool)
                .await?;

        let cubes: Vec<(i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT cube_id, direction, pos_x, pos_y
                 FROM hero_talent_template_cubes
                 WHERE template_row_id = ?",
        )
        .bind(template_row_id)
        .fetch_all(&self.pool)
        .await?;

        let talent_cube_infos: Vec<sonettobuf::TalentCubeInfo> = cubes
            .into_iter()
            .map(
                |(cube_id, direction, pos_x, pos_y)| sonettobuf::TalentCubeInfo {
                    cube_id: Some(cube_id),
                    direction: Some(direction),
                    pos_x: Some(pos_x),
                    pos_y: Some(pos_y),
                },
            )
            .collect();

        Ok(sonettobuf::TalentTemplateInfo {
            id: Some(template_id),
            talent_cube_infos,
            name: Some(template_data.0),
            style: Some(template_data.1),
        })
    }

    async fn load_talent_scheme(
        &self,
        hero_id: i32,
        talent_id: i32,
        talent_mould: i32,
        template_id: i32,
    ) -> Result<sonettobuf::TalentTemplateInfo> {
        let hero_data = self.get(hero_id).await?;
        let game_data = data::exceldb::get();

        let talent_scheme = game_data
            .talent_scheme
            .iter()
            .find(|s| s.talent_id == talent_id && s.talent_mould == talent_mould)
            .ok_or_else(|| {
                tracing::error!(
                    "Talent scheme not found for talent {} mould {}",
                    talent_id,
                    talent_mould
                );
                anyhow::anyhow!("Talent scheme not found")
            })?;

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let cubes: Vec<(i32, i32, i32, i32)> = talent_scheme
            .talen_scheme
            .split('#')
            .filter_map(|cube_str| {
                let parts: Vec<&str> = cube_str.split(',').collect();
                if parts.len() == 4 {
                    let cube_id = parts[0].parse::<i32>().ok()?;
                    let direction = parts[1].parse::<i32>().ok()?;
                    let pos_x = parts[2].parse::<i32>().ok()?;
                    let pos_y = parts[3].parse::<i32>().ok()?;
                    Some((cube_id, direction, pos_x, pos_y))
                } else {
                    None
                }
            })
            .collect();

        sqlx::query("DELETE FROM hero_talent_template_cubes WHERE template_row_id = ?")
            .bind(template_row_id)
            .execute(&self.pool)
            .await?;

        for (cube_id, direction, pos_x, pos_y) in &cubes {
            sqlx::query(
                "INSERT INTO hero_talent_template_cubes
                 (template_row_id, cube_id, direction, pos_x, pos_y)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(template_row_id)
            .bind(cube_id)
            .bind(direction)
            .bind(pos_x)
            .bind(pos_y)
            .execute(&self.pool)
            .await?;
        }

        tracing::info!(
            "Loaded {} cubes from talent scheme {} into template {}",
            cubes.len(),
            talent_id,
            template_id
        );

        if template_id == hero_data.record.use_talent_template_id {
            sqlx::query("DELETE FROM hero_talent_cubes WHERE hero_uid = ?")
                .bind(hero_data.record.uid)
                .execute(&self.pool)
                .await?;

            for (cube_id, direction, pos_x, pos_y) in &cubes {
                sqlx::query(
                    "INSERT INTO hero_talent_cubes
                     (hero_uid, cube_id, direction, pos_x, pos_y)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(hero_data.record.uid)
                .bind(cube_id)
                .bind(direction)
                .bind(pos_x)
                .bind(pos_y)
                .execute(&self.pool)
                .await?;
            }

            tracing::info!(
                "Updated active talent cubes for hero {} (template {})",
                hero_id,
                template_id
            );
        }

        let template_data: (String, i32) =
            sqlx::query_as("SELECT name, style FROM hero_talent_templates WHERE id = ?")
                .bind(template_row_id)
                .fetch_one(&self.pool)
                .await?;

        let talent_cube_infos: Vec<sonettobuf::TalentCubeInfo> = cubes
            .into_iter()
            .map(
                |(cube_id, direction, pos_x, pos_y)| sonettobuf::TalentCubeInfo {
                    cube_id: Some(cube_id),
                    direction: Some(direction),
                    pos_x: Some(pos_x),
                    pos_y: Some(pos_y),
                },
            )
            .collect();

        Ok(sonettobuf::TalentTemplateInfo {
            id: Some(template_id),
            talent_cube_infos,
            name: Some(template_data.0),
            style: Some(template_data.1),
        })
    }

    async fn has_talent_style(&self, hero_id: i32, style: i32) -> Result<bool> {
        let hero_data = self.get(hero_id).await?;

        let has_style: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM hero_talent_styles WHERE hero_uid = ? AND style_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(style)
        .fetch_optional(&self.pool)
        .await?;

        Ok(has_style.is_some())
    }

    async fn unlock_talent_style(
        &self,
        hero_id: i32,
        style: i32,
    ) -> Result<(Vec<(u32, i32)>, Vec<(i32, i32)>)> {
        let hero_data = self.get(hero_id).await?;
        let game_data = data::exceldb::get();

        let style_cost = game_data
            .talent_style_cost
            .iter()
            .find(|s| s.hero_id == hero_id && s.style_id == style)
            .ok_or_else(|| {
                tracing::error!("Style cost not found for hero {} style {}", hero_id, style);
                anyhow::anyhow!("Style cost not found")
            })?;

        let mut cost_items = Vec::new();
        let mut cost_currencies = Vec::new();

        for cost_part in style_cost.consume.split('|') {
            let parts: Vec<&str> = cost_part.split('#').collect();
            if parts.len() >= 3 {
                match parts[0] {
                    "1" => {
                        let item_id: u32 = parts[1].parse()?;
                        let amount: i32 = parts[2].parse()?;
                        cost_items.push((item_id, amount));
                    }
                    "2" => {
                        let currency_id: i32 = parts[1].parse()?;
                        let amount: i32 = parts[2].parse()?;
                        cost_currencies.push((currency_id, amount));
                    }
                    _ => {}
                }
            }
        }

        sqlx::query("INSERT INTO hero_talent_styles (hero_uid, style_id) VALUES (?, ?)")
            .bind(hero_data.record.uid)
            .bind(style)
            .execute(&self.pool)
            .await?;

        let style_bit = 1 << style;
        let new_unlock = hero_data.record.talent_style_unlock | style_bit;

        sqlx::query("UPDATE heroes SET talent_style_unlock = ? WHERE uid = ? AND user_id = ?")
            .bind(new_unlock)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!(
            "User {} unlocked talent style {} for hero {}",
            self.user_id,
            style,
            hero_id
        );

        Ok((cost_items, cost_currencies))
    }

    async fn apply_talent_style(&self, hero_id: i32, template_id: i32, style: i32) -> Result<()> {
        let hero_data = self.get(hero_id).await?;

        if style != 0 {
            let has_style: Option<i32> = sqlx::query_scalar(
                "SELECT 1 FROM hero_talent_styles WHERE hero_uid = ? AND style_id = ?",
            )
            .bind(hero_data.record.uid)
            .bind(style)
            .fetch_optional(&self.pool)
            .await?;

            if has_style.is_none() {
                tracing::warn!(
                    "User {} does not own style {} for hero {}",
                    self.user_id,
                    style,
                    hero_id
                );
                return Err(anyhow::anyhow!("Style not owned"));
            }
        }

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query("UPDATE hero_talent_templates SET style = ? WHERE id = ?")
            .bind(style)
            .bind(template_row_id)
            .execute(&self.pool)
            .await?;

        tracing::info!(
            "User {} applied style {} to template {} for hero {}",
            self.user_id,
            style,
            template_id,
            hero_id
        );

        Ok(())
    }

    async fn switch_talent_template(
        &self,
        hero_id: i32,
        template_id: i32,
    ) -> Result<sonettobuf::TalentTemplateInfo> {
        let hero_data = self.get(hero_id).await?;

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero_data.record.uid)
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let cubes: Vec<(i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT cube_id, direction, pos_x, pos_y
             FROM hero_talent_template_cubes
             WHERE template_row_id = ?",
        )
        .bind(template_row_id)
        .fetch_all(&self.pool)
        .await?;

        let template_data: (String, i32) =
            sqlx::query_as("SELECT name, style FROM hero_talent_templates WHERE id = ?")
                .bind(template_row_id)
                .fetch_one(&self.pool)
                .await?;

        sqlx::query("DELETE FROM hero_talent_cubes WHERE hero_uid = ?")
            .bind(hero_data.record.uid)
            .execute(&self.pool)
            .await?;

        for (cube_id, direction, pos_x, pos_y) in &cubes {
            sqlx::query(
                "INSERT INTO hero_talent_cubes
                 (hero_uid, cube_id, direction, pos_x, pos_y)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(hero_data.record.uid)
            .bind(cube_id)
            .bind(direction)
            .bind(pos_x)
            .bind(pos_y)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query("UPDATE heroes SET use_talent_template_id = ? WHERE uid = ? AND user_id = ?")
            .bind(template_id)
            .bind(hero_data.record.uid)
            .bind(self.user_id)
            .execute(&self.pool)
            .await?;

        tracing::info!(
            "User {} switched to talent template {} for hero {}",
            self.user_id,
            template_id,
            hero_id
        );

        let talent_cube_infos = cubes
            .into_iter()
            .map(
                |(cube_id, direction, pos_x, pos_y)| sonettobuf::TalentCubeInfo {
                    cube_id: Some(cube_id),
                    direction: Some(direction),
                    pos_x: Some(pos_x),
                    pos_y: Some(pos_y),
                },
            )
            .collect();

        Ok(sonettobuf::TalentTemplateInfo {
            id: Some(template_id),
            talent_cube_infos,
            name: Some(template_data.0),
            style: Some(template_data.1),
        })
    }
}
