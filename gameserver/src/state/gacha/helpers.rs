use rand::Rng;

const RATE_6_BASE: f64 = 0.015;
pub const RATE_5: f64 = 0.085;
pub const RATE_4: f64 = 0.40;
pub const RATE_3: f64 = 0.45;
pub const RATE_2: f64 = 0.05;

pub fn parse_up_heroes(s: &str) -> (Vec<i32>, Vec<i32>) {
    if s.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut parts = s.split('|');

    let six_up = parts.next().map(parse_id_list).unwrap_or_default();
    let five_up = parts.next().map(parse_id_list).unwrap_or_default();

    (six_up, five_up)
}

pub fn parse_id_list(s: &str) -> Vec<i32> {
    if s.is_empty() {
        return Vec::new();
    }

    s.split('#').filter_map(|x| x.parse::<i32>().ok()).collect()
}

pub fn six_star_probability(pity_6: u32) -> f64 {
    match pity_6 {
        0..=59 => RATE_6_BASE,
        60..=69 => 0.04 + (pity_6 - 60) as f64 * 0.025,
        _ => 1.0,
    }
}

pub fn pick_weighted<T: Copy>(items: &[(T, f64)], rng: &mut impl Rng) -> T {
    let roll: f64 = Rng::r#gen(rng);
    let mut acc = 0.0;

    for (item, weight) in items {
        acc += weight;
        if roll < acc {
            return *item;
        }
    }

    items.last().unwrap().0
}
