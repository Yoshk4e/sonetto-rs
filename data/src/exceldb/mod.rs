// Auto-generated module declarations

pub mod activity174_role;
pub mod activity191_role;
pub mod antique;
pub mod battle;
pub mod bgm_switch;
pub mod bonus;
pub mod chapter;
pub mod character;
pub mod character_cosume;
pub mod character_destiny;
pub mod character_destiny_facets;
pub mod character_level;
pub mod character_rank;
pub mod character_rank_replace;
pub mod character_talent;
pub mod character_voice;
pub mod cloth_level;
pub mod currency;
pub mod episode;
pub mod equip;
pub mod equip_break_cost;
pub mod equip_skill;
pub mod equip_strengthen;
pub mod equip_strengthen_cost;
pub mod guide;
pub mod hero_trial;
pub mod insight_item;
pub mod item;
pub mod monster;
pub mod monster_skill_template;
pub mod monster_template;
pub mod month_card;
pub mod open;
pub mod power_item;
pub mod skill;
pub mod skill_behavior;
pub mod skill_buff;
pub mod skill_effect;
pub mod skill_ex_level;
pub mod skill_passive_level;
pub mod skin;
pub mod store_charge_goods;
pub mod store_charge_optional;
pub mod store_goods;
pub mod summon;
pub mod summon_pool;
pub mod talent_scheme;
pub mod talent_style_cost;

use std::sync::OnceLock;

pub struct GameDB {
    pub activity174_role: activity174_role::Activity174RoleTable,
    pub activity191_role: activity191_role::Activity191RoleTable,
    pub antique: antique::AntiqueTable,
    pub battle: battle::BattleTable,
    pub bgm_switch: bgm_switch::BgmSwitchTable,
    pub bonus: bonus::BonusTable,
    pub chapter: chapter::ChapterTable,
    pub character: character::CharacterTable,
    pub character_cosume: character_cosume::CharacterCosumeTable,
    pub character_destiny: character_destiny::CharacterDestinyTable,
    pub character_destiny_facets: character_destiny_facets::CharacterDestinyFacetsTable,
    pub character_level: character_level::CharacterLevelTable,
    pub character_rank: character_rank::CharacterRankTable,
    pub character_rank_replace: character_rank_replace::CharacterRankReplaceTable,
    pub character_talent: character_talent::CharacterTalentTable,
    pub character_voice: character_voice::CharacterVoiceTable,
    pub cloth_level: cloth_level::ClothLevelTable,
    pub currency: currency::CurrencyTable,
    pub episode: episode::EpisodeTable,
    pub equip: equip::EquipTable,
    pub equip_break_cost: equip_break_cost::EquipBreakCostTable,
    pub equip_skill: equip_skill::EquipSkillTable,
    pub equip_strengthen: equip_strengthen::EquipStrengthenTable,
    pub equip_strengthen_cost: equip_strengthen_cost::EquipStrengthenCostTable,
    pub guide: guide::GuideTable,
    pub hero_trial: hero_trial::HeroTrialTable,
    pub insight_item: insight_item::InsightItemTable,
    pub item: item::ItemTable,
    pub monster: monster::MonsterTable,
    pub monster_skill_template: monster_skill_template::MonsterSkillTemplateTable,
    pub monster_template: monster_template::MonsterTemplateTable,
    pub month_card: month_card::MonthCardTable,
    pub open: open::OpenTable,
    pub power_item: power_item::PowerItemTable,
    pub skill: skill::SkillTable,
    pub skill_behavior: skill_behavior::SkillBehaviorTable,
    pub skill_buff: skill_buff::SkillBuffTable,
    pub skill_effect: skill_effect::SkillEffectTable,
    pub skill_ex_level: skill_ex_level::SkillExLevelTable,
    pub skill_passive_level: skill_passive_level::SkillPassiveLevelTable,
    pub skin: skin::SkinTable,
    pub store_charge_goods: store_charge_goods::StoreChargeGoodsTable,
    pub store_charge_optional: store_charge_optional::StoreChargeOptionalTable,
    pub store_goods: store_goods::StoreGoodsTable,
    pub summon: summon::SummonTable,
    pub summon_pool: summon_pool::SummonPoolTable,
    pub talent_scheme: talent_scheme::TalentSchemeTable,
    pub talent_style_cost: talent_style_cost::TalentStyleCostTable,
}

impl GameDB {
    pub fn load(data_dir: &str) -> anyhow::Result<Self> {
        let activity174_role = activity174_role::Activity174RoleTable::load(
            &format!("{}/activity174_role.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load activity174_role.json: {}", e))?;
        let activity191_role = activity191_role::Activity191RoleTable::load(
            &format!("{}/activity191_role.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load activity191_role.json: {}", e))?;
        let antique = antique::AntiqueTable::load(
            &format!("{}/antique.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load antique.json: {}", e))?;
        let battle = battle::BattleTable::load(
            &format!("{}/battle.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load battle.json: {}", e))?;
        let bgm_switch = bgm_switch::BgmSwitchTable::load(
            &format!("{}/bgm_switch.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load bgm_switch.json: {}", e))?;
        let bonus = bonus::BonusTable::load(
            &format!("{}/bonus.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load bonus.json: {}", e))?;
        let chapter = chapter::ChapterTable::load(
            &format!("{}/chapter.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load chapter.json: {}", e))?;
        let character = character::CharacterTable::load(
            &format!("{}/character.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character.json: {}", e))?;
        let character_cosume = character_cosume::CharacterCosumeTable::load(
            &format!("{}/character_cosume.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_cosume.json: {}", e))?;
        let character_destiny = character_destiny::CharacterDestinyTable::load(
            &format!("{}/character_destiny.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_destiny.json: {}", e))?;
        let character_destiny_facets = character_destiny_facets::CharacterDestinyFacetsTable::load(
            &format!("{}/character_destiny_facets.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_destiny_facets.json: {}", e))?;
        let character_level = character_level::CharacterLevelTable::load(
            &format!("{}/character_level.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_level.json: {}", e))?;
        let character_rank = character_rank::CharacterRankTable::load(
            &format!("{}/character_rank.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_rank.json: {}", e))?;
        let character_rank_replace = character_rank_replace::CharacterRankReplaceTable::load(
            &format!("{}/character_rank_replace.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_rank_replace.json: {}", e))?;
        let character_talent = character_talent::CharacterTalentTable::load(
            &format!("{}/character_talent.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_talent.json: {}", e))?;
        let character_voice = character_voice::CharacterVoiceTable::load(
            &format!("{}/character_voice.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load character_voice.json: {}", e))?;
        let cloth_level = cloth_level::ClothLevelTable::load(
            &format!("{}/cloth_level.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load cloth_level.json: {}", e))?;
        let currency = currency::CurrencyTable::load(
            &format!("{}/currency.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load currency.json: {}", e))?;
        let episode = episode::EpisodeTable::load(
            &format!("{}/episode.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load episode.json: {}", e))?;
        let equip = equip::EquipTable::load(
            &format!("{}/equip.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load equip.json: {}", e))?;
        let equip_break_cost = equip_break_cost::EquipBreakCostTable::load(
            &format!("{}/equip_break_cost.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load equip_break_cost.json: {}", e))?;
        let equip_skill = equip_skill::EquipSkillTable::load(
            &format!("{}/equip_skill.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load equip_skill.json: {}", e))?;
        let equip_strengthen = equip_strengthen::EquipStrengthenTable::load(
            &format!("{}/equip_strengthen.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load equip_strengthen.json: {}", e))?;
        let equip_strengthen_cost = equip_strengthen_cost::EquipStrengthenCostTable::load(
            &format!("{}/equip_strengthen_cost.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load equip_strengthen_cost.json: {}", e))?;
        let guide = guide::GuideTable::load(
            &format!("{}/guide.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load guide.json: {}", e))?;
        let hero_trial = hero_trial::HeroTrialTable::load(
            &format!("{}/hero_trial.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load hero_trial.json: {}", e))?;
        let insight_item = insight_item::InsightItemTable::load(
            &format!("{}/insight_item.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load insight_item.json: {}", e))?;
        let item = item::ItemTable::load(
            &format!("{}/item.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load item.json: {}", e))?;
        let monster = monster::MonsterTable::load(
            &format!("{}/monster.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load monster.json: {}", e))?;
        let monster_skill_template = monster_skill_template::MonsterSkillTemplateTable::load(
            &format!("{}/monster_skill_template.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load monster_skill_template.json: {}", e))?;
        let monster_template = monster_template::MonsterTemplateTable::load(
            &format!("{}/monster_template.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load monster_template.json: {}", e))?;
        let month_card = month_card::MonthCardTable::load(
            &format!("{}/month_card.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load month_card.json: {}", e))?;
        let open = open::OpenTable::load(
            &format!("{}/open.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load open.json: {}", e))?;
        let power_item = power_item::PowerItemTable::load(
            &format!("{}/power_item.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load power_item.json: {}", e))?;
        let skill = skill::SkillTable::load(
            &format!("{}/skill.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skill.json: {}", e))?;
        let skill_behavior = skill_behavior::SkillBehaviorTable::load(
            &format!("{}/skill_behavior.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_behavior.json: {}", e))?;
        let skill_buff = skill_buff::SkillBuffTable::load(
            &format!("{}/skill_buff.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_buff.json: {}", e))?;
        let skill_effect = skill_effect::SkillEffectTable::load(
            &format!("{}/skill_effect.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_effect.json: {}", e))?;
        let skill_ex_level = skill_ex_level::SkillExLevelTable::load(
            &format!("{}/skill_ex_level.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_ex_level.json: {}", e))?;
        let skill_passive_level = skill_passive_level::SkillPassiveLevelTable::load(
            &format!("{}/skill_passive_level.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skill_passive_level.json: {}", e))?;
        let skin = skin::SkinTable::load(
            &format!("{}/skin.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load skin.json: {}", e))?;
        let store_charge_goods = store_charge_goods::StoreChargeGoodsTable::load(
            &format!("{}/store_charge_goods.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load store_charge_goods.json: {}", e))?;
        let store_charge_optional = store_charge_optional::StoreChargeOptionalTable::load(
            &format!("{}/store_charge_optional.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load store_charge_optional.json: {}", e))?;
        let store_goods = store_goods::StoreGoodsTable::load(
            &format!("{}/store_goods.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load store_goods.json: {}", e))?;
        let summon = summon::SummonTable::load(
            &format!("{}/summon.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load summon.json: {}", e))?;
        let summon_pool = summon_pool::SummonPoolTable::load(
            &format!("{}/summon_pool.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load summon_pool.json: {}", e))?;
        let talent_scheme = talent_scheme::TalentSchemeTable::load(
            &format!("{}/talent_scheme.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load talent_scheme.json: {}", e))?;
        let talent_style_cost = talent_style_cost::TalentStyleCostTable::load(
            &format!("{}/talent_style_cost.json", data_dir)
        ).map_err(|e| anyhow::anyhow!("Failed to load talent_style_cost.json: {}", e))?;

        Ok(Self {
            activity174_role,
            activity191_role,
            antique,
            battle,
            bgm_switch,
            bonus,
            chapter,
            character,
            character_cosume,
            character_destiny,
            character_destiny_facets,
            character_level,
            character_rank,
            character_rank_replace,
            character_talent,
            character_voice,
            cloth_level,
            currency,
            episode,
            equip,
            equip_break_cost,
            equip_skill,
            equip_strengthen,
            equip_strengthen_cost,
            guide,
            hero_trial,
            insight_item,
            item,
            monster,
            monster_skill_template,
            monster_template,
            month_card,
            open,
            power_item,
            skill,
            skill_behavior,
            skill_buff,
            skill_effect,
            skill_ex_level,
            skill_passive_level,
            skin,
            store_charge_goods,
            store_charge_optional,
            store_goods,
            summon,
            summon_pool,
            talent_scheme,
            talent_style_cost,
        })
    }

    pub fn global() -> &'static GameDB {
        static DB: OnceLock<GameDB> = OnceLock::new();
        DB.get_or_init(|| {
            Self::load("data").expect("Failed to load game database")
        })
    }
}

static GAME_DATA: OnceLock<GameDB> = OnceLock::new();

pub fn init(data_dir: &str) -> anyhow::Result<()> {
    let db = GameDB::load(data_dir)?;
    GAME_DATA.set(db)
        .map_err(|_| anyhow::anyhow!("Game data already initialized"))
}

#[inline]
pub fn get() -> &'static GameDB {
    GAME_DATA.get().expect("Game data not initialized. Call init() first.")
}

#[inline]
pub fn try_get() -> Option<&'static GameDB> {
    GAME_DATA.get()
}