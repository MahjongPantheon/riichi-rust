use crate::agari::{check, check7};
use crate::constants::{Suit, Tiles, is19, slice_by_suit, sum};

pub struct YakuCheckInput<'t> {
    pub(crate) haipai: &'t Vec<i32>,
    pub(crate) haipai34: &'t Vec<i32>,
    pub(crate) furo: &'t Vec<Vec<i32>>,
    pub(crate) current_pattern: &'t Vec<Vec<i32>>,
    pub(crate) taken_tile: i32,
    pub(crate) is_tsumo: bool,
    pub(crate) jikaze: i32,
    pub(crate) bakaze: i32,
    pub(crate) first_take: bool,
    pub(crate) riichi: bool,
    pub(crate) double_riichi: bool,
    pub(crate) ippatsu: bool,
    pub(crate) after_kan: bool,
    pub(crate) last_tile: bool,
    pub(crate) allow_kuitan: bool,
}

pub struct YakuCheck {
    pub(crate) is_local: bool,
    pub(crate) yakuman: i32,
    pub(crate) han: i32,
    pub(crate) is_menzen_only: bool,
    pub(crate) is_furo_minus: bool,
    pub(crate) check: fn(&YakuCheckInput) -> bool,
}

static GREENS: [i32; 6] = [
    Tiles::S2 as i32,
    Tiles::S3 as i32,
    Tiles::S4 as i32,
    Tiles::S6 as i32,
    Tiles::S8 as i32,
    Tiles::GD as i32,
];

static WINDS: [i32; 4] = [
    Tiles::E as i32,
    Tiles::S as i32,
    Tiles::W as i32,
    Tiles::N as i32,
];

static HONORS: [i32; 7] = [
    Tiles::E as i32,
    Tiles::S as i32,
    Tiles::W as i32,
    Tiles::N as i32,
    Tiles::GD as i32,
    Tiles::RD as i32,
    Tiles::WD as i32,
];

static TERMINALS: [i32; 6] = [
    Tiles::M1 as i32,
    Tiles::M9 as i32,
    Tiles::S1 as i32,
    Tiles::S9 as i32,
    Tiles::P1 as i32,
    Tiles::P9 as i32,
];

static TERMINALS_AND_HONORS: [i32; 13] = [
    Tiles::M1 as i32,
    Tiles::M9 as i32,
    Tiles::S1 as i32,
    Tiles::S9 as i32,
    Tiles::P1 as i32,
    Tiles::P9 as i32,
    Tiles::E as i32,
    Tiles::S as i32,
    Tiles::W as i32,
    Tiles::N as i32,
    Tiles::GD as i32,
    Tiles::RD as i32,
    Tiles::WD as i32,
];

static CHI_START: [i32; 9] = [
    Tiles::M1 as i32,
    Tiles::M4 as i32,
    Tiles::M7 as i32,
    Tiles::P1 as i32,
    Tiles::P4 as i32,
    Tiles::P7 as i32,
    Tiles::S1 as i32,
    Tiles::S4 as i32,
    Tiles::S7 as i32,
];

static SIMPLE_TILES: [i32; 21] = [
    Tiles::M2 as i32,
    Tiles::M3 as i32,
    Tiles::M4 as i32,
    Tiles::M5 as i32,
    Tiles::M6 as i32,
    Tiles::M7 as i32,
    Tiles::M8 as i32,
    Tiles::P2 as i32,
    Tiles::P3 as i32,
    Tiles::P4 as i32,
    Tiles::P5 as i32,
    Tiles::P6 as i32,
    Tiles::P7 as i32,
    Tiles::P8 as i32,
    Tiles::S2 as i32,
    Tiles::S3 as i32,
    Tiles::S4 as i32,
    Tiles::S5 as i32,
    Tiles::S6 as i32,
    Tiles::S7 as i32,
    Tiles::S8 as i32,
];

fn check_allowed(haipai: &Vec<i32>, furo: &Vec<Vec<i32>>, allowed: &Vec<i32>) -> bool {
    for v in haipai {
        if !allowed.contains(v) {
            return false;
        }
    }

    for v in furo {
        for vv in v {
            if !allowed.contains(&vv.abs()) {
                return false;
            }
        }
    }

    true
}

fn to_hand(pattern: &Vec<Vec<i32>>) -> Vec<i32> {
    let occurrences = pattern.iter().flatten().collect::<Vec<&i32>>();
    let mut hand = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0,
    ];
    for tile in occurrences {
        hand[(tile.abs() - 1) as usize] += 1;
    }
    hand
}

// Get same suit index or -1 if there are more than one suit in hand or hand consists of HONORS
fn get_same_suit(current_pattern: &Vec<Vec<i32>>, exclude_honors: bool) -> i32 {
    let mut found_suit: i32 = -1;
    let slices = slice_by_suit(&to_hand(current_pattern));
    for i in 0..4 {
        if sum(&slices[i]) == 0 {
            continue;
        }
        if exclude_honors && i == Suit::Honor as usize {
            continue;
        }
        if found_suit != -1 {
            // more than one suit in hand
            return -1;
        }
        found_suit = i as i32;
    }

    if found_suit == 3 { -1 } else { found_suit }
}

// Simple 64-bit digest for positive numbers not more than 7-bit length (<63)
fn digest_simple(input: &Vec<i32>) -> i64 {
    let mut output: i64 = 0;
    for i in 0..input.len() {
        output += (input[i] as i64) << 7 * i;
    }
    output
}

fn check_chanta_like(current_pattern: &Vec<Vec<i32>>, allow: &Vec<i32>) -> bool {
    let mut has_jyuntsu = false;
    for v in current_pattern {
        if v.len() <= 2 || v[0] == v[1] {
            if !allow.contains(&v[0].abs()) {
                return false;
            }
        } else {
            if v[0] == v[1] && v[0] == v[2] {
                if !allow.contains(&v[0].abs()) {
                    return false;
                }
            } else {
                has_jyuntsu = true;
                if !is19(v[0]) && !is19(v[2]) {
                    return false;
                }
            }
        }
    }

    has_jyuntsu
}

fn check_yakuhai(current_pattern: &Vec<Vec<i32>>, jikaze: i32, bakaze: i32, which: i32) -> bool {
    for v in current_pattern {
        if v[0].abs() == which && [jikaze, bakaze, 32, 33, 34].contains(&v[0].abs()) && v.len() >= 3
        {
            return true;
        }
    }
    false
}

// Certain yaku checkers

fn yaku_check_kokushimusou_13_sides(i: &YakuCheckInput) -> bool {
    check(i.haipai34)
        && i.haipai.iter().fold(
            0,
            |total, v| {
                if *v == i.taken_tile { total + 1 } else { total }
            },
        ) == 2
}

fn yaku_check_kokusimusou(i: &YakuCheckInput) -> bool {
    check(i.haipai34)
        && i.haipai.iter().fold(
            0,
            |total, v| {
                if *v == i.taken_tile { total + 1 } else { total }
            },
        ) == 1
}

fn yaku_check_chuurenpoto_9_sides(i: &YakuCheckInput) -> bool {
    let slices = slice_by_suit(i.haipai34);
    let found_suit = get_same_suit(i.current_pattern, false);
    if found_suit == -1
        || slices[found_suit as usize][0] < 3
        || slices[found_suit as usize][8] < 3
        || slices[found_suit as usize].contains(&0)
    {
        return false;
    }
    i.taken_tile != -1 && [2, 4].contains(&slices[found_suit as usize][(i.taken_tile % 9) as usize])
}

fn yaku_check_chuurenpoto(i: &YakuCheckInput) -> bool {
    let slices = slice_by_suit(i.haipai34);
    let found_suit = get_same_suit(i.current_pattern, false);
    if found_suit == -1
        || slices[found_suit as usize][0] < 3
        || slices[found_suit as usize][8] < 3
        || slices[found_suit as usize].contains(&0)
    {
        return false;
    }
    i.taken_tile != -1 && [1, 3].contains(&slices[found_suit as usize][(i.taken_tile % 9) as usize])
}

fn yaku_check_suuankou_tanki(i: &YakuCheckInput) -> bool {
    if i.furo.len() > 0 {
        if i.furo.iter().any(|v| v[0] > 0) {
            // open sets are not allowed
            return false;
        }
    }

    let mut kotsu = 0;
    for v in i.current_pattern {
        if v.len() >= 3 && v[0] == v[1] {
            kotsu += 1;
        } else {
            if v[0] != i.taken_tile {
                return false;
            }
        }
    }

    kotsu == 4
}

fn yaku_check_suuankou(i: &YakuCheckInput) -> bool {
    if i.furo.len() > 0 {
        if i.furo.iter().any(|v| v[0] > 0) {
            // open sets are not allowed
            return false;
        }
    }

    let mut kotsu = 0;
    for v in i.current_pattern {
        if v.len() >= 3 && v[0] == v[1] {
            kotsu += 1;
        }
    }

    kotsu == 4 && i.is_tsumo && !yaku_check_suuankou_tanki(i)
}

fn yaku_check_daisuushi(i: &YakuCheckInput) -> bool {
    let mut res = 0;
    for v in i.current_pattern {
        if WINDS.contains(&v[0].abs()) && v.len() >= 3 {
            res += 1;
        }
    }
    res == 4
}

fn yaku_check_shosuushi(i: &YakuCheckInput) -> bool {
    let mut kotsu = 0;
    let mut toitsu = 0;
    for v in i.current_pattern {
        if WINDS.contains(&v[0].abs()) && v.len() >= 3 {
            kotsu += 1;
        }
        if WINDS.contains(&v[0]) && v.len() == 2 {
            toitsu += 1;
        }
    }
    kotsu == 3 && toitsu == 1
}

fn yaku_check_daisangen(i: &YakuCheckInput) -> bool {
    let need = [Tiles::WD as i32, Tiles::GD as i32, Tiles::RD as i32];
    let mut res = 0;
    for v in i.current_pattern {
        if need.contains(&v[0].abs()) && v.len() >= 3 {
            res += 1;
        }
    }
    res == 3
}

fn yaku_check_tsuuiisou(i: &YakuCheckInput) -> bool {
    check_allowed(i.haipai, i.furo, &Vec::from(HONORS))
}

fn yaku_check_ryuuiisou(i: &YakuCheckInput) -> bool {
    check_allowed(i.haipai, i.furo, &Vec::from(GREENS))
}

fn yaku_check_chinroutou(i: &YakuCheckInput) -> bool {
    check_allowed(i.haipai, i.furo, &Vec::from(TERMINALS))
}

fn yaku_check_suukantsu(i: &YakuCheckInput) -> bool {
    let mut kantsu = 0;
    for v in i.current_pattern {
        if v.len() == 4 {
            kantsu += 1;
        }
    }
    kantsu == 4
}

fn yaku_check_tenhou(i: &YakuCheckInput) -> bool {
    i.first_take && i.is_tsumo && i.jikaze == Tiles::E as i32 && i.furo.len() == 0
}

fn yaku_check_chihou(i: &YakuCheckInput) -> bool {
    i.first_take && i.is_tsumo && i.jikaze != Tiles::E as i32 && i.furo.len() == 0
}

fn yaku_check_renhou(i: &YakuCheckInput) -> bool {
    i.first_take && !i.is_tsumo && i.furo.len() == 0
}

fn yaku_check_daisharin(i: &YakuCheckInput) -> bool {
    yaku_check_tsuuiisou(i) && yaku_check_chiitoitsu(i)
}

fn yaku_check_chinitsu(i: &YakuCheckInput) -> bool {
    get_same_suit(i.current_pattern, false) != -1
}

fn yaku_check_honitsu(i: &YakuCheckInput) -> bool {
    get_same_suit(i.current_pattern, true) != -1 && !yaku_check_chinitsu(i)
}

fn yaku_check_ryanpeikou(i: &YakuCheckInput) -> bool {
    if i.furo.len() > 0 {
        return false;
    }

    let mut digests: Vec<i64> = Vec::new();
    for v in i.current_pattern {
        if v.len() >= 3 && v[0] == v[1] {
            return false;
        }
        if v.len() == 3 {
            digests.push(digest_simple(v));
        }
    }

    // Imply that every chi occurs twice, so double XORing them over each other gives 0
    digests.len() == 4 && digests.iter().fold(0, |acc, v| acc ^ v) == 0
}

fn yaku_check_junchan(i: &YakuCheckInput) -> bool {
    check_chanta_like(i.current_pattern, &Vec::from(TERMINALS))
}

fn yaku_check_chanta(i: &YakuCheckInput) -> bool {
    check_chanta_like(i.current_pattern, &Vec::from(TERMINALS_AND_HONORS)) && !yaku_check_junchan(i)
}

fn yaku_check_toitoi(i: &YakuCheckInput) -> bool {
    let mut kotsu = 0;
    for v in i.current_pattern {
        if v.len() >= 3 && v[0] == v[1] {
            kotsu += 1;
        }
    }
    kotsu == 4
}

fn yaku_check_honroutou(i: &YakuCheckInput) -> bool {
    check_allowed(i.haipai, i.furo, &Vec::from(TERMINALS_AND_HONORS))
}

fn yaku_check_sankantsu(i: &YakuCheckInput) -> bool {
    let mut kantsu = 0;
    for v in i.current_pattern {
        if v.len() == 4 {
            kantsu += 1;
        }
    }
    kantsu == 3
}

fn yaku_check_shosangen(i: &YakuCheckInput) -> bool {
    let need = vec![Tiles::WD as i32, Tiles::GD as i32, Tiles::RD as i32];
    let mut kotsu_or_toitsu = 0;
    for v in i.current_pattern {
        if need.contains(&v[0].abs()) && v[0] == v[1] {
            kotsu_or_toitsu += 1;
        }
    }
    kotsu_or_toitsu == 3 && !yaku_check_daisangen(i) && !yaku_check_chiitoitsu(i)
}

fn yaku_check_sanshoku_doukou(i: &YakuCheckInput) -> bool {
    let mut res = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    for v in i.current_pattern {
        let abs = v[0].abs();
        if v.len() >= 3 && v[0] == v[1] && !HONORS.contains(&abs) {
            res[((abs - 1) % 9) as usize] += 1;
        }
    }
    res.contains(&3)
}

fn yaku_check_sanankou(i: &YakuCheckInput) -> bool {
    let mut kotsu = 0;

    // keep here all tiles that are not closed kantsu
    let mut open_kotsu: Vec<i32> = Vec::new();
    for v in i.furo {
        if v[0] > 0 && v[0] == v[1] {
            open_kotsu.push(v[0]);
        }
    }

    let closed_part = i
        .current_pattern
        .iter()
        .filter(|set| {
            i.furo
                .iter()
                .find(|meld| digest_simple(set) == digest_simple(meld))
                == None
        })
        .collect::<Vec<&Vec<i32>>>();

    for v in i.current_pattern {
        if open_kotsu.contains(&v[0])
            || (v.len() >= 3
                && !i.is_tsumo
                && i.taken_tile != -1
                && v[0] == v[1]
                && v.contains(&i.taken_tile)
                && closed_part.iter().find(|set| {
                    i.taken_tile == v[0] && set[0] != set[1] && set.contains(&&i.taken_tile)
                }) == None)
        {
            continue;
        }
        if v.len() >= 3 && v[0] == v[1] {
            kotsu += 1;
        }
    }

    kotsu == 3
}

fn yaku_check_chiitoitsu(i: &YakuCheckInput) -> bool {
    check7(i.haipai34) && !yaku_check_ryanpeikou(i)
}

fn yaku_check_daburu_riichi(i: &YakuCheckInput) -> bool {
    i.double_riichi && i.furo.len() == 0
}

fn yaku_check_ittsu(i: &YakuCheckInput) -> bool {
    let mut res: Vec<Vec<i32>> = vec![
        vec![0, 0, 0], // man shuntsu
        vec![0, 0, 0], // pin shuntsu
        vec![0, 0, 0], // sou shuntsu
    ];

    for v in i.current_pattern {
        if v[0] == v[1] {
            continue;
        }
        if CHI_START.contains(&v[0]) {
            res[((v[0] - 1) as f32 / 9.).floor() as usize]
                [(((v[1] - 1) % 9) as f32 / 3.).round() as usize] += 1;
        }
    }

    res.iter().any(|set| set.iter().all(|t| *t >= 1))
}

fn yaku_check_sanshoku(i: &YakuCheckInput) -> bool {
    let mut res: Vec<Vec<i32>> = vec![
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    for v in i.current_pattern {
        if v[0] == v[1] {
            continue;
        }
        // collect shuntsu count starting at current tile
        res[((v[0] - 1) as f32 / 9.).floor() as usize][((v[1] - 1) % 9) as usize] += 1;
    }

    for i in 0..9 {
        if res[0][i] > 0 && res[1][i] > 0 && res[2][i] > 0 {
            return true;
        }
    }

    false
}

fn yaku_check_tanyao(i: &YakuCheckInput) -> bool {
    if i.furo.iter().any(|s| s[0] > 0) && !i.allow_kuitan {
        return false;
    }
    check_allowed(i.haipai, i.furo, &Vec::from(SIMPLE_TILES))
}

fn yaku_check_pinfu(i: &YakuCheckInput) -> bool {
    if i.furo.len() > 0 {
        return false;
    }

    let mut fu = 0;
    let mut has_pair_fu = false;

    for v in i.current_pattern {
        if v.len() == 4 {
            fu += if is19(v[0].abs()) {
                if v[0] > 0 { 16 } else { 32 }
            } else {
                if v[0] > 0 { 8 } else { 16 }
            };
        } else if v.len() == 2 {
            if [
                i.bakaze,
                i.jikaze,
                Tiles::WD as i32,
                Tiles::RD as i32,
                Tiles::GD as i32,
            ]
            .contains(&v[0])
            {
                // pair of yakuhai tile
                fu += 2;
            }
            if v[0] == i.taken_tile {
                has_pair_fu = true;
            }
        } else if v.len() == 3 {
            if v[0] == v[1] {
                fu += if is19(v[0]) { 4 } else { 2 };
            }
        }
    }

    // check kanchan/penchan
    let mut can_be_ryanmen = false;
    let mut can_be_shanpon = false;
    let mut can_be_tanki = false;

    if i.taken_tile != -1 {
        for v in i.current_pattern {
            if v.len() != 3 {
                if v.len() == 2 && v[0] == i.taken_tile {
                    // tanki waits are already handled above
                    can_be_tanki = true;
                }
                continue;
            }

            if v[0] == v[1] && v[0] == i.taken_tile {
                can_be_shanpon = true;
            }

            if (v[0] == i.taken_tile && !is19(v[2])) || (v[2] == i.taken_tile && !is19(v[0])) {
                can_be_ryanmen = true;
            }
        }

        if !can_be_shanpon && !can_be_ryanmen && !can_be_tanki {
            fu += 2;
        }
    }

    if has_pair_fu && !can_be_ryanmen {
        fu += 2;
    }

    fu == 0
}

fn yaku_check_iipeikou(i: &YakuCheckInput) -> bool {
    if yaku_check_ryanpeikou(i) {
        return false;
    }

    for idx in 0..i.current_pattern.len() {
        let mut idxc = idx;
        let v = &i.current_pattern[idxc];
        if v.len() == 3 && v[0] != v[1] {
            while idxc < 4 {
                idxc += 1;
                if v.eq(&i.current_pattern[idxc]) {
                    return true;
                }
            }
        }
    }

    false
}

fn yaku_check_menzentsumo(i: &YakuCheckInput) -> bool {
    i.is_tsumo
}

fn yaku_check_riichi(i: &YakuCheckInput) -> bool {
    i.riichi && !yaku_check_daburu_riichi(i)
}

fn yaku_check_ippatsu(i: &YakuCheckInput) -> bool {
    i.ippatsu
}

fn yaku_check_rinshan(i: &YakuCheckInput) -> bool {
    let mut has_kantsu = false;
    for v in i.furo {
        if v.len() == 4 {
            has_kantsu = true;
            break;
        }
    }
    has_kantsu && i.after_kan && !i.last_tile && i.is_tsumo
}

fn yaku_check_chankan(i: &YakuCheckInput) -> bool {
    i.after_kan && !i.last_tile && !i.is_tsumo
}

fn yaku_check_haitei(i: &YakuCheckInput) -> bool {
    i.last_tile && i.is_tsumo
}

fn yaku_check_houtei(i: &YakuCheckInput) -> bool {
    i.last_tile && !i.is_tsumo
}

fn yaku_check_round_wind_east(i: &YakuCheckInput) -> bool {
    i.bakaze == Tiles::E as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::E as i32)
}

fn yaku_check_round_wind_south(i: &YakuCheckInput) -> bool {
    i.bakaze == Tiles::S as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::S as i32)
}

fn yaku_check_round_wind_west(i: &YakuCheckInput) -> bool {
    i.bakaze == Tiles::W as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::W as i32)
}

fn yaku_check_round_wind_north(i: &YakuCheckInput) -> bool {
    i.bakaze == Tiles::N as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::N as i32)
}

fn yaku_check_own_wind_east(i: &YakuCheckInput) -> bool {
    i.jikaze == Tiles::E as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::E as i32)
}

fn yaku_check_own_wind_south(i: &YakuCheckInput) -> bool {
    i.jikaze == Tiles::S as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::S as i32)
}

fn yaku_check_own_wind_west(i: &YakuCheckInput) -> bool {
    i.jikaze == Tiles::W as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::W as i32)
}

fn yaku_check_own_wind_north(i: &YakuCheckInput) -> bool {
    i.jikaze == Tiles::N as i32
        && check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::N as i32)
}

fn yaku_check_haku(i: &YakuCheckInput) -> bool {
    check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::WD as i32)
}

fn yaku_check_hatsu(i: &YakuCheckInput) -> bool {
    check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::GD as i32)
}

fn yaku_check_chun(i: &YakuCheckInput) -> bool {
    check_yakuhai(i.current_pattern, i.jikaze, i.bakaze, Tiles::RD as i32)
}

// Yaku settings aggregate

// Note: order MUST be the same as in Yaku enum
// to be able to reference the settings by enum
pub static YAKU_SETTINGS: [YakuCheck; 53] = [
    // 'kokushimusou 13 sides':
    YakuCheck {
        is_local: false,
        yakuman: 2,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_kokushimusou_13_sides,
    },
    // kokushimusou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_kokusimusou,
    },
    // 'chuurenpoto 9 sides':
    YakuCheck {
        is_local: false,
        yakuman: 2,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_chuurenpoto_9_sides,
    },
    // chuurenpoto:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_chuurenpoto,
    },
    // 'suuankou tanki':
    YakuCheck {
        is_local: false,
        yakuman: 2,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_suuankou_tanki,
    },
    // suuankou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_suuankou,
    },
    // daisuushi:
    YakuCheck {
        is_local: false,
        yakuman: 2,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_daisuushi,
    },
    // shosuushi:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_shosuushi,
    },
    // daisangen:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_daisangen,
    },
    // tsuuiisou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_tsuuiisou,
    },
    // ryuuiisou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_ryuuiisou,
    },
    // chinroutou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_chinroutou,
    },
    // suukantsu:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: false,
        is_furo_minus: false,
        check: yaku_check_suukantsu,
    },
    // tenhou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_tenhou,
    },
    // chihou:
    YakuCheck {
        is_local: false,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_chihou,
    },
    // renhou:
    YakuCheck {
        is_local: true,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_renhou,
    },
    // daisharin:
    YakuCheck {
        is_local: true,
        yakuman: 1,
        han: 0,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_daisharin,
    },
    // chinitsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 6,
        is_furo_minus: true,
        check: yaku_check_chinitsu,
    },
    // honitsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 3,
        is_furo_minus: true,
        check: yaku_check_honitsu,
    },
    // ryanpeikou:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 3,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_ryanpeikou,
    },
    // junchan:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 3,
        is_furo_minus: true,
        check: yaku_check_junchan,
    },
    // chanta:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: true,
        check: yaku_check_chanta,
    },
    // toitoi:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: false,
        check: yaku_check_toitoi,
    },
    // honroutou:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: false,
        check: yaku_check_honroutou,
    },
    // sankantsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: false,
        check: yaku_check_sankantsu,
    },
    // shosangen:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: false,
        check: yaku_check_shosangen,
    },
    // 'sanshoku doukou':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: false,
        check: yaku_check_sanshoku_doukou,
    },
    // sanankou:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: false,
        check: yaku_check_sanankou,
    },
    // chiitoitsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 2,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_chiitoitsu,
    },
    // 'daburu riichi':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 2,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_daburu_riichi,
    },
    // ittsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: true,
        check: yaku_check_ittsu,
    },
    // sanshoku:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 2,
        is_furo_minus: true,
        check: yaku_check_sanshoku,
    },
    // tanyao:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_tanyao,
    },
    // pinfu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 1,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_pinfu,
    },
    // iipeikou:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 1,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_iipeikou,
    },
    // menzentsumo:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 1,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_menzentsumo,
    },
    // riichi:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 1,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_riichi,
    },
    // ippatsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        han: 1,
        is_menzen_only: true,
        is_furo_minus: false,
        check: yaku_check_ippatsu,
    },
    // rinshan:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_rinshan,
    },
    // chankan:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_chankan,
    },
    // haitei:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_haitei,
    },
    // houtei:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_houtei,
    },
    // 'round wind east':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_round_wind_east,
    },
    // 'round wind south':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_round_wind_south,
    },
    // 'round wind west':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_round_wind_west,
    },
    // 'round wind north':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_round_wind_north,
    },
    // 'own wind east':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_own_wind_east,
    },
    // 'own wind south':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_own_wind_south,
    },
    // 'own wind west':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_own_wind_west,
    },
    // 'own wind north':
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_own_wind_north,
    },
    // haku:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_haku,
    },
    // hatsu:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_hatsu,
    },
    // chun:
    YakuCheck {
        is_local: false,
        yakuman: 0,
        is_menzen_only: false,
        han: 1,
        is_furo_minus: false,
        check: yaku_check_chun,
    },
];
