use crate::agari::{check_all, find_all_agari_patterns};
use crate::constants::{Yaku, ceil10, ceil100, is_proper_open_set, is19};
use crate::interfaces::{RiichiHand, RiichiOptions, RiichiResult};
use crate::shanten::hairi;
use crate::yaku::{YAKU_SETTINGS, YakuCheckInput};

pub fn calc_riichi(
    hand: RiichiHand,
    options: &mut RiichiOptions,
    calc_hairi: bool,
) -> Result<RiichiResult, String> {
    // Closed part
    let mut haipai = hand.closed_part.clone();
    // Open tiles
    // Closed kan will be with minus sign in tile values
    let mut furo: Vec<Vec<i8>> = Vec::new();
    // tile34-formatted haipai
    let mut haipai34: Vec<i8> = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0,
    ];

    let is_tsumo: bool;

    if options.tile_discarded_by_someone != -1 {
        haipai.push(options.tile_discarded_by_someone);
        is_tsumo = false;
    } else {
        options.tile_discarded_by_someone = haipai.last().unwrap().clone();
        is_tsumo = true;
    }

    for (open, meld_tiles) in hand.open_part {
        if is_proper_open_set(&meld_tiles) {
            let mut tiles = meld_tiles
                .iter()
                .map(|tile| if open { *tile } else { -1 * tile })
                .collect::<Vec<i8>>();
            tiles.sort();
            furo.push(tiles);
        } else {
            haipai.extend_from_slice(meld_tiles.as_slice());
        }
    }

    if haipai.len() % 3 == 0 {
        return Err("Incorrect number of tiles".parse().unwrap());
    }

    if haipai.len() + furo.len() * 3 > 14 {
        return Err("Incorrect number of tiles".parse().unwrap());
    }

    for hai in &haipai {
        haipai34[*hai as usize - 1] += 1;
    }

    calc_all(
        &haipai,
        &haipai34,
        &furo,
        &options.dora,
        options,
        is_tsumo,
        calc_hairi,
    )
}

fn is_menzen(furo: &Vec<Vec<i8>>) -> bool {
    for meld in furo {
        if meld.len() > 2 {
            // closed kan - negative values
            if meld[0] == meld[1] && meld[0] < 0 {
                continue;
            }
            return false;
        }
    }
    true
}

fn calc_dora(haipai: &Vec<i8>, furo: &Vec<Vec<i8>>, dora: &Vec<i8>) -> i8 {
    let mut dora_count = 0;

    for hai in haipai {
        for d in dora {
            // loop over to detect multiple dora
            if hai == d {
                dora_count += 1;
            }
        }
    }

    for meld in furo {
        for tile in meld {
            for d in dora {
                // loop over to detect multiple dora
                if tile.abs() == *d {
                    dora_count += 1;
                }
            }
        }
    }

    dora_count
}

// Return: (han, dora, akadora)
fn calc_all_dora(
    haipai: &Vec<i8>,
    furo: &Vec<Vec<i8>>,
    dora: &Vec<i8>,
    current_han: i32,
    current_aka: i8,
    aka_enabled: bool,
) -> (i8, i8) {
    if current_han == 0 {
        return (0, 0);
    }

    let dora = calc_dora(haipai, furo, dora);

    (dora, if aka_enabled { current_aka } else { 0 })
}

// Return: (total, oya_points, ko_points)
fn calc_ten(
    jikaze: i8,
    is_tsumo: bool,
    yakuman_count: i8,
    han_count: i32,
    fu_count: i32,
    kiriage: bool,
) -> (i32, i32, i32) {
    let mut base: i32;

    if yakuman_count > 0 {
        base = 8000 * yakuman_count as i32;
    } else {
        if han_count == 0 {
            return (0, 0, 0);
        }
        base = fu_count as i32 * 2_i32.pow(han_count as u32 + 2);
        if base > 2000 {
            if han_count >= 13 {
                base = 8000;
            } else if han_count >= 11 {
                base = 6000;
            } else if han_count >= 8 {
                base = 4000;
            } else if han_count >= 6 {
                base = 3000;
            } else {
                base = 2000;
            }
        } else {
            if kiriage && ((han_count == 4 && fu_count == 30) || (han_count == 3 && fu_count == 60))
            {
                base = 2000;
            }
        }
    }

    if is_tsumo {
        (
            if jikaze == 28 {
                ceil100(base * 2) * 3
            } else {
                ceil100(base * 2) + 2 * ceil100(base)
            },
            ceil100(base * 2),
            ceil100(base),
        )
    } else {
        (
            if jikaze == 28 {
                ceil100(base * 6)
            } else {
                ceil100(base * 4)
            },
            0,
            0,
        )
    }
}

pub fn calc_fu(
    is_tsumo: bool,
    bakaze: i8,
    jikaze: i8,
    found_yaku: Vec<i8>,
    taken_tile: i8, // -1 if nothing taken
    current_pattern: &Vec<Vec<i8>>,
    furo: &Vec<Vec<i8>>,
) -> Option<i32> {
    let mut fu;
    let have_pinfu = found_yaku.contains(&(Yaku::Pinfu as i8));

    if found_yaku.contains(&(Yaku::Chiitoitsu as i8)) {
        fu = 25;
    } else if found_yaku.contains(&(Yaku::Kokushimusou as i8))
        || found_yaku.contains(&(Yaku::Kokushimusou13Sides as i8))
    {
        fu = 0;
    } else if found_yaku.contains(&(Yaku::Pinfu as i8)) {
        fu = if is_tsumo { 20 } else { 30 };
    } else {
        fu = 20;
        if !is_tsumo && is_menzen(furo) {
            fu += 10;
        }

        // check waiting
        let mut can_be_ryanmen = false;
        let mut can_be_kanchan = false;
        let mut can_be_penchan = false;
        let mut can_be_shanpon = false;
        let mut can_be_tanki = false;
        if taken_tile != -1 {
            for form in current_pattern {
                if form.len() != 3 {
                    if form.len() == 2 && taken_tile == form[0] {
                        can_be_tanki = true;
                    }
                    continue;
                }

                if form[0] == form[1] {
                    if form[0] == taken_tile {
                        can_be_shanpon = true;
                    }
                } else {
                    if (form[0] == taken_tile && !is19(form[2]))
                        || (form[2] == taken_tile && !is19(form[0]))
                    {
                        can_be_ryanmen = true;
                    }

                    if (form[0] == taken_tile && is19(form[2]))
                        || (form[2] == taken_tile && is19(form[0]))
                    {
                        can_be_penchan = true;
                    }

                    if form[1] == taken_tile {
                        can_be_kanchan = true;
                    }
                }
            }
        }

        for meld in furo {
            if meld.len() == 4 {
                fu += if is19(meld[0].abs()) {
                    if meld[0] > 0 { 16 } else { 32 }
                } else {
                    if meld[0] > 0 { 8 } else { 16 }
                }
            } else if meld.len() == 3 && meld[0] == meld[1] {
                fu += if is19(meld[0]) { 4 } else { 2 };
            }
        }

        for form in current_pattern {
            if form.len() == 2 {
                if [bakaze, jikaze, 32, 33, 34].contains(&form[0]) {
                    // pair of yakuhai tile
                    fu += 2;
                }
                if bakaze == jikaze && bakaze == form[0] {
                    // pair of own wind which is also a seat wind
                    fu += 2
                }
                if form[0] == taken_tile {
                    fu += 2; // fu for tanki agari
                }
            } else if form.len() == 3 && form[0] == form[1] {
                if !is_tsumo && taken_tile == form[0] {
                    if can_be_ryanmen || can_be_kanchan || can_be_penchan {
                        fu += if is19(form[0]) { 8 } else { 4 };
                    } else {
                        fu += if is19(form[0]) { 4 } else { 2 };
                    }
                } else {
                    fu += if is19(form[0]) { 8 } else { 4 };
                }
            }
        }

        if can_be_penchan || can_be_kanchan {
            if !have_pinfu && !can_be_tanki {
                fu += 2;
            } else if !can_be_shanpon && !can_be_ryanmen && !can_be_tanki {
                fu += 2;
            }
        }

        if is_tsumo {
            fu += 2;
        }

        fu = ceil10(fu);
        if fu < 30 {
            fu = 30;
        }
    }

    Option::from(fu)
}

fn calc_yaku(
    haipai: &Vec<i8>,
    haipai34: &Vec<i8>,
    furo: &Vec<Vec<i8>>,
    current_pattern: &Vec<Vec<i8>>,
    settings: &RiichiOptions,
    is_tsumo: bool,
) -> (Vec<(i8, i8)>, i8, i32) {
    let mut yaku_list: Vec<(i8, i8)> = Vec::new();
    let mut yakuman = 0;
    let mut han: i32 = 0;

    let check_input = YakuCheckInput {
        haipai,
        haipai34,
        furo,
        current_pattern,
        taken_tile: settings.tile_discarded_by_someone,
        is_tsumo,
        jikaze: settings.jikaze,
        bakaze: settings.bakaze,
        first_take: settings.first_take,
        riichi: settings.riichi,
        double_riichi: settings.double_riichi,
        ippatsu: settings.ippatsu,
        after_kan: settings.after_kan,
        last_tile: settings.last_tile,
        allow_kuitan: settings.allow_kuitan,
    };

    for y in 0..YAKU_SETTINGS.len() {
        let yaku = &YAKU_SETTINGS[y];
        if settings.disabled_yaku.contains(&(y as i8)) {
            continue;
        }
        if yaku.is_local
            && !settings.all_local_yaku_enabled
            && !settings.local_yaku_enabled.contains(&(y as i8))
        {
            continue;
        }
        if yakuman > 0 && yaku.yakuman == 0 {
            continue;
        }
        if yaku.is_menzen_only && !is_menzen(furo) {
            continue;
        }

        let checker = &yaku.check;
        if checker(&check_input) {
            if yaku.yakuman > 0 {
                let n = if settings.allow_double_yakuman {
                    yaku.yakuman
                } else {
                    1
                };
                yakuman += n;
                yaku_list.push((y as i8, if n > 1 { 26 } else { 13 }));
            } else {
                let mut n = yaku.han;
                if yaku.is_furo_minus && !is_menzen(furo) {
                    n -= 1;
                }
                yaku_list.push((y as i8, n));
                han += n as i32;
            }
        }
    }

    (yaku_list, yakuman, han)
}

fn calc_all(
    haipai: &Vec<i8>,
    haipai34: &Vec<i8>,
    furo: &Vec<Vec<i8>>,
    dora: &Vec<i8>,
    opts: &RiichiOptions,
    is_tsumo: bool,
    calc_hairi: bool,
) -> Result<RiichiResult, String> {
    let mut result = RiichiResult {
        is_agari: false,
        yakuman: 0,
        han: 0,
        fu: 0,
        ten: 0,
        outgoing_ten: Option::from((0, 0)),
        yaku: Vec::new(),
        hairi: None,
    };

    result.is_agari = check_all(haipai34);

    if !result.is_agari || haipai.len() + furo.len() * 3 != 14 {
        if calc_hairi {
            result.hairi = hairi(haipai34.clone().as_mut());
        }
        return Ok(result);
    }

    result.is_agari = true;
    let agari_patterns = find_all_agari_patterns(haipai34);

    for v in agari_patterns {
        let mut current_pattern = v.clone();
        for f in furo {
            current_pattern.push(f.clone());
        }
        let (mut yaku_list, yakuman, mut han) =
            calc_yaku(haipai, haipai34, furo, &current_pattern, &opts, is_tsumo);
        if yakuman == 0 && han == 0 {
            continue;
        }

        let mut fu = 0;
        if han > 0 || yakuman > 0 {
            let (dora_count, akadora_count) =
                calc_all_dora(haipai, furo, dora, han, opts.aka_count, opts.allow_aka);
            if dora_count > 0 {
                han += dora_count as i32;
                yaku_list.push((Yaku::Dora as i8, dora_count));
            }
            if opts.allow_aka && akadora_count > 0 {
                han += akadora_count as i32;
                yaku_list.push((Yaku::Akadora as i8, akadora_count));
            }
            fu = calc_fu(
                is_tsumo,
                opts.bakaze,
                opts.jikaze,
                yaku_list.iter().map(|(y, _c)| *y).collect::<Vec<i8>>(),
                opts.tile_discarded_by_someone,
                &v,
                furo,
            )
            .unwrap_or(0);
        }
        let (total, oya, ko) = calc_ten(opts.jikaze, is_tsumo, yakuman, han, fu, opts.with_kiriage);

        // Find variant with maximum points
        if total > result.ten
            || (total == result.ten && han > result.han)
            || (total == result.ten && han == result.han && fu > result.fu)
        {
            result.ten = total;
            result.han = han;
            result.fu = fu;
            result.yaku = yaku_list;
            result.yakuman = yakuman;
            result.outgoing_ten = if is_tsumo {
                Option::from((oya, ko))
            } else {
                None
            };
        }
    }

    if result.ten == 0 {
        return Err("no yaku".parse().unwrap());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::Tiles;

    #[test]
    pub fn should_parse_yakuhai() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M1 as i8,
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                    Tiles::E as i8,
                ],
                open_part: vec![(
                    true,
                    vec![Tiles::WD as i8, Tiles::WD as i8, Tiles::WD as i8],
                )],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::E as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::E as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 30);
        assert_eq!(r.han, 1);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 1500);
        assert_eq!(r.yaku, vec![(Yaku::Haku as i8, 1)]);
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_pinfu_tsumo() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M1 as i8,
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                    Tiles::M7 as i8,
                    Tiles::M8 as i8,
                    Tiles::P5 as i8,
                    Tiles::P5 as i8,
                    Tiles::M9 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::E as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 20);
        assert_eq!(r.han, 2);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 2100);
        assert_eq!(r.outgoing_ten.unwrap(), (700, 400));
        assert_eq!(
            r.yaku,
            vec![(Yaku::Pinfu as i8, 1), (Yaku::Menzentsumo as i8, 1)]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_haku_toitoi() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::M4 as i8,
                    Tiles::M4 as i8,
                    Tiles::M4 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                ],
                open_part: vec![
                    (
                        true,
                        vec![Tiles::WD as i8, Tiles::WD as i8, Tiles::WD as i8],
                    ),
                    (
                        true,
                        vec![Tiles::M9 as i8, Tiles::M9 as i8, Tiles::M9 as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::E as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 3);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 7800);
        assert_eq!(r.outgoing_ten.unwrap(), (2600, 1300));
        assert_eq!(r.yaku, vec![(Yaku::Toitoi as i8, 2), (Yaku::Haku as i8, 1)]);
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_toitoi_chinitsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M2 as i8,
                    Tiles::M2 as i8,
                    Tiles::M2 as i8,
                    Tiles::M5 as i8,
                    Tiles::M5 as i8,
                ],
                open_part: vec![
                    (
                        true,
                        vec![Tiles::M3 as i8, Tiles::M3 as i8, Tiles::M3 as i8],
                    ),
                    (
                        true,
                        vec![Tiles::M4 as i8, Tiles::M4 as i8, Tiles::M4 as i8],
                    ),
                    (
                        true,
                        vec![Tiles::M9 as i8, Tiles::M9 as i8, Tiles::M9 as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::E as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 7);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 18000);
        assert_eq!(r.outgoing_ten.unwrap(), (6000, 3000));
        assert_eq!(
            r.yaku,
            vec![(Yaku::Chinitsu as i8, 5), (Yaku::Toitoi as i8, 2)]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_chanta_tsumo() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M1 as i8,
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::P1 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::M7 as i8,
                    Tiles::M8 as i8,
                    Tiles::M9 as i8,
                    Tiles::N as i8,
                    Tiles::N as i8,
                    Tiles::N as i8,
                    Tiles::S as i8,
                    Tiles::S as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 3);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 5200);
        assert_eq!(r.outgoing_ten.unwrap(), (2600, 1300));
        assert_eq!(
            r.yaku,
            vec![(Yaku::Chanta as i8, 2), (Yaku::Menzentsumo as i8, 1)]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_pinfu_ittsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M1 as i8,
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::M7 as i8,
                    Tiles::M8 as i8,
                    Tiles::M9 as i8,
                    Tiles::M4 as i8,
                    Tiles::M5 as i8,
                    Tiles::M6 as i8,
                    Tiles::S3 as i8,
                    Tiles::S3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::P6 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 20);
        assert_eq!(r.han, 4);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 5200);
        assert_eq!(r.outgoing_ten.unwrap(), (2600, 1300));
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Ittsu as i8, 2),
                (Yaku::Pinfu as i8, 1),
                (Yaku::Menzentsumo as i8, 1)
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_haku_toitoi_honroutou() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M1 as i8,
                    Tiles::M1 as i8,
                    Tiles::M1 as i8,
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::S as i8,
                    Tiles::S as i8,
                ],
                open_part: vec![
                    (
                        true,
                        vec![Tiles::P1 as i8, Tiles::P1 as i8, Tiles::P1 as i8],
                    ),
                    (
                        true,
                        vec![Tiles::M9 as i8, Tiles::M9 as i8, Tiles::M9 as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 50);
        assert_eq!(r.han, 5);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(r.outgoing_ten.unwrap(), (4000, 2000));
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Toitoi as i8, 2),
                (Yaku::Honroutou as i8, 2),
                (Yaku::Haku as i8, 1)
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_pinfu_sanshoku_tanyao() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P5 as i8,
                    Tiles::P6 as i8,
                    Tiles::P7 as i8,
                    Tiles::M5 as i8,
                    Tiles::M6 as i8,
                    Tiles::M7 as i8,
                    Tiles::S5 as i8,
                    Tiles::S6 as i8,
                    Tiles::S7 as i8,
                    Tiles::S3 as i8,
                    Tiles::S3 as i8,
                    Tiles::P4 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 20);
        assert_eq!(r.han, 5);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(r.outgoing_ten.unwrap(), (4000, 2000));
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Sanshoku as i8, 2),
                (Yaku::Tanyao as i8, 1),
                (Yaku::Pinfu as i8, 1),
                (Yaku::Menzentsumo as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_tanyao_chiitoitsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M2 as i8,
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::S3 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                    Tiles::S8 as i8,
                    Tiles::S8 as i8,
                    Tiles::M8 as i8,
                    Tiles::M8 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 25);
        assert_eq!(r.han, 4);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 6400);
        assert_eq!(r.outgoing_ten.unwrap(), (3200, 1600));
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Chiitoitsu as i8, 2),
                (Yaku::Tanyao as i8, 1),
                (Yaku::Menzentsumo as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_east_haku() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::E as i8,
                    Tiles::E as i8,
                    Tiles::E as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::M4 as i8,
                    Tiles::M4 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                ],
                open_part: vec![(
                    true,
                    vec![Tiles::WD as i8, Tiles::WD as i8, Tiles::WD as i8],
                )],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 2);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 2700);
        assert_eq!(r.outgoing_ten.unwrap(), (1300, 700));
        assert_eq!(
            r.yaku,
            vec![(Yaku::RoundWindEast as i8, 1), (Yaku::Haku as i8, 1),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_ryanpeiko_tanyao_pinfu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::S6 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                    Tiles::S6 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                    Tiles::P8 as i8,
                    Tiles::P8 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::P3 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 30);
        assert_eq!(r.han, 5);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Ryanpeikou as i8, 3),
                (Yaku::Tanyao as i8, 1),
                (Yaku::Pinfu as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_east_haku_honitsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::P1 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P7 as i8,
                    Tiles::P7 as i8,
                ],
                open_part: vec![
                    (true, vec![Tiles::E as i8, Tiles::E as i8, Tiles::E as i8]),
                    (
                        true,
                        vec![Tiles::P4 as i8, Tiles::P5 as i8, Tiles::P6 as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 4);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(r.outgoing_ten.unwrap(), (4000, 2000));
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Honitsu as i8, 2),
                (Yaku::RoundWindEast as i8, 1),
                (Yaku::Haku as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_east_honitsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::S1 as i8,
                    Tiles::S1 as i8,
                    Tiles::S1 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                    Tiles::S9 as i8,
                    Tiles::E as i8,
                    Tiles::E as i8,
                    Tiles::E as i8,
                    Tiles::S as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::S as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 50);
        assert_eq!(r.han, 4);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(
            r.yaku,
            vec![(Yaku::Honitsu as i8, 3), (Yaku::RoundWindEast as i8, 1),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_chinitsu_tanyao() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P2 as i8,
                    Tiles::P2 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P4 as i8,
                ],
                open_part: vec![
                    (
                        true,
                        vec![Tiles::P4 as i8, Tiles::P5 as i8, Tiles::P6 as i8],
                    ),
                    (
                        true,
                        vec![Tiles::P5 as i8, Tiles::P6 as i8, Tiles::P7 as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::P5 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 30);
        assert_eq!(r.han, 6);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 12000);
        assert_eq!(
            r.yaku,
            vec![(Yaku::Chinitsu as i8, 5), (Yaku::Tanyao as i8, 1)]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_hatsu_iipeiko() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::M1 as i8,
                    Tiles::M1 as i8,
                    Tiles::M1 as i8,
                    Tiles::GD as i8,
                    Tiles::GD as i8,
                    Tiles::GD as i8,
                    Tiles::M3 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::M3 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 50);
        assert_eq!(r.han, 2);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 3200);
        assert_eq!(
            r.yaku,
            vec![(Yaku::Iipeikou as i8, 1), (Yaku::Hatsu as i8, 1),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_honitsu_chiitoitsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::M1 as i8,
                    Tiles::M1 as i8,
                    Tiles::M4 as i8,
                    Tiles::M4 as i8,
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::W as i8,
                    Tiles::W as i8,
                    Tiles::M7 as i8,
                    Tiles::M7 as i8,
                    Tiles::M9 as i8,
                    Tiles::M9 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 25);
        assert_eq!(r.han, 6);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 12000);
        assert_eq!(r.outgoing_ten.unwrap(), (6000, 3000));
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Honitsu as i8, 3),
                (Yaku::Chiitoitsu as i8, 2),
                (Yaku::Menzentsumo as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_suukantsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![Tiles::M1 as i8],
                open_part: vec![
                    (
                        true,
                        vec![
                            Tiles::S7 as i8,
                            Tiles::S7 as i8,
                            Tiles::S7 as i8,
                            Tiles::S7 as i8,
                        ],
                    ),
                    (
                        false,
                        vec![
                            Tiles::W as i8,
                            Tiles::W as i8,
                            Tiles::W as i8,
                            Tiles::W as i8,
                        ],
                    ),
                    (
                        true,
                        vec![
                            Tiles::S1 as i8,
                            Tiles::S1 as i8,
                            Tiles::S1 as i8,
                            Tiles::S1 as i8,
                        ],
                    ),
                    (
                        true,
                        vec![
                            Tiles::P4 as i8,
                            Tiles::P4 as i8,
                            Tiles::P4 as i8,
                            Tiles::P4 as i8,
                        ],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::M1 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 90);
        assert_eq!(r.han, 0);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 32000);
        assert_eq!(r.yaku, vec![(Yaku::Suukantsu as i8, 13)]);
        assert_eq!(r.yakuman, 1);
    }

    #[test]
    pub fn should_parse_chun_sankantsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P4 as i8,
                ],
                open_part: vec![
                    (
                        false,
                        vec![
                            Tiles::RD as i8,
                            Tiles::RD as i8,
                            Tiles::RD as i8,
                            Tiles::RD as i8,
                        ],
                    ),
                    (
                        true,
                        vec![
                            Tiles::S4 as i8,
                            Tiles::S4 as i8,
                            Tiles::S4 as i8,
                            Tiles::S4 as i8,
                        ],
                    ),
                    (
                        true,
                        vec![
                            Tiles::P3 as i8,
                            Tiles::P3 as i8,
                            Tiles::P3 as i8,
                            Tiles::P3 as i8,
                        ],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 80);
        assert_eq!(r.han, 5);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Toitoi as i8, 2),
                (Yaku::Sankantsu as i8, 2),
                (Yaku::Chun as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_haku_toitoi_sanankou() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::M3 as i8,
                    Tiles::M6 as i8,
                    Tiles::M6 as i8,
                    Tiles::M6 as i8,
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::WD as i8,
                    Tiles::P9 as i8,
                ],
                open_part: vec![(
                    true,
                    vec![Tiles::S4 as i8, Tiles::S4 as i8, Tiles::S4 as i8],
                )],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::P9 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 5);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Toitoi as i8, 2),
                (Yaku::Sanankou as i8, 2),
                (Yaku::Haku as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_suuankou() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M4 as i8,
                    Tiles::M4 as i8,
                    Tiles::M4 as i8,
                    Tiles::P8 as i8,
                    Tiles::P8 as i8,
                    Tiles::P8 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                    Tiles::S2 as i8,
                    Tiles::S2 as i8,
                    Tiles::M2 as i8,
                    Tiles::M2 as i8,
                    Tiles::S2 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 0);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 32000);
        assert_eq!(r.outgoing_ten.unwrap(), (16000, 8000));
        assert_eq!(r.yaku, vec![(Yaku::Suuankou as i8, 13),]);
        assert_eq!(r.yakuman, 1);
    }

    #[test]
    pub fn should_parse_sanankou_chinitsu_tanyao() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::S2 as i8,
                    Tiles::S2 as i8,
                    Tiles::S2 as i8,
                    Tiles::S3 as i8,
                    Tiles::S3 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S4 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                    Tiles::S6 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 10);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 16000);
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Chinitsu as i8, 6),
                (Yaku::Sanankou as i8, 2),
                (Yaku::Tanyao as i8, 1),
                (Yaku::Menzentsumo as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_sanshoku_junchan() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P1 as i8,
                    Tiles::P2 as i8,
                    Tiles::M1 as i8,
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::P9 as i8,
                    Tiles::P9 as i8,
                    Tiles::P9 as i8,
                    Tiles::M9 as i8,
                    Tiles::M9 as i8,
                ],
                open_part: vec![(
                    true,
                    vec![Tiles::S3 as i8, Tiles::S1 as i8, Tiles::S2 as i8],
                )],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::P3 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 5);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(
            r.yaku,
            vec![(Yaku::Junchan as i8, 3), (Yaku::Sanshoku as i8, 2),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_pinfu_junchan() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M2 as i8,
                    Tiles::M3 as i8,
                    Tiles::M7 as i8,
                    Tiles::M8 as i8,
                    Tiles::M9 as i8,
                    Tiles::P1 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                    Tiles::S9 as i8,
                    Tiles::S9 as i8,
                    Tiles::S9 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::M1 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 30);
        assert_eq!(r.han, 4);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 7700);
        assert_eq!(
            r.yaku,
            vec![(Yaku::Junchan as i8, 3), (Yaku::Pinfu as i8, 1),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_pinfu_iipeiko_ittsu_chinitsu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::S1 as i8,
                    Tiles::S1 as i8,
                    Tiles::S2 as i8,
                    Tiles::S2 as i8,
                    Tiles::S3 as i8,
                    Tiles::S3 as i8,
                    Tiles::S4 as i8,
                    Tiles::S4 as i8,
                    Tiles::S5 as i8,
                    Tiles::S6 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                    Tiles::S9 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::S4 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 30);
        assert_eq!(r.han, 10);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 16000);
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::Chinitsu as i8, 6),
                (Yaku::Ittsu as i8, 2),
                (Yaku::Pinfu as i8, 1),
                (Yaku::Iipeikou as i8, 1),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_toitoi_sandoukou() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P4 as i8,
                    Tiles::P4 as i8,
                    Tiles::P4 as i8,
                    Tiles::M1 as i8,
                ],
                open_part: vec![
                    (
                        true,
                        vec![Tiles::M4 as i8, Tiles::M4 as i8, Tiles::M4 as i8],
                    ),
                    (
                        true,
                        vec![Tiles::S4 as i8, Tiles::S4 as i8, Tiles::S4 as i8],
                    ),
                    (
                        true,
                        vec![Tiles::M9 as i8, Tiles::M9 as i8, Tiles::M9 as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::M1 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 4);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 8000);
        assert_eq!(
            r.yaku,
            vec![(Yaku::Toitoi as i8, 2), (Yaku::SanshokuDoukou as i8, 2),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn should_parse_daisangen() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P4 as i8,
                    Tiles::P4 as i8,
                    Tiles::P4 as i8,
                    Tiles::M1 as i8,
                ],
                open_part: vec![
                    (
                        true,
                        vec![Tiles::WD as i8, Tiles::WD as i8, Tiles::WD as i8],
                    ),
                    (
                        true,
                        vec![Tiles::GD as i8, Tiles::GD as i8, Tiles::GD as i8],
                    ),
                    (
                        true,
                        vec![Tiles::RD as i8, Tiles::RD as i8, Tiles::RD as i8],
                    ),
                ],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::M1 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: true, // enable open tanyao
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 40);
        assert_eq!(r.han, 0);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 32000);
        assert_eq!(r.yaku, vec![(Yaku::Daisangen as i8, 13),]);
        assert_eq!(r.yakuman, 1);
    }

    #[test]
    pub fn debug_1_should_parse_tsumo_pinfu() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::M4 as i8,
                    Tiles::M5 as i8,
                    Tiles::M6 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::P7 as i8,
                    Tiles::P8 as i8,
                    Tiles::P9 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                    Tiles::S7 as i8,
                    Tiles::S8 as i8,
                    Tiles::S6 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::W as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 20);
        assert_eq!(r.han, 2);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 1500);
        assert_eq!(r.outgoing_ten.unwrap(), (700, 400));
        assert_eq!(
            r.yaku,
            vec![(Yaku::Pinfu as i8, 1), (Yaku::Menzentsumo as i8, 1),]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn debug_2_should_parse_double_riichi_tsumo_ittsu_10dora() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P1 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::P6 as i8,
                    Tiles::P7 as i8,
                    Tiles::P8 as i8,
                    Tiles::P9 as i8,
                    Tiles::S5 as i8,
                    Tiles::S5 as i8,
                ],
                open_part: vec![(
                    false,
                    vec![
                        Tiles::M1 as i8,
                        Tiles::M1 as i8,
                        Tiles::M1 as i8,
                        Tiles::M1 as i8,
                    ],
                )],
            },
            &mut RiichiOptions {
                dora: vec![
                    Tiles::S5 as i8,
                    Tiles::M1 as i8,
                    Tiles::S3 as i8,
                    Tiles::M1 as i8,
                ],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: true,
                after_kan: false,
                tile_discarded_by_someone: -1,
                bakaze: Tiles::S as i8,
                jikaze: Tiles::E as i8,
                allow_aka: false,
                allow_kuitan: false,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 60);
        assert_eq!(r.han, 15);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 48000);
        assert_eq!(
            r.yaku,
            vec![
                (Yaku::DaburuRiichi as i8, 2),
                (Yaku::Ittsu as i8, 2),
                (Yaku::Menzentsumo as i8, 1),
                (Yaku::Dora as i8, 10),
            ]
        );
        assert_eq!(r.yakuman, 0);
    }

    #[test]
    pub fn debug_3_should_parse_chuurenpoto() {
        let res = calc_riichi(
            RiichiHand {
                closed_part: vec![
                    Tiles::P1 as i8,
                    Tiles::P1 as i8,
                    Tiles::P2 as i8,
                    Tiles::P2 as i8,
                    Tiles::P3 as i8,
                    Tiles::P4 as i8,
                    Tiles::P5 as i8,
                    Tiles::P6 as i8,
                    Tiles::P7 as i8,
                    Tiles::P8 as i8,
                    Tiles::P9 as i8,
                    Tiles::P9 as i8,
                    Tiles::P9 as i8,
                ],
                open_part: vec![],
            },
            &mut RiichiOptions {
                dora: vec![Tiles::S8 as i8, Tiles::M5 as i8],
                aka_count: 0,
                first_take: false,
                riichi: false,
                ippatsu: false,
                double_riichi: false,
                after_kan: false,
                tile_discarded_by_someone: Tiles::P1 as i8,
                bakaze: Tiles::E as i8,
                jikaze: Tiles::S as i8,
                allow_aka: true,
                allow_kuitan: true,
                with_kiriage: false,
                disabled_yaku: vec![],
                local_yaku_enabled: vec![],
                all_local_yaku_enabled: false,
                allow_double_yakuman: false,
                last_tile: false,
            },
            false,
        );

        assert!(!res.is_err());
        let r = res.unwrap();
        assert_eq!(r.fu, 50);
        assert_eq!(r.han, 0);
        assert_eq!(r.is_agari, true);
        assert_eq!(r.ten, 32000);
        assert_eq!(r.yaku, vec![(Yaku::Chuurenpoto as i8, 13),]);
        assert_eq!(r.yakuman, 1);
    }
}
