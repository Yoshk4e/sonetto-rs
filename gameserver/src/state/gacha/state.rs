use super::helpers::*;
use super::{GachaPool, GachaResult};
use rand::{Rng, seq::SliceRandom};

#[derive(Debug, Clone, Copy)]
pub enum BannerType {
    Yearning,
    RateUp,
    Ripple,
    Standard,
}

impl BannerType {
    pub fn from(t: i32) -> Self {
        match t {
            202 => BannerType::Yearning,
            12 => BannerType::Ripple,
            2 => BannerType::Standard,
            _ => BannerType::RateUp,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GachaState {
    pub pity_6: u32,
    pub up_guaranteed: bool,
}

impl GachaState {
    pub fn single_pull(
        &mut self,
        banner_type: BannerType,
        pool: &GachaPool,
        rng: &mut impl Rng,
        force_five_star: bool,
    ) -> GachaResult {
        self.pity_6 += 1;

        let six_rate = six_star_probability(self.pity_6);
        let roll: f64 = rng.r#gen();

        if roll < six_rate {
            self.pity_6 = 0;

            let (hero_id, is_up) = match banner_type {
                BannerType::RateUp | BannerType::Standard => {
                    let has_up = !pool.six_up.is_empty();

                    let is_up = if has_up {
                        if self.up_guaranteed {
                            self.up_guaranteed = false;
                            true
                        } else {
                            let hit = rng.gen_bool(0.5);
                            if !hit {
                                self.up_guaranteed = true;
                            }
                            hit
                        }
                    } else {
                        self.up_guaranteed = false;
                        false
                    };

                    let hero_id = if is_up {
                        *pool.six_up.choose(rng).unwrap()
                    } else {
                        *pool.six_normal.choose(rng).expect("six_normal empty")
                    };

                    (hero_id, is_up)
                }

                BannerType::Ripple => {
                    let has_up = !pool.six_up.is_empty();

                    let is_up = if has_up {
                        if self.up_guaranteed {
                            self.up_guaranteed = false;
                            true
                        } else {
                            let hit = rng.gen_bool(0.5);
                            if !hit {
                                self.up_guaranteed = true;
                            }
                            hit
                        }
                    } else {
                        self.up_guaranteed = false;
                        false
                    };

                    let hero_id = if is_up {
                        *pool
                            .six_up
                            .choose(rng)
                            .expect("Ripple requires at least one UP hero")
                    } else {
                        *pool.six_normal.choose(rng).expect("six_normal empty")
                    };

                    (hero_id, is_up)
                }

                BannerType::Yearning => {
                    let has_up = !pool.six_up.is_empty();

                    let is_up = if has_up {
                        if self.up_guaranteed {
                            self.up_guaranteed = false;
                            true
                        } else {
                            let hit = rng.gen_bool(0.7);
                            if !hit {
                                self.up_guaranteed = true;
                            }
                            hit
                        }
                    } else {
                        self.up_guaranteed = false;
                        false
                    };

                    let hero_id = if is_up {
                        if !pool.six_up_weighted.is_empty() {
                            choose_weighted(rng, &pool.six_up_weighted)
                        } else {
                            *pool.six_up.choose(rng).unwrap()
                        }
                    } else {
                        *pool.six_normal.choose(rng).expect("six_normal empty")
                    };

                    (hero_id, is_up)
                }
            };

            return GachaResult::Hero {
                hero_id,
                rare: 6,
                is_up,
            };
        }

        let remaining = 1.0 - six_rate;

        let mut rarity_weights = [(5u8, RATE_5), (4u8, RATE_4), (3u8, RATE_3), (2u8, RATE_2)]
            .map(|(r, w)| (r, w / remaining))
            .to_vec();

        if force_five_star {
            rarity_weights.retain(|(r, _)| *r >= 5);
        }

        let rare = pick_weighted(&rarity_weights, rng);

        let hero_id = match rare {
            5 => {
                if !pool.five_up.is_empty() && rng.gen_bool(0.5) {
                    *pool.five_up.choose(rng).unwrap()
                } else {
                    *pool.five_normal.choose(rng).unwrap()
                }
            }
            4 => *pool.four.choose(rng).unwrap(),
            3 => *pool.three.choose(rng).unwrap(),
            2 => *pool.two.choose(rng).unwrap(),
            _ => unreachable!(),
        };

        GachaResult::Hero {
            hero_id,
            rare,
            is_up: false,
        }
    }

    pub fn ten_pull(
        &mut self,
        banner_type: BannerType,
        pool: &GachaPool,
        rng: &mut impl Rng,
    ) -> Vec<GachaResult> {
        let mut results = Vec::with_capacity(10);

        results.push(self.single_pull(banner_type, pool, rng, true));

        for _ in 1..10 {
            results.push(self.single_pull(banner_type, pool, rng, false));
        }

        results
    }
}

#[derive(Debug)]
pub struct UserGachaState {
    pub pity_6: u32,
    pub up_guaranteed: bool,
}

pub async fn load_gacha_state(
    pool: &sqlx::SqlitePool,
    user_id: i64,
    pool_id: i32,
) -> sqlx::Result<UserGachaState> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"
        SELECT pity_6, up_guaranteed
        FROM user_gacha_state
        WHERE user_id = ? AND pool_id = ?
        "#,
    )
    .bind(user_id)
    .bind(pool_id)
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some((pity, up)) => UserGachaState {
            pity_6: pity as u32,
            up_guaranteed: up != 0,
        },
        None => UserGachaState {
            pity_6: 0,
            up_guaranteed: false,
        },
    })
}

pub async fn save_gacha_state(
    pool: &sqlx::SqlitePool,
    user_id: i64,
    pool_id: i32,
    gacha: &GachaState,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_gacha_state (user_id, pool_id, pity_6, up_guaranteed)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(user_id, pool_id)
        DO UPDATE SET
            pity_6 = excluded.pity_6,
            up_guaranteed = excluded.up_guaranteed
        "#,
    )
    .bind(user_id)
    .bind(pool_id)
    .bind(gacha.pity_6 as i32)
    .bind(gacha.up_guaranteed)
    .execute(pool)
    .await?;

    Ok(())
}
