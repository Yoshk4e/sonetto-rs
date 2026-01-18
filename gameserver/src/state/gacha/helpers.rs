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

pub fn parse_weighted_id_list(s: &str) -> Vec<(i32, u32)> {
    if s.is_empty() {
        return Vec::new();
    }

    s.split('|')
        .map(|part| {
            let mut it = part.split('#');

            let id: i32 = it
                .next()
                .expect("missing id")
                .parse()
                .expect("bad id");

            let weight: u32 = it
                .next()
                .expect("missing weight")
                .parse()
                .expect("bad weight");

            (id, weight)
        })
        .collect()
}

pub fn parse_dupe_rewards(s: &str) -> (Vec<(u32, i32)>, Vec<(i32, i32)>) {
    if s.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut items = Vec::new();
    let mut currencies = Vec::new();

    for segment in s.split('|') {
        let parts: Vec<i32> = segment
            .split('#')
            .filter_map(|x| x.parse::<i32>().ok())
            .collect();

        if parts.len() == 3 {
            let (reward_type, id, amount) = (parts[0], parts[1], parts[2]);
            match reward_type {
                1 => items.push((id as u32, amount)),
                2 => currencies.push((id, amount)),
                _ => tracing::warn!("Unknown dupe reward type: {}", reward_type),
            }
        }
    }

    (items, currencies)
}

pub fn parse_store_product(
    s: &str,
) -> (
    Vec<(u32, i32)>,
    Vec<(i32, i32)>,
    Vec<(u32, i32)>,
    Vec<(u32, i32)>,
    Vec<(u32, i32)>,
    Vec<(u32, i32)>,
) {
    if s.is_empty() {
        return (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
    }

    let mut items = Vec::new();
    let mut currencies = Vec::new();
    let mut equip = Vec::new();
    let mut heroes = Vec::new();
    let mut power_items = Vec::new();
    let mut insight_selector = Vec::new();

    for segment in s.split('|') {
        let parts: Vec<&str> = segment.split('#').collect();

        if parts.len() == 3 {
            if let (Ok(reward_type), Ok(id), Ok(amount)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
            ) {
                match reward_type {
                    1 => items.push((id as u32, amount)),
                    2 => currencies.push((id, amount)),
                    4 => heroes.push((id as u32, amount)),
                    9 => equip.push((id as u32, amount)),
                    10 => power_items.push((id as u32, amount)),
                    24 => insight_selector.push((id as u32, amount)),
                    _ => tracing::warn!("Unknown store product type: {}", reward_type),
                }
            }
        }
    }

    (
        items,
        currencies,
        equip,
        heroes,
        power_items,
        insight_selector,
    )
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

pub fn choose_weighted(rng: &mut impl rand::Rng, items: &[(i32, u32)]) -> i32 {
    let total: u32 = items.iter().map(|(_, w)| *w).sum();
    let mut roll = rng.gen_range(0..total);

    for (id, weight) in items {
        if roll < *weight {
            return *id;
        }
        roll -= *weight;
    }

    unreachable!("weighted selection failed")
}

pub fn parse_item(effect: &str) -> Option<(Vec<(u32, i32)>, Vec<(i32, i32)>)> {
    if effect.is_empty() {
        return None;
    }

    let mut items = Vec::new();
    let mut currencies = Vec::new();
    let mut valid = true;

    for segment in effect.split('|') {
        let parts: Vec<&str> = segment.split('#').collect();

        if parts.len() == 3 {
            if let (Ok(reward_type), Ok(id), Ok(amount)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
            ) {
                match reward_type {
                    1 => items.push((id as u32, amount)),
                    2 => currencies.push((id, amount)),
                    _ => {
                        valid = false;
                        break;
                    }
                }
            } else {
                valid = false;
                break;
            }
        } else {
            valid = false;
            break;
        }
    }

    if valid && (!items.is_empty() || !currencies.is_empty()) {
        Some((items, currencies))
    } else {
        None
    }
}

pub fn get_rewards(material_id: u32) -> (Vec<(u32, i32)>, Vec<(i32, i32)>) {
    match material_id {
        481002 => (
            vec![
                (1529, 1),
                (1501, 1),
                (1502, 1),
                (1503, 1),
                (1504, 1),
                (1507, 1),
                (1508, 1),
                (1510, 1),
            ],
            vec![],
        ),
        481003 => (
            vec![(115011, 1), (115021, 1), (115031, 1), (115041, 1)],
            vec![],
        ),
        491003 | 481005 => (
            vec![
                (110101, 1),
                (110201, 1),
                (110301, 1),
                (110401, 1),
                (110501, 1),
            ],
            vec![],
        ),
        491004 | 481006 => (
            vec![
                (110102, 1),
                (110202, 1),
                (110302, 1),
                (110402, 1),
                (110502, 1),
                (111001, 1),
                (110602, 1),
                (110702, 1),
                (111012, 1),
                (110802, 1),
                (110902, 1),
            ],
            vec![],
        ),
        491007 | 481007 => (
            vec![
                (110103, 1),
                (110203, 1),
                (110303, 1),
                (110403, 1),
                (110503, 1),
                (111002, 1),
                (110603, 1),
                (110703, 1),
                (111012, 1),
                (110803, 1),
                (110903, 1),
            ],
            vec![],
        ),
        481008 => (
            vec![(115012, 1), (115022, 1), (115032, 1), (115042, 1)],
            vec![],
        ),
        481010 => (
            vec![(120001, 1), (120002, 1), (120003, 1), (120004, 1)],
            vec![],
        ),
        491011 => (
            vec![
                (111004, 1),
                (111005, 1),
                (111006, 1),
                (111007, 1),
                (111008, 1),
            ],
            vec![],
        ),
        491010 => (
            vec![
                (110104, 1),
                (110204, 1),
                (110304, 1),
                (110404, 1),
                (110504, 1),
                (111003, 1),
                (110604, 1),
                (110704, 1),
                (111013, 1),
                (110804, 1),
                (110904, 1),
            ],
            vec![],
        ),
        _ => (vec![], vec![]),
    }
}
