#[derive(Debug)]
pub struct GachaPool {
    pub six_up: Vec<i32>,
    pub six_up_weighted: Vec<(i32, u32)>, // ONLY for Yearning
    pub six_normal: Vec<i32>,

    pub five_up: Vec<i32>,
    pub five_normal: Vec<i32>,

    pub four: Vec<i32>,
    pub three: Vec<i32>,
    pub two: Vec<i32>,
}

#[derive(Debug)]
pub enum GachaResult {
    Hero { hero_id: i32, rare: u8, is_up: bool },
}
