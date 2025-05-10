use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HairiResult {
    pub now: i8,
    pub wait: Vec<i8>,
    pub waits_after_discard: Vec<(i8, Vec<i8>)>,
}

#[derive(Serialize, Deserialize)]
pub struct RiichiOptions {
    pub dora: Vec<i8>,
    pub aka_count: i8,
    pub first_take: bool, // tenhou/chihou/renhou
    pub riichi: bool,
    pub ippatsu: bool,
    pub double_riichi: bool,
    pub last_tile: bool,               // haitei/houtei
    pub after_kan: bool,               // chankan/rinshan
    pub tile_discarded_by_someone: i8, // -1 if tsumo
    pub bakaze: i8,
    pub jikaze: i8,
    pub allow_aka: bool,
    pub allow_kuitan: bool,
    pub with_kiriage: bool,
    pub disabled_yaku: Vec<i8>,
    pub local_yaku_enabled: Vec<i8>,
    pub all_local_yaku_enabled: bool,
    pub allow_double_yakuman: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RiichiHand {
    pub closed_part: Vec<i8>,
    pub open_part: Vec<(bool, Vec<i8>)>, // (isOpenMeld, tiles)
}

#[derive(Serialize, Deserialize)]
pub struct RiichiResult {
    pub is_agari: bool,
    pub yakuman: i8,
    pub han: i32,
    pub fu: i32,
    pub ten: i32,                         // points amount
    pub outgoing_ten: Option<(i32, i32)>, // (oya, ko) points or nothing
    pub yaku: Vec<(i8, i8)>,              // (yaku_id, han_count)
    pub hairi: Option<HairiResult>,
}
