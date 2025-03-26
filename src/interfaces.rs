pub struct HairiResult {
    pub now: i32,
    pub wait: Vec<i32>,
    pub waits_after_discard: Vec<(i32, Vec<i32>)>,
}

pub struct RiichiOptions<'t> {
    pub dora: &'t Vec<i32>,
    pub aka_count: i32,
    pub first_take: bool, // tenhou/chihou/renhou
    pub riichi: bool,
    pub ippatsu: bool,
    pub double_riichi: bool,
    pub last_take: bool,                // haitei/houtei
    pub after_kan: bool,                // chankan/rinshan
    pub tile_discarded_by_someone: i32, // -1 if tsumo
    pub bakaze: i32,
    pub jikaze: i32,
    pub allow_aka: bool,
    pub allow_kuitan: bool,
    pub with_kiriage: bool,
    pub disabled_yaku: &'t Vec<i32>,
    pub local_yaku_enabled: &'t Vec<i32>,
    pub all_local_yaku_enabled: bool,
    pub allow_double_yakuman: bool,
    pub taken_tile: i32,
    pub last_tile: bool,
}

pub struct RiichiHand {
    pub closed_part: Vec<i32>,
    pub open_part: Vec<(bool, Vec<i32>)>, // (isOpenMeld, tiles)
}

pub struct RiichiResult {
    pub is_agari: bool,
    pub yakuman: i32,
    pub han: i32,
    pub fu: i32,
    pub ten: i32,                         // points amount
    pub outgoing_ten: Option<(i32, i32)>, // (oya, ko) points or nothing
    pub yaku: Vec<(i32, i32)>,            // (yaku_id, han_count)
    pub hairi: Option<HairiResult>,
    pub hairi7and13: Option<HairiResult>,
}
