#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum EffectType {
    None = 0,
    Miss = 1,
    Damage = 2,                             // Hurt/Damage dealt
    Crit = 3,                               // Critical hit
    Heal = 4,                               // Healing
    BuffAdd = 5,                            // Add buff
    BuffDel = 6,                            // Remove buff
    BuffUpdate = 7,                         // Update buff
    BuffEffect = 8,                         // Buff effect trigger
    Dead = 9,                               // Entity death
    AttackAlter = 10,                       // Attack stat modification
    DefenseAlter = 11,                      // Defense stat modification
    Bloodlust = 12,                         // Bloodlust mechanic
    Purify = 13,                            // Purify/cleanse
    Disperse = 14,                          // Disperse effect
    AddAct = 15,                            // Add action points
    AddCard = 16,                           // Add card
    AddExPoint = 17,                        // Add EX points
    DamageExtra = 18,                       // Extra damage
    BuffReject = 19,                        // Buff rejection
    Dizzy = 20,                             // Dizzy/stun status
    Invincible = 21,                        // Invincibility
    Protect = 22,                           // Protection
    Frozen = 23,                            // Frozen status
    Silence = 24,                           // Silence status
    Shield = 25,                            // Shield generation
    Attr = 26,                              // Attribute change (same as IndicatorChange)
    Cure = 27,                              // Cure status
    Seal = 28,                              // Seal status
    Disarm = 29,                            // Disarm status
    Forbid = 30,                            // Forbid status
    Sleep = 31,                             // Sleep status
    Petrified = 32,                         // Petrified status
    Immunity = 33,                          // Immunity status
    Injury = 34,                            // Injury effect
    Dot = 35,                               // Damage over time
    Rebound = 36,                           // Rebound/reflect
    Taunt = 37,                             // Taunt status
    BeatBack = 38,                          // Knockback
    ExPointFix = 39,                        // Fix EX points
    AverageLife = 40,                       // Average life
    ShieldChange = 41,                      // Shield value change
    AddToAttacker = 42,                     // Add to attacker
    Cure2 = 43,                             // Secondary cure
    ForbidSpecEffect = 44,                  // Forbid special effect
    CantCrit = 45,                          // Cannot critical
    PetrifiedResist = 46,                   // Petrified resistance
    SleepResist = 47,                       // Sleep resistance
    FrozenResist = 48,                      // Frozen resistance
    DizzyResist = 49,                       // Dizzy resistance
    AddToTarget = 50,                       // Add to target
    CritPileup = 51,                        // Critical pileup
    DodgeSpecSkill = 52,                    // Dodge special skill
    DodgeSpecSkill2 = 53,                   // Dodge special skill 2
    ReDealCard = 54,                        // Re-deal cards
    BuffAddNoEffect = 55,                   // Add buff without effect
    BuffDelNoEffect = 56,                   // Remove buff without effect
    HealCrit = 57,                          // Critical heal
    UniversalCard = 58,                     // Universal card
    DealCard1 = 59,                         // Deal card type 1
    DealCard2 = 60,                         // Deal card type 2
    RoundEnd = 61,                          // End of round
    ShieldDel = 62,                         // Remove shield
    ExPointCantAdd = 63,                    // Cannot add EX points
    AddBuffRound = 64,                      // Add buff duration
    CardLevelAdd = 65,                      // Increase card level
    ImmunityExPointChange = 66,             // Immunity EX point change
    MonsterChange = 67,                     // Monster transformation
    ExPointAdd = 68,                        // EX point addition
    ExPointDel = 69,                        // EX point deletion
    DamageNotMoreThan = 70,                 // Damage cap
    BuffAttr = 71,                          // Buff attribute
    ExPointCardMove = 72,                   // EX point card move
    ExPointCardUpgrade = 73,                // EX point card upgrade
    FixedHurt = 74,                         // Fixed damage
    CardLevelChange = 75,                   // Card level change
    BuffReplace = 76,                       // Buff replacement
    ExtraMoveAct = 77,                      // Extra move action
    SpCardAdd = 78,                         // Special card add
    Rigid = 79,                             // Rigid/stiff status
    Cold = 80,                              // Cold status
    Palsy = 81,                             // Palsy status
    AddBuffRoundByTypeId = 82,              // Add buff round by type
    ExSkillNoConsumption = 83,              // EX skill no consumption
    ExPointAddAfterDelOrAbsorbExPoint = 84, // EX point add after delete/absorb
    CardEffectChange = 85,                  // Card effect change
    Summon = 86,                            // Summon entity
    SkillWeightSelect = 87,                 // Skill weight selection
    SkillPowerUp = 88,                      // Skill power up
    BuffRateUp = 89,                        // Buff rate up
    SkillRateUp = 90,                       // Skill rate up
    ExPointMaxAdd = 91,                     // EX point max increase
    HaloBase = 92,                          // Halo base
    HaloSlave = 93,                         // Halo slave
    SelectLast = 94,                        // Select last
    CantSelect = 95,                        // Cannot select
    ClearUniversalCard = 96,                // Clear universal card
    ChangeCareer = 97,                      // Change career
    FixedDamage = 98,                       // Fixed damage
    PassiveSkillInvalid = 99,               // Passive skill invalid
    HideLife = 100,                         // Hide life
    BuffAddAct = 101,                       // Buff add action
    AddCardLimit = 102,                     // Add card limit
    AddBuffRoundByTypeGroup = 103,          // Add buff round by type group
    Freeze = 104,                           // Freeze effect
    CantSelectEx = 105,                     // Cannot select EX
    CardDisappear = 106,                    // Card disappear
    ChangeHero = 107,                       // Change hero
    MaxHpChange = 108,                      // Max HP change
    CurrentHpChange = 109,                  // Current HP change
    Kill = 110,                             // Kill effect
    ExPointChange = 111,                    // EX point change
    MonsterSpLife = 112,                    // Monster special life
    ExSkillPointChange = 113,               // EX skill point change
    HarmStatistic = 114,                    // Harm statistic
    OverflowHealToShield = 115,             // Overflow heal to shield
    AddSkillBuffCountAndDuration = 116,     // Add skill buff count and duration
    IndicatorChange = 117,                  // Indicator change
    PowerChange = 128,                      // Power change
    CantGetExSkill = 129,                   // Cannot get EX skill
    OriginDamage = 130,                     // Origin damage
    OriginCrit = 131,                       // Origin critical
    ShieldBroken = 132,                     // Shield broken
    CardRemove = 133,                       // Card remove
    SummonedAdd = 134,                      // Summoned add
    SummonedDelete = 135,                   // Summoned delete
    SummonedLevelUp = 136,                  // Summoned level up
    Burn = 137,                             // Burn status
    MagicCircleAdd = 138,                   // Magic circle add
    MagicCircleDelete = 139,                // Magic circle delete
    MagicCircleUpdate = 140,                // Magic circle update
    ChangeToTempCard = 141,                 // Change to temp card
    RogueHeartChange = 142,                 // Rogue heart change
    RogueCoinChange = 143,                  // Rogue coin change
    RogueSaveCoin = 144,                    // Rogue save coin
    RogueEscape = 145,                      // Rogue escape
    RegainPower = 146,                      // Regain power
    AddToBuffEntity = 147,                  // Add to buff entity
    MasterPowerChange = 148,                // Master power change
    AddHandCard = 149,                      // Add card to hand
    OverflowPowerAddBuff = 150,             // Overflow power add buff
    PowerCantDecr = 151,                    // Power cannot decrease
    RemoveEntityCards = 152,                // Remove entity cards
    CardsCompose = 153,                     // Cards compose/merge
    CardsPush = 154,                        // Cards push (initial deal)
    CardRemove2 = 155,                      // Card remove 2
    BfsgConvertCard = 156,                  // BFSG convert card
    BfsgUseCard = 157,                      // BFSG use card
    BfsgSkillEnd = 158,                     // BFSG skill end
    UseCards = 159,                         // Use cards (deal during battle)
    CardInvalid = 160,                      // Card invalid
    BfsgSkillStart = 161,                   // BFSG skill start
    FightStep = 162,                        // Nested fight step
    IgnoreDodgeSpecSkill = 163,             // Ignore dodge special skill
    IgnoreCounter = 164,                    // Ignore counter
    IgnoreRebound = 165,                    // Ignore rebound
    CareerRestraint = 166,                  // Career restraint
    StorageInjury = 167,                    // Storage injury
    InjuryLogBack = 168,                    // Injury log back
    AbsorbHurt = 169,                       // Absorb hurt
    CardAConvertCardB = 170,                // Card A convert to card B
    HeroUpgrade = 171,                      // Hero upgrade
    MasterHalo = 172,                       // Master halo
    SlaveHalo = 173,                        // Slave halo
    NotifyUpgradeHero = 174,                // Notify upgrade hero
    PolarizationAddLimit = 175,             // Polarization add limit
    PolarizationDecCard = 176,              // Polarization decrease card
    PolarizationAddLevel = 177,             // Polarization add level
    PolarizationExSkillAdd = 178,           // Polarization EX skill add
    ResonanceAddLimit = 179,                // Resonance add limit
    ResonanceDecCard = 180,                 // Resonance decrease card
    ResonanceAddLevel = 181,                // Resonance add level
    ResonanceExSkillAdd = 182,              // Resonance EX skill add
    PolarizationLevel = 183,                // Polarization level
    ResonanceLevel = 184,                   // Resonance level
    PolarizationActive = 185,               // Polarization active
    ResonanceActive = 186,                  // Resonance active
    RougeReward = 187,                      // Rogue reward
    RougePowerLimitChange = 188,            // Rogue power limit change
    RougePowerChange = 189,                 // Rogue power change
    RougeCoinChange2 = 190,                 // Rogue coin change 2
    RougeSpCardAdd = 191,                   // Rogue special card add
    DamageFromAbsorb = 192,                 // Damage from absorb
    DamageFromLostHp = 193,                 // Damage from lost HP
    RecordTeamInjuryCount = 194,            // Record team injury count
    InjuryBankHeal = 195,                   // Injury bank heal
    MasterCardRemove = 196,                 // Master card remove
    MasterAddHandCard = 197,                // Master add hand card
    FightCounter = 198,                     // Fight counter
    IgnoreBeatBack = 199,                   // Ignore beat back
    EnterTeamStage = 200,                   // Enter team stage
    MockTaunt = 201,                        // Mock taunt
    EnchantBurnDamage = 202,                // Enchant burn damage
    RealHurtFixWithLimit = 203,             // Real hurt fix with limit
    BuffTypeNumLimitUpdate = 204,           // Buff type num limit update
    LockHp = 205,                           // Lock HP
    Move = 206,                             // Move
    MoveFront = 207,                        // Move front
    MoveBack = 208,                         // Move back
    SkillLevelJudgeAdd = 209,               // Skill level judge add
    TeammateInjuryCount = 210,              // Teammate injury count
    SmallRoundEnd = 211,                    // Small round end
    ChangeRound = 212,                      // Change round
    Poison = 213,                           // Poison status
    ExPointOverflowBank = 214,              // EX point overflow bank
    AddUseCard = 215,                       // Add use card
    LockDot = 216,                          // Lock DOT
    CatapultBuff = 217,                     // Catapult buff
    PlayAroundUpRank = 218,                 // Play around up rank
    PlayAroundDownRank = 219,               // Play around down rank
    PlaySetGray = 220,                      // Play set gray
    ResistancesAttr = 221,                  // Resistances attribute
    Resistances = 222,                      // Resistances
    AddBuffRoundBySkill = 223,              // Add buff round by skill
    PlayChangeRankFail = 224,               // Play change rank fail
    CopyBuffByKill = 225,                   // Copy buff by kill
    PoisonSettleCanCrit = 226,              // Poison settle can crit
    ChangeWave = 227,                       // Change wave
    ShieldValueChange = 228,                // Shield value change
    BreakShield = 229,                      // Break shield
    StressTrigger = 230,                    // Stress trigger
    LayerMasterHalo = 231,                  // Layer master halo
    LayerSlaveHalo = 232,                   // Layer slave halo
    EnterFightDeal = 233,                   // Enter fight deal
    LayerHaloSync = 234,                    // Layer halo sync
    SubHeroLifeChange = 235,                // Sub hero life change
    GuardChange = 236,                      // Guard change
    LockBulletCountDecr = 237,              // Lock bullet count decrease
    EntitySync = 238,                       // Entity sync
    PrecisionRegion = 239,                  // Precision region
    TransferAddExPoint = 240,               // Transfer add EX point
    NotifyHeroContract = 241,               // Notify hero contract
    Contract = 242,                         // Contract
    BeContracted = 243,                     // Be contracted
    SpExPointMaxAdd = 244,                  // Special EX point max add
    TransferAddStress = 245,                // Transfer add stress
    GuardBreak = 246,                       // Guard break
    CardDeckGenerate = 247,                 // Card deck generate
    CardDeckDelete = 248,                   // Card deck delete
    DelCardAndDamage = 249,                 // Delete card and damage
    Charm = 250,                            // Charm status
    ProgressChange = 251,                   // Progress change
    AssistBossSkillCd = 252,                // Assist boss skill CD
    DamageShareHp = 253,                    // Damage share HP
    UseCardFixExPoint = 254,                // Use card fix EX point
    DeadlyPoison = 255,                     // Deadly poison
    ProgressMaxChange = 256,                // Progress max change
    DuduBoneContinueChannel = 257,          // Dudu bone continue channel
    ZxqRemoveCard = 258,                    // ZXQ remove card
    CureCorrect = 259,                      // Cure correct
    AssistBossChange = 260,                 // Assist boss change
    Confusion = 261,                        // Confusion status
    RetainPetrified = 262,                  // Retain petrified
    DeadlyPoisonOriginDamage = 263,         // Deadly poison origin damage
    DeadlyPoisonOriginCrit = 264,           // Deadly poison origin crit
    AssistBossSkillChange = 265,            // Assist boss skill change
    LockBurn = 266,                         // Lock burn
    AdditionalDamage = 267,                 // Additional damage
    AdditionalDamageCrit = 268,             // Additional damage crit
    Act174First = 269,                      // Act 174 first
    Act174UseCard = 270,                    // Act 174 use card
    ChangeShield = 271,                     // Change shield
    TowerScoreChange = 272,                 // Tower score change
    Act174MonsterAiCard = 273,              // Act 174 monster AI card
    AfterReDealCard = 274,                  // After re-deal card
    TeamEnergyChange = 275,                 // Team energy change
    AllocateCardEnergy = 276,               // Allocate card energy
    EmitterEnergyChange = 277,              // Emitter energy change
    EmitterSkillEnd = 278,                  // Emitter skill end
    CardDeckClear = 279,                    // Card deck clear
    EmitterCreate = 280,                    // Emitter create
    AddOnceCard = 281,                      // Add once card
    ShareHurt = 282,                        // Share hurt
    PlayerFinisherSkillChange = 283,        // Player finisher skill change
    EmitterCareerChange = 284,              // Emitter career change
    EmitterNumChange = 285,                 // Emitter num change
    EmitterTag = 286,                       // Emitter tag
    EmitterRemove = 287,                    // Emitter remove
    UseSkillTeamAddEmitterEnergy = 288,     // Use skill team add emitter energy
    FixAttrTeamEnergy = 289,                // Fix attr team energy
    SimplePolarizationActive = 290,         // Simple polarization active
    SimplePolarizationLevel = 291,          // Simple polarization level
    SimplePolarizationAddLevel = 292,       // Simple polarization add level
    CallMonsterToSub = 293,                 // Call monster to sub
    FixAttrTeamEnergyAndBuff = 294,         // Fix attr team energy and buff
    PowerInfoChange = 295,                  // Power info change
    SimplePolarizationAddLimit = 296,       // Simple polarization add limit
    EmitterMainTarget = 297,                // Emitter main target
    ConditionSplitEmitterNum = 298,         // Condition split emitter num
    AddSplitEmitterNum = 299,               // Add split emitter num
    EmitterFightNotify = 300,               // Emitter fight notify
    MustCritBuff = 301,                     // Must crit buff
    MustCrit = 302,                         // Must crit
    CardAreaRedOrBlue = 303,                // Card area red or blue
    ToCardAreaRedOrBlue = 304,              // To card area red or blue
    RedOrBlueCount = 305,                   // Red or blue count
    RedOrBlueCountChange = 306,             // Red or blue count change
    RedOrBlueChangeTrigger = 307,           // Red or blue change trigger
    CardHeatInit = 308,                     // Card heat init
    CardHeatValueChange = 309,              // Card heat value change
    CardDeckNum = 310,                      // Card deck num
    RedOrBlueCountExSkill = 311,            // Red or blue count EX skill
    StorageDamage = 312,                    // Storage damage
    Elusive = 313,                          // Elusive status
    EnchantDepresseDamage = 314,            // Enchant depress damage
    SaveFightRecordStart = 315,             // Save fight record start
    SaveFightRecordUpdate = 316,            // Save fight record update
    SaveFightRecordEnd = 317,               // Save fight record end
    RoundOffset = 318,                      // Round offset
    SaveFightRecord = 319,                  // Save fight record
    AddSpHandCard = 320,                    // Add special hand card
    NonCareerRestraint = 321,               // Non career restraint
    ClearMonsterSub = 322,                  // Clear monster sub
    FightTaskUpdate = 323,                  // Fight task update
    RetainSleep = 324,                      // Retain sleep
    RemoveMonsterSub = 325,                 // Remove monster sub
    AddCardRecordByRound = 326,             // Add card record by round
    DirectUseExSkill = 327,                 // Direct use EX skill
    SplitStart = 328,                       // Split start
    SplitEnd = 329,                         // Split end
    FightParamChange = 330,                 // Fight param change
    BloodPoolMaxCreate = 333,               // Blood pool max create
    BloodPoolMaxChange = 334,               // Blood pool max change
    BloodPoolValueChange = 335,             // Blood pool value change
    ColdSaturdayHurt = 336,                 // Cold saturday hurt
    NewChangeWave = 337,                    // New change wave
    ChangeCardEnergy = 338,                 // Change card energy
    ClientEffect = 339,                     // Client effect
    MagicCircleUpgrade = 340,               // Magic circle upgrade
    NuoDiKaRandomAttack = 341,              // Nuo Di Ka random attack
    NuoDiKaTeamAttack = 342,                // Nuo Di Ka team attack
    TriggerAnalysis = 343,                  // Trigger analysis
    GetSecretKey = 344,                     // Get secret key
    SurvivalHealthChange = 345,             // Survival health change
    LockHpMax = 346,                        // Lock HP max
    CureUpByLostHp = 347,                   // Cure up by lost HP
    NoUseCardEnergyRecordByRound = 348,     // No use card energy record by round
    NuoDiKaRandomAttackNum = 349,           // Nuo Di Ka random attack num
    BuffActInfoUpdate = 350,                // Buff act info update
    RealDamageKill = 351,                   // Real damage kill
    BuffDelReason = 352,                    // Buff delete reason
    RandomDiceUseSkill = 353,               // Random dice use skill
    TowerDeepChange = 354,                  // Tower deep change
    FightHurtDetail = 355,                  // Fight hurt detail

    // Special/Trigger
    Trigger = 999,

    // Custom effects (for specific content)
    EzioBigSkillDamage = 1000,
    EzioBigSkillOriginDamage = 1001,
    UpdateItemPlayerSkill = 1002,
    EzioBigSkillExit = 1003,

    // Unknown
    Unknown = -1,
}

impl EffectType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Miss,
            2 => Self::Damage,
            3 => Self::Crit,
            4 => Self::Heal,
            5 => Self::BuffAdd,
            6 => Self::BuffDel,
            7 => Self::BuffUpdate,
            8 => Self::BuffEffect,
            9 => Self::Dead,
            10 => Self::AttackAlter,
            11 => Self::DefenseAlter,
            12 => Self::Bloodlust,
            13 => Self::Purify,
            14 => Self::Disperse,
            15 => Self::AddAct,
            16 => Self::AddCard,
            17 => Self::AddExPoint,
            18 => Self::DamageExtra,
            19 => Self::BuffReject,
            20 => Self::Dizzy,
            21 => Self::Invincible,
            22 => Self::Protect,
            23 => Self::Frozen,
            24 => Self::Silence,
            25 => Self::Shield,
            26 => Self::Attr,
            27 => Self::Cure,
            28 => Self::Seal,
            29 => Self::Disarm,
            30 => Self::Forbid,
            31 => Self::Sleep,
            32 => Self::Petrified,
            33 => Self::Immunity,
            34 => Self::Injury,
            35 => Self::Dot,
            36 => Self::Rebound,
            37 => Self::Taunt,
            38 => Self::BeatBack,
            39 => Self::ExPointFix,
            40 => Self::AverageLife,
            41 => Self::ShieldChange,
            42 => Self::AddToAttacker,
            43 => Self::Cure2,
            44 => Self::ForbidSpecEffect,
            45 => Self::CantCrit,
            46 => Self::PetrifiedResist,
            47 => Self::SleepResist,
            48 => Self::FrozenResist,
            49 => Self::DizzyResist,
            50 => Self::AddToTarget,
            51 => Self::CritPileup,
            52 => Self::DodgeSpecSkill,
            53 => Self::DodgeSpecSkill2,
            54 => Self::ReDealCard,
            55 => Self::BuffAddNoEffect,
            56 => Self::BuffDelNoEffect,
            57 => Self::HealCrit,
            58 => Self::UniversalCard,
            59 => Self::DealCard1,
            60 => Self::DealCard2,
            61 => Self::RoundEnd,
            62 => Self::ShieldDel,
            63 => Self::ExPointCantAdd,
            64 => Self::AddBuffRound,
            65 => Self::CardLevelAdd,
            66 => Self::ImmunityExPointChange,
            67 => Self::MonsterChange,
            68 => Self::ExPointAdd,
            69 => Self::ExPointDel,
            70 => Self::DamageNotMoreThan,
            71 => Self::BuffAttr,
            72 => Self::ExPointCardMove,
            73 => Self::ExPointCardUpgrade,
            74 => Self::FixedHurt,
            75 => Self::CardLevelChange,
            76 => Self::BuffReplace,
            77 => Self::ExtraMoveAct,
            78 => Self::SpCardAdd,
            79 => Self::Rigid,
            80 => Self::Cold,
            81 => Self::Palsy,
            82 => Self::AddBuffRoundByTypeId,
            83 => Self::ExSkillNoConsumption,
            84 => Self::ExPointAddAfterDelOrAbsorbExPoint,
            85 => Self::CardEffectChange,
            86 => Self::Summon,
            87 => Self::SkillWeightSelect,
            88 => Self::SkillPowerUp,
            89 => Self::BuffRateUp,
            90 => Self::SkillRateUp,
            91 => Self::ExPointMaxAdd,
            92 => Self::HaloBase,
            93 => Self::HaloSlave,
            94 => Self::SelectLast,
            95 => Self::CantSelect,
            96 => Self::ClearUniversalCard,
            97 => Self::ChangeCareer,
            98 => Self::FixedDamage,
            99 => Self::PassiveSkillInvalid,
            100 => Self::HideLife,
            101 => Self::BuffAddAct,
            102 => Self::AddCardLimit,
            103 => Self::AddBuffRoundByTypeGroup,
            104 => Self::Freeze,
            105 => Self::CantSelectEx,
            106 => Self::CardDisappear,
            107 => Self::ChangeHero,
            108 => Self::MaxHpChange,
            109 => Self::CurrentHpChange,
            110 => Self::Kill,
            111 => Self::ExPointChange,
            112 => Self::MonsterSpLife,
            113 => Self::ExSkillPointChange,
            114 => Self::HarmStatistic,
            115 => Self::OverflowHealToShield,
            116 => Self::AddSkillBuffCountAndDuration,
            117 => Self::IndicatorChange,
            128 => Self::PowerChange,
            129 => Self::CantGetExSkill,
            130 => Self::OriginDamage,
            131 => Self::OriginCrit,
            132 => Self::ShieldBroken,
            133 => Self::CardRemove,
            134 => Self::SummonedAdd,
            135 => Self::SummonedDelete,
            136 => Self::SummonedLevelUp,
            137 => Self::Burn,
            138 => Self::MagicCircleAdd,
            139 => Self::MagicCircleDelete,
            140 => Self::MagicCircleUpdate,
            141 => Self::ChangeToTempCard,
            142 => Self::RogueHeartChange,
            143 => Self::RogueCoinChange,
            144 => Self::RogueSaveCoin,
            145 => Self::RogueEscape,
            146 => Self::RegainPower,
            147 => Self::AddToBuffEntity,
            148 => Self::MasterPowerChange,
            149 => Self::AddHandCard,
            150 => Self::OverflowPowerAddBuff,
            151 => Self::PowerCantDecr,
            152 => Self::RemoveEntityCards,
            153 => Self::CardsCompose,
            154 => Self::CardsPush,
            155 => Self::CardRemove2,
            156 => Self::BfsgConvertCard,
            157 => Self::BfsgUseCard,
            158 => Self::BfsgSkillEnd,
            159 => Self::UseCards,
            160 => Self::CardInvalid,
            161 => Self::BfsgSkillStart,
            162 => Self::FightStep,
            163 => Self::IgnoreDodgeSpecSkill,
            164 => Self::IgnoreCounter,
            165 => Self::IgnoreRebound,
            166 => Self::CareerRestraint,
            167 => Self::StorageInjury,
            168 => Self::InjuryLogBack,
            169 => Self::AbsorbHurt,
            170 => Self::CardAConvertCardB,
            171 => Self::HeroUpgrade,
            172 => Self::MasterHalo,
            173 => Self::SlaveHalo,
            174 => Self::NotifyUpgradeHero,
            175 => Self::PolarizationAddLimit,
            176 => Self::PolarizationDecCard,
            177 => Self::PolarizationAddLevel,
            178 => Self::PolarizationExSkillAdd,
            179 => Self::ResonanceAddLimit,
            180 => Self::ResonanceDecCard,
            181 => Self::ResonanceAddLevel,
            182 => Self::ResonanceExSkillAdd,
            183 => Self::PolarizationLevel,
            184 => Self::ResonanceLevel,
            185 => Self::PolarizationActive,
            186 => Self::ResonanceActive,
            187 => Self::RougeReward,
            188 => Self::RougePowerLimitChange,
            189 => Self::RougePowerChange,
            190 => Self::RougeCoinChange2,
            191 => Self::RougeSpCardAdd,
            192 => Self::DamageFromAbsorb,
            193 => Self::DamageFromLostHp,
            194 => Self::RecordTeamInjuryCount,
            195 => Self::InjuryBankHeal,
            196 => Self::MasterCardRemove,
            197 => Self::MasterAddHandCard,
            198 => Self::FightCounter,
            199 => Self::IgnoreBeatBack,
            200 => Self::EnterTeamStage,
            201 => Self::MockTaunt,
            202 => Self::EnchantBurnDamage,
            203 => Self::RealHurtFixWithLimit,
            204 => Self::BuffTypeNumLimitUpdate,
            205 => Self::LockHp,
            206 => Self::Move,
            207 => Self::MoveFront,
            208 => Self::MoveBack,
            209 => Self::SkillLevelJudgeAdd,
            210 => Self::TeammateInjuryCount,
            211 => Self::SmallRoundEnd,
            212 => Self::ChangeRound,
            213 => Self::Poison,
            214 => Self::ExPointOverflowBank,
            215 => Self::AddUseCard,
            216 => Self::LockDot,
            217 => Self::CatapultBuff,
            218 => Self::PlayAroundUpRank,
            219 => Self::PlayAroundDownRank,
            220 => Self::PlaySetGray,
            221 => Self::ResistancesAttr,
            222 => Self::Resistances,
            223 => Self::AddBuffRoundBySkill,
            224 => Self::PlayChangeRankFail,
            225 => Self::CopyBuffByKill,
            226 => Self::PoisonSettleCanCrit,
            227 => Self::ChangeWave,
            228 => Self::ShieldValueChange,
            229 => Self::BreakShield,
            230 => Self::StressTrigger,
            231 => Self::LayerMasterHalo,
            232 => Self::LayerSlaveHalo,
            233 => Self::EnterFightDeal,
            234 => Self::LayerHaloSync,
            235 => Self::SubHeroLifeChange,
            236 => Self::GuardChange,
            237 => Self::LockBulletCountDecr,
            238 => Self::EntitySync,
            239 => Self::PrecisionRegion,
            240 => Self::TransferAddExPoint,
            241 => Self::NotifyHeroContract,
            242 => Self::Contract,
            243 => Self::BeContracted,
            244 => Self::SpExPointMaxAdd,
            245 => Self::TransferAddStress,
            246 => Self::GuardBreak,
            247 => Self::CardDeckGenerate,
            248 => Self::CardDeckDelete,
            249 => Self::DelCardAndDamage,
            250 => Self::Charm,
            251 => Self::ProgressChange,
            252 => Self::AssistBossSkillCd,
            253 => Self::DamageShareHp,
            254 => Self::UseCardFixExPoint,
            255 => Self::DeadlyPoison,
            256 => Self::ProgressMaxChange,
            257 => Self::DuduBoneContinueChannel,
            258 => Self::ZxqRemoveCard,
            259 => Self::CureCorrect,
            260 => Self::AssistBossChange,
            261 => Self::Confusion,
            262 => Self::RetainPetrified,
            263 => Self::DeadlyPoisonOriginDamage,
            264 => Self::DeadlyPoisonOriginCrit,
            265 => Self::AssistBossSkillChange,
            266 => Self::LockBurn,
            267 => Self::AdditionalDamage,
            268 => Self::AdditionalDamageCrit,
            269 => Self::Act174First,
            270 => Self::Act174UseCard,
            271 => Self::ChangeShield,
            272 => Self::TowerScoreChange,
            273 => Self::Act174MonsterAiCard,
            274 => Self::AfterReDealCard,
            275 => Self::TeamEnergyChange,
            276 => Self::AllocateCardEnergy,
            277 => Self::EmitterEnergyChange,
            278 => Self::EmitterSkillEnd,
            279 => Self::CardDeckClear,
            280 => Self::EmitterCreate,
            281 => Self::AddOnceCard,
            282 => Self::ShareHurt,
            283 => Self::PlayerFinisherSkillChange,
            284 => Self::EmitterCareerChange,
            285 => Self::EmitterNumChange,
            286 => Self::EmitterTag,
            287 => Self::EmitterRemove,
            288 => Self::UseSkillTeamAddEmitterEnergy,
            289 => Self::FixAttrTeamEnergy,
            290 => Self::SimplePolarizationActive,
            291 => Self::SimplePolarizationLevel,
            292 => Self::SimplePolarizationAddLevel,
            293 => Self::CallMonsterToSub,
            294 => Self::FixAttrTeamEnergyAndBuff,
            295 => Self::PowerInfoChange,
            296 => Self::SimplePolarizationAddLimit,
            297 => Self::EmitterMainTarget,
            298 => Self::ConditionSplitEmitterNum,
            299 => Self::AddSplitEmitterNum,
            300 => Self::EmitterFightNotify,
            301 => Self::MustCritBuff,
            302 => Self::MustCrit,
            303 => Self::CardAreaRedOrBlue,
            304 => Self::ToCardAreaRedOrBlue,
            305 => Self::RedOrBlueCount,
            306 => Self::RedOrBlueCountChange,
            307 => Self::RedOrBlueChangeTrigger,
            308 => Self::CardHeatInit,
            309 => Self::CardHeatValueChange,
            310 => Self::CardDeckNum,
            311 => Self::RedOrBlueCountExSkill,
            312 => Self::StorageDamage,
            313 => Self::Elusive,
            314 => Self::EnchantDepresseDamage,
            315 => Self::SaveFightRecordStart,
            316 => Self::SaveFightRecordUpdate,
            317 => Self::SaveFightRecordEnd,
            318 => Self::RoundOffset,
            319 => Self::SaveFightRecord,
            320 => Self::AddSpHandCard,
            321 => Self::NonCareerRestraint,
            322 => Self::ClearMonsterSub,
            323 => Self::FightTaskUpdate,
            324 => Self::RetainSleep,
            325 => Self::RemoveMonsterSub,
            326 => Self::AddCardRecordByRound,
            327 => Self::DirectUseExSkill,
            328 => Self::SplitStart,
            329 => Self::SplitEnd,
            330 => Self::FightParamChange,
            333 => Self::BloodPoolMaxCreate,
            334 => Self::BloodPoolMaxChange,
            335 => Self::BloodPoolValueChange,
            336 => Self::ColdSaturdayHurt,
            337 => Self::NewChangeWave,
            338 => Self::ChangeCardEnergy,
            339 => Self::ClientEffect,
            340 => Self::MagicCircleUpgrade,
            341 => Self::NuoDiKaRandomAttack,
            342 => Self::NuoDiKaTeamAttack,
            343 => Self::TriggerAnalysis,
            344 => Self::GetSecretKey,
            345 => Self::SurvivalHealthChange,
            346 => Self::LockHpMax,
            347 => Self::CureUpByLostHp,
            348 => Self::NoUseCardEnergyRecordByRound,
            349 => Self::NuoDiKaRandomAttackNum,
            350 => Self::BuffActInfoUpdate,
            351 => Self::RealDamageKill,
            352 => Self::BuffDelReason,
            353 => Self::RandomDiceUseSkill,
            354 => Self::TowerDeepChange,
            355 => Self::FightHurtDetail,
            999 => Self::Trigger,
            1000 => Self::EzioBigSkillDamage,
            1001 => Self::EzioBigSkillOriginDamage,
            1002 => Self::UpdateItemPlayerSkill,
            1003 => Self::EzioBigSkillExit,
            _ => Self::Unknown,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }

    pub fn is_hp_modification(&self) -> bool {
        matches!(
            self,
            Self::Damage
                | Self::Crit
                | Self::Heal
                | Self::HealCrit
                | Self::CurrentHpChange
                | Self::MaxHpChange
                | Self::FixedDamage
                | Self::FixedHurt
                | Self::OriginDamage
                | Self::OriginCrit
        )
    }

    pub fn is_damage(&self) -> bool {
        matches!(
            self,
            Self::Damage
                | Self::Crit
                | Self::FixedDamage
                | Self::FixedHurt
                | Self::OriginDamage
                | Self::OriginCrit
                | Self::Burn
                | Self::Poison
                | Self::Dot
                | Self::DamageExtra
                | Self::AdditionalDamage
                | Self::AdditionalDamageCrit
        )
    }

    pub fn is_healing(&self) -> bool {
        matches!(
            self,
            Self::Heal
                | Self::HealCrit
                | Self::Cure
                | Self::Cure2
                | Self::CureCorrect
                | Self::CureUpByLostHp
        )
    }

    pub fn is_buffing(&self) -> bool {
        matches!(
            self,
            Self::BuffAdd
                | Self::BuffDel
                | Self::BuffUpdate
                | Self::BuffEffect
                | Self::BuffAddNoEffect
                | Self::BuffDelNoEffect
                | Self::BuffReplace
                | Self::BuffReject
                | Self::BuffAttr
                | Self::BuffRateUp
                | Self::BuffAddAct
        )
    }

    pub fn is_card_related(&self) -> bool {
        matches!(
            self,
            Self::AddHandCard
                | Self::CardsPush
                | Self::UseCards
                | Self::AddCard
                | Self::CardRemove
                | Self::CardRemove2
                | Self::ReDealCard
                | Self::CardInvalid
                | Self::CardDisappear
                | Self::CardsCompose
                | Self::CardLevelAdd
                | Self::CardLevelChange
                | Self::UniversalCard
                | Self::DealCard1
                | Self::DealCard2
                | Self::AddUseCard
                | Self::MasterAddHandCard
                | Self::MasterCardRemove
                | Self::SpCardAdd
                | Self::AddOnceCard
                | Self::CardAConvertCardB
                | Self::CardEffectChange
                | Self::CardDeckGenerate
                | Self::CardDeckDelete
                | Self::CardDeckClear
                | Self::RemoveEntityCards
        )
    }

    pub fn is_control_effect(&self) -> bool {
        matches!(
            self,
            Self::Dizzy
                | Self::Sleep
                | Self::Frozen
                | Self::Petrified
                | Self::Silence
                | Self::Seal
                | Self::Disarm
                | Self::Forbid
                | Self::Charm
                | Self::Confusion
                | Self::Taunt
                | Self::MockTaunt
                | Self::Freeze
                | Self::Rigid
                | Self::Cold
                | Self::Palsy
        )
    }

    pub fn is_resistance(&self) -> bool {
        matches!(
            self,
            Self::DizzyResist
                | Self::SleepResist
                | Self::FrozenResist
                | Self::PetrifiedResist
                | Self::Immunity
                | Self::Resistances
                | Self::ResistancesAttr
        )
    }

    pub fn is_ex_point_related(&self) -> bool {
        matches!(
            self,
            Self::AddExPoint
                | Self::ExPointAdd
                | Self::ExPointDel
                | Self::ExPointChange
                | Self::ExPointFix
                | Self::ExPointCantAdd
                | Self::ExPointMaxAdd
                | Self::ExPointOverflowBank
                | Self::ExPointCardMove
                | Self::ExPointCardUpgrade
                | Self::ExSkillPointChange
                | Self::CantGetExSkill
                | Self::TransferAddExPoint
                | Self::SpExPointMaxAdd
                | Self::ExPointAddAfterDelOrAbsorbExPoint
                | Self::ImmunityExPointChange
                | Self::UseCardFixExPoint
                | Self::DirectUseExSkill
        )
    }

    pub fn is_power_related(&self) -> bool {
        matches!(
            self,
            Self::PowerChange
                | Self::CardDeckNum
                | Self::MasterPowerChange
                | Self::RegainPower
                | Self::PowerCantDecr
                | Self::OverflowPowerAddBuff
                | Self::PowerInfoChange
                | Self::TeamEnergyChange
                | Self::AllocateCardEnergy
                | Self::EmitterEnergyChange
                | Self::FixAttrTeamEnergy
                | Self::FixAttrTeamEnergyAndBuff
                | Self::ChangeCardEnergy
        )
    }

    pub fn is_summon_related(&self) -> bool {
        matches!(
            self,
            Self::Summon
                | Self::SummonedAdd
                | Self::SummonedDelete
                | Self::SummonedLevelUp
                | Self::EmitterCreate
                | Self::EmitterRemove
        )
    }

    pub fn is_round_control(&self) -> bool {
        matches!(
            self,
            Self::RoundEnd
                | Self::ChangeRound
                | Self::SmallRoundEnd
                | Self::ChangeWave
                | Self::NewChangeWave
                | Self::RoundOffset
        )
    }

    pub fn is_client_only(&self) -> bool {
        matches!(
            self,
            Self::ClientEffect
                | Self::IndicatorChange
                | Self::EntitySync
                | Self::LayerHaloSync
                | Self::FightTaskUpdate
        )
    }
}

impl From<i32> for EffectType {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<EffectType> for i32 {
    fn from(effect: EffectType) -> Self {
        effect.to_i32()
    }
}
