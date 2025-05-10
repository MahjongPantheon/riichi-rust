use crate::constants::sum;
use crate::interfaces::HairiResult;

// Ported from https://github.com/MahjongRepository/mahjong/blob/master/mahjong/shanten.py
// Implements tenhou.net logic for shanten calculation; hairi calculation taken from https://github.com/takayama-lily/riichi

const AGARI_STATE: i8 = -1;
const HONOR_INDICES: &[usize] = &[27, 28, 29, 30, 31, 32, 33];
const TERMINAL_INDICES: &[usize] = &[0, 8, 9, 17, 18, 26];

struct Shanten {
    tiles: Vec<i8>,
    number_melds: i8,
    number_tatsu: i8,
    number_pairs: i8,
    number_jidahai: i8,
    number_characters: i32,
    number_isolated_tiles: i32,
    min_shanten: i8,
}

impl Shanten {
    fn new() -> Self {
        Shanten {
            tiles: Vec::new(),
            number_melds: 0,
            number_tatsu: 0,
            number_pairs: 0,
            number_jidahai: 0,
            number_characters: 0,
            number_isolated_tiles: 0,
            min_shanten: 8,
        }
    }

    fn calculate_shanten(
        &mut self,
        tiles_34: &[i8],
        use_chiitoitsu: bool,
        use_kokushi: bool,
    ) -> i8 {
        let mut shanten_results = vec![self.calculate_shanten_for_regular_hand(&tiles_34.to_vec())];

        if use_chiitoitsu {
            shanten_results.push(self.calculate_shanten_for_chiitoitsu_hand(tiles_34));
        }
        if use_kokushi {
            shanten_results.push(self.calculate_shanten_for_kokushi_hand(tiles_34));
        }

        *shanten_results.iter().min().unwrap()
    }

    fn calculate_shanten_for_chiitoitsu_hand(&self, tiles_34: &[i8]) -> i8 {
        let pairs = tiles_34.iter().filter(|&&x| x >= 2).count();
        if pairs == 7 {
            return AGARI_STATE;
        }

        let kinds = tiles_34.iter().filter(|&&x| x >= 1).count();
        6 - (pairs as i8) + if kinds < 7 { 7 - (kinds as i8) } else { 0 }
    }

    fn calculate_shanten_for_kokushi_hand(&self, tiles_34: &[i8]) -> i8 {
        let indices: Vec<usize> = TERMINAL_INDICES
            .iter()
            .chain(HONOR_INDICES.iter())
            .cloned()
            .collect();

        let completed_terminals = indices.iter().filter(|&&i| tiles_34[i] >= 2).count();

        let terminals = indices.iter().filter(|&&i| tiles_34[i] != 0).count();

        13 - (terminals as i8) - if completed_terminals > 0 { 1 } else { 0 }
    }

    fn calculate_shanten_for_regular_hand(&mut self, tiles_34: &[i8]) -> i8 {
        let tiles = tiles_34.to_vec();
        self.init(&tiles);

        let count_of_tiles: i8 = tiles.iter().sum();
        assert!(count_of_tiles <= 14, "Too many tiles = {}", count_of_tiles);

        self.remove_character_tiles(count_of_tiles);

        let init_mentsu = (14 - count_of_tiles) / 3;
        self.scan(init_mentsu);

        self.min_shanten
    }

    fn init(&mut self, tiles: &[i8]) {
        self.tiles = tiles.to_vec();
        self.number_melds = 0;
        self.number_tatsu = 0;
        self.number_pairs = 0;
        self.number_jidahai = 0;
        self.number_characters = 0;
        self.number_isolated_tiles = 0;
        self.min_shanten = 8;
    }

    fn scan(&mut self, init_mentsu: i8) {
        for i in 0..27 {
            if self.tiles[i] == 4 {
                self.number_characters |= 1 << i;
            }
        }
        self.number_melds += init_mentsu;
        self.run(0);
    }

    fn run(&mut self, mut depth: usize) {
        if self.min_shanten == AGARI_STATE {
            return;
        }

        while depth < 27 && self.tiles[depth] == 0 {
            depth += 1;
        }

        if depth >= 27 {
            self.update_result();
            return;
        }

        let mut i = depth;
        if i > 8 {
            i -= 9;
        }
        if i > 8 {
            i -= 9;
        }

        match self.tiles[depth] {
            4 => {
                self.increase_set(depth);
                if i < 7 && self.tiles[depth + 2] != 0 {
                    if self.tiles[depth + 1] != 0 {
                        self.increase_shuntsu(depth);
                        self.run(depth + 1);
                        self.decrease_shuntsu(depth);
                    }
                    self.increase_tatsu_second(depth);
                    self.run(depth + 1);
                    self.decrease_tatsu_second(depth);
                }

                if i < 8 && self.tiles[depth + 1] != 0 {
                    self.increase_tatsu_first(depth);
                    self.run(depth + 1);
                    self.decrease_tatsu_first(depth);
                }

                self.increase_isolated_tile(depth);
                self.run(depth + 1);
                self.decrease_isolated_tile(depth);
                self.decrease_set(depth);
                self.increase_pair(depth);

                // Continue with a similar pattern...
                self.decrease_pair(depth);
            }
            3 => {
                self.increase_set(depth);
                self.run(depth + 1);
                self.decrease_set(depth);
                self.increase_pair(depth);

                if i < 7 && self.tiles[depth + 1] != 0 && self.tiles[depth + 2] != 0 {
                    self.increase_shuntsu(depth);
                    self.run(depth + 1);
                    self.decrease_shuntsu(depth);
                } else {
                    if i < 7 && self.tiles[depth + 2] != 0 {
                        self.increase_tatsu_second(depth);
                        self.run(depth + 1);
                        self.decrease_tatsu_second(depth);
                    }

                    if i < 8 && self.tiles[depth + 1] != 0 {
                        self.increase_tatsu_first(depth);
                        self.run(depth + 1);
                        self.decrease_tatsu_first(depth);
                    }
                }

                self.decrease_pair(depth);
            }
            2 => {
                self.increase_pair(depth);
                self.run(depth + 1);
                self.decrease_pair(depth);

                if i < 7 && self.tiles[depth + 2] != 0 && self.tiles[depth + 1] != 0 {
                    self.increase_shuntsu(depth);
                    self.run(depth);
                    self.decrease_shuntsu(depth);
                }
            }
            1 => {
                if i < 6
                    && self.tiles[depth + 1] == 1
                    && self.tiles[depth + 2] != 0
                    && self.tiles[depth + 3] != 4
                {
                    self.increase_shuntsu(depth);
                    self.run(depth + 2);
                    self.decrease_shuntsu(depth);
                } else {
                    self.increase_isolated_tile(depth);
                    self.run(depth + 1);
                    self.decrease_isolated_tile(depth);

                    if i < 7 && self.tiles[depth + 2] != 0 {
                        if self.tiles[depth + 1] != 0 {
                            self.increase_shuntsu(depth);
                            self.run(depth + 1);
                            self.decrease_shuntsu(depth);
                        }
                        self.increase_tatsu_second(depth);
                        self.run(depth + 1);
                        self.decrease_tatsu_second(depth);
                    }

                    if i < 8 && self.tiles[depth + 1] != 0 {
                        self.increase_tatsu_first(depth);
                        self.run(depth + 1);
                        self.decrease_tatsu_first(depth);
                    }
                }
            }
            _ => {}
        }
    }

    fn update_result(&mut self) {
        let mut ret_shanten = 8 - self.number_melds * 2 - self.number_tatsu - self.number_pairs;
        let mut n_mentsu_kouho = self.number_melds + self.number_tatsu;

        if self.number_pairs > 0 {
            n_mentsu_kouho += self.number_pairs - 1;
        } else if self.number_characters != 0 && self.number_isolated_tiles != 0 {
            if (self.number_characters | self.number_isolated_tiles) == self.number_characters {
                ret_shanten += 1;
            }
        }

        if n_mentsu_kouho > 4 {
            ret_shanten += n_mentsu_kouho - 4;
        }

        if ret_shanten != AGARI_STATE && ret_shanten < self.number_jidahai {
            ret_shanten = self.number_jidahai;
        }

        if ret_shanten < self.min_shanten {
            self.min_shanten = ret_shanten;
        }
    }

    // Helper methods for tile manipulation
    fn increase_set(&mut self, k: usize) {
        self.tiles[k] -= 3;
        self.number_melds += 1;
    }

    fn decrease_set(&mut self, k: usize) {
        self.tiles[k] += 3;
        self.number_melds -= 1;
    }

    fn increase_pair(&mut self, k: usize) {
        self.tiles[k] -= 2;
        self.number_pairs += 1;
    }

    fn decrease_pair(&mut self, k: usize) {
        self.tiles[k] += 2;
        self.number_pairs -= 1;
    }

    fn increase_shuntsu(&mut self, k: usize) {
        self.tiles[k] -= 1;
        self.tiles[k + 1] -= 1;
        self.tiles[k + 2] -= 1;
        self.number_melds += 1;
    }

    fn decrease_shuntsu(&mut self, k: usize) {
        self.tiles[k] += 1;
        self.tiles[k + 1] += 1;
        self.tiles[k + 2] += 1;
        self.number_melds -= 1;
    }

    fn increase_tatsu_first(&mut self, k: usize) {
        self.tiles[k] -= 1;
        self.tiles[k + 1] -= 1;
        self.number_tatsu += 1;
    }

    fn decrease_tatsu_first(&mut self, k: usize) {
        self.tiles[k] += 1;
        self.tiles[k + 1] += 1;
        self.number_tatsu -= 1;
    }

    fn increase_tatsu_second(&mut self, k: usize) {
        self.tiles[k] -= 1;
        self.tiles[k + 2] -= 1;
        self.number_tatsu += 1;
    }

    fn decrease_tatsu_second(&mut self, k: usize) {
        self.tiles[k] += 1;
        self.tiles[k + 2] += 1;
        self.number_tatsu -= 1;
    }

    fn increase_isolated_tile(&mut self, k: usize) {
        self.tiles[k] -= 1;
        self.number_isolated_tiles |= 1 << k;
    }

    fn decrease_isolated_tile(&mut self, k: usize) {
        self.tiles[k] += 1;
        self.number_isolated_tiles &= !(1 << k);
    }

    fn remove_character_tiles(&mut self, nc: i8) {
        let mut number = 0;
        let mut isolated = 0;

        for i in 27..34 {
            match self.tiles[i] {
                4 => {
                    self.number_melds += 1;
                    self.number_jidahai += 1;
                    number |= 1 << (i - 27);
                    isolated |= 1 << (i - 27);
                }
                3 => self.number_melds += 1,
                2 => self.number_pairs += 1,
                1 => isolated |= 1 << (i - 27),
                _ => {}
            }
        }

        if self.number_jidahai > 0 && (nc % 3) == 2 {
            self.number_jidahai -= 1;
        }

        if isolated != 0 {
            self.number_isolated_tiles |= 1 << 27;
            if (number | isolated) == number {
                self.number_characters |= 1 << 27;
            }
        }
    }
}

pub fn calc_shanten(tiles_34: &[i8]) -> i8 {
    let mut shanten = Shanten::new();
    shanten.calculate_shanten(tiles_34, true, true)
}

pub fn hairi(tiles_34: &mut Vec<i8>) -> Option<HairiResult> {
    let sht = calc_shanten(tiles_34);

    let mut res: HairiResult = HairiResult {
        now: sht,
        wait: Vec::new(),
        waits_after_discard: Vec::new(),
    };

    if sht == -1 {
        return None;
    }

    let calc_hairi = |tiles_34: &mut Vec<i8>, current_index: i8| -> Vec<i8> {
        let mut waits: Vec<i8> = Vec::new();
        for i in 0..34 {
            if i == current_index {
                continue;
            }
            tiles_34[i as usize] += 1;
            if calc_shanten(tiles_34) < sht {
                waits.push(i);
            }
            tiles_34[i as usize] -= 1;
        }
        waits
    };

    // 13-tile hand: calculate hairi once and return waits
    if sum(tiles_34) % 3 == 1 {
        res.wait = calc_hairi(tiles_34, -1);
        return Option::from(res);
    }

    // 14-tile non-tempai hand: try to detect possible discards and waits after it
    let mut waits_after_discard: Vec<(i8, Vec<i8>)> = Vec::new();
    for i in 0..34 {
        if tiles_34[i] == 0 {
            continue;
        }
        tiles_34[i] -= 1;
        if calc_shanten(tiles_34) == sht {
            waits_after_discard.push((i as i8, calc_hairi(tiles_34, i as i8)));
        }
        tiles_34[i] += 1;
    }

    res.waits_after_discard = waits_after_discard;
    Option::from(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::Tiles::*;

    #[test]
    pub fn shanten_13tiles_works() {
        // Hand with 13 tiles but not tenpai
        let res = calc_shanten(&vec![
            2, 2, 2, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 1, 0, 0, //
            0, 1, 0, 0, 1, 2, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0,
        ]);
        assert_eq!(res, 1);
    }

    #[test]
    pub fn shanten_works() {
        let hands = [
            ([M4, M5, P1, P2, P7, P7, P8, S3, S7, E, S, W, WD, RD], 5),
            ([M2, M5, P2, S1, S1, S4, S5, S8, S9, S9, S, W, WD, GD], 4),
            ([M2, M3, M3, M4, M5, M6, M7, P3, P6, P6, S4, S7, S8, GD], 2),
            ([M6, M7, M8, P2, P4, P5, P6, P6, S3, S4, S5, S7, S8, RD], 1),
            ([M2, M3, M6, M6, M7, M8, M9, P2, P5, P6, S4, S8, W, W], 2),
            ([M8, M9, P3, P3, P4, P6, P7, S2, S6, S7, S8, S9, S, GD], 3),
            ([M2, M3, P4, P5, P5, P9, S3, S4, S7, S8, S8, S8, N, RD], 3),
            ([M1, M1, M2, M4, M6, M6, P6, P7, S4, S5, S7, S8, S8, GD], 3),
            ([M3, M5, M5, M6, M7, M8, P6, P6, P7, S2, S3, S7, S8, S9], 1),
            ([M2, M3, M6, M7, P3, P4, P5, P8, P8, P9, S9, E, WD, RD], 3),
            ([M3, M3, M4, M7, M7, P1, P3, P4, P8, S3, S4, S6, S7, S8], 2),
            ([M2, M4, M6, M6, P1, P2, P5, P6, P7, S1, S2, S4, S6, S8], 2),
            ([M2, M2, M5, M8, P2, P3, P8, P9, S3, S3, S3, S5, S5, S5], 1),
            ([M4, M4, M5, M6, M7, M8, P4, P5, S1, S1, S2, S2, S3, S3], 0),
            ([M1, M3, M4, M4, P3, P5, P7, S3, S3, S5, W, W, WD, WD], 2),
            ([M1, M3, M5, M7, M7, P2, P3, S4, S4, S5, S5, S7, E, E], 2),
            ([M3, M3, M5, M9, M9, M9, P6, S4, S5, S5, S7, S8, S8, S9], 2),
            ([M1, M2, M3, M7, M9, P1, P2, P6, S1, S3, S3, S4, S7, S8], 2),
            ([M1, M4, M6, M7, P2, P2, S2, S4, S6, S6, S, N, N, GD], 3),
            ([M1, M4, M6, M8, M9, M9, P8, P9, S5, S5, E, N, GD, RD], 4),
            ([M1, M2, M4, M4, M8, P3, P4, S5, S6, S6, S7, E, E, WD], 2),
            ([M2, M6, M6, M7, M7, P1, P5, P7, P9, S6, E, WD, WD, RD], 3),
            ([M3, M4, P1, P2, P3, P4, P7, P9, S5, S6, S7, WD, GD, RD], 2),
            ([M5, M7, M8, P3, P3, S2, S3, S4, S7, S8, S8, E, S, W], 3),
            ([M7, M9, P1, P1, P2, P3, P3, P5, P8, S2, S4, S7, S8, S8], 2),
            ([M6, M6, P2, P3, P4, P8, P9, S1, S3, S4, S5, S6, S6, S6], 0),
            ([M3, M4, M7, M7, P1, P1, P2, P3, P4, S4, S6, S6, S7, S], 2),
            ([M1, M2, M3, M5, M8, M9, P2, P4, S4, S6, S9, W, GD, RD], 3),
            ([M1, M1, M7, P3, P4, P6, P7, P8, S3, S4, E, E, E, W], 1),
            ([M6, M7, M8, P1, P2, P3, P6, P7, P8, P9, P9, S5, S6, N], 0),
            ([M3, M5, P3, P4, P5, P6, P7, P8, P8, S1, S2, S3, E, E], 0),
            ([M6, M8, P1, P2, P2, P4, P4, P5, P5, S3, S5, S8, S8, W], 2),
            ([M2, M3, M4, M6, P4, P5, P6, P7, S1, S2, S3, S3, S4, S5], 0),
            ([M2, M2, M4, M7, M8, M9, P6, S1, S2, S3, S3, S5, RD, RD], 1),
            ([M4, M4, M6, M9, M9, P2, P5, P7, P9, S1, S6, S8, E, RD], 4),
            ([M4, M5, M5, M7, M7, P2, P5, P8, P9, S2, S3, S5, S5, WD], 3),
            ([M4, M7, M9, P4, P7, P7, P8, S1, S2, S3, S5, S9, W, W], 3),
            ([M2, M3, M4, P4, P4, P6, P8, S2, S2, S3, S4, S4, S6, W], 1),
            ([M3, M5, M5, M6, M7, P2, P3, P7, S5, S5, S6, S7, S7, S7], 1),
            ([P3, P5, P6, P7, P7, P8, P8, P9, S6, S6, S6, S7, S8, S9], 0),
            ([M1, M2, M7, M9, M9, P1, P3, P9, P9, S1, S3, S3, S4, S6], 3),
            ([M3, M4, M4, M4, M5, M6, M7, P5, P6, P7, P8, S6, S7, S8], 0),
            ([M6, M6, M8, P2, P3, P3, P4, P7, S6, S6, S7, S8, RD, RD], 2),
            ([M3, M4, M6, P4, P5, P6, P8, P9, P9, S4, S5, S6, E, RD], 2),
            ([M7, M8, P1, P2, P3, P7, S4, S5, S6, E, E, W, W, RD], 1),
            ([M2, M3, M5, M6, M6, M7, P3, P5, S2, S3, S4, S7, S8, N], 2),
            ([M1, M5, M6, M9, P2, P2, P3, P6, P6, S3, S3, S3, RD, RD], 2),
            ([M1, M2, M6, M6, P1, P3, P6, P6, P7, P8, P8, S3, S4, S5], 1),
            ([M3, M5, M9, P2, P3, P5, P7, S2, S4, S5, S7, S9, N, GD], 4),
            ([M1, M6, M8, P2, P3, P4, P6, P8, P9, S2, S5, S7, E, RD], 3),
            ([M3, M4, M8, M9, P4, P7, P8, P9, S7, E, S, W, W, N], 3),
            ([M1, M1, M9, M9, P4, P4, P5, P5, P9, P9, S5, S5, E, GD], 0),
            ([M2, M4, P2, P2, P3, P4, P4, P5, P5, P6, P8, P8, S7, N], 1),
            ([M2, M3, M6, M7, M8, P1, P2, P4, P4, P6, P7, P8, E, W], 1),
            ([M1, M5, M6, M7, M7, P2, P4, P4, P5, P6, S2, S4, S6, S6], 1),
            ([M3, M4, M6, M8, P1, P2, P5, P8, S2, S5, S7, S7, S7, S8], 3),
            ([M2, M3, M3, M5, M6, M6, M8, P4, P5, P6, S2, S4, S7, WD], 2),
            ([M1, M3, M4, M6, M7, M8, P2, P2, P5, P5, P6, P7, S2, S8], 2),
            ([M4, M4, M7, M7, M9, M9, P3, P3, P9, P9, S1, S6, S6, RD], 0),
            ([M3, M4, M5, M8, M9, M9, P2, P5, P6, P6, S1, S1, S5, S6], 2),
            ([M3, M5, M7, M8, P3, P3, P3, P6, S1, S1, S3, S7, S8, S9], 1),
            ([M1, M3, M6, M7, M7, M9, P3, P3, P4, S1, S4, S6, S6, WD], 3),
            ([M5, M6, M7, M7, P2, P4, P4, P5, P6, P7, S1, S4, S6, N], 2),
            ([M7, M8, S1, S3, S5, S6, S7, S9, E, S, W, WD, RD, RD], 3),
            ([M1, M3, M3, M6, P1, P3, P3, P5, P6, P7, P8, P8, P9, P9], 2),
            ([M4, M6, M6, M6, M7, M8, P5, P5, P9, P9, S5, WD, RD, RD], 2),
            ([M1, M2, M3, M4, M5, M6, M7, M8, P2, P4, P4, S4, S5, S7], 1),
            ([M2, M2, M3, M3, M4, M5, P2, P2, P6, P6, P8, S5, S7, E], 2),
            ([M5, M6, M7, P1, P2, P3, P5, P7, S5, S5, S7, S8, S8, S9], 0),
            ([M2, M3, M3, M3, M4, M6, M7, M7, M9, P2, P8, S4, S6, WD], 2),
            ([M1, M2, M3, M4, P1, P2, P3, P6, P8, P9, P9, S3, S4, S7], 1),
            ([M4, M6, P2, P3, P4, P6, P7, P8, S1, S1, S4, S5, S6, RD], 0),
            ([M2, M3, M4, M5, M5, M6, M6, P6, P7, P7, P8, S4, S4, S4], 0),
            ([M4, M5, M6, P4, P4, P5, P5, P6, P7, P8, P8, S4, S6, S9], 1),
            ([M1, M2, M2, M4, M4, M5, P4, P5, S2, S4, S4, S6, S8, RD], 3),
            ([P2, P4, P5, P6, P8, P8, P8, P9, S5, S6, S9, N, N, GD], 2),
            ([M4, M5, M5, P3, P6, P8, P8, S3, S4, S5, S6, S7, S9, E], 3),
            ([M1, M3, M6, M6, M7, M8, M9, P1, P1, P2, P4, S1, S2, S], 2),
            ([M4, M5, M6, P4, P5, P6, P7, P8, P8, S4, S4, S5, S7, S8], 1),
            ([M5, M5, M8, P2, P3, P6, S1, S1, S4, S5, S7, S8, S9, N], 2),
            ([M3, M5, M6, M6, M8, S1, S1, S2, S5, S5, S6, S7, E, W], 3),
            ([M4, M4, M8, M9, P4, P5, S1, S2, S4, S7, S8, S8, WD, WD], 3),
            ([M2, M3, M4, P1, P3, P3, P5, P6, P7, P8, P9, W, RD, RD], 1),
            ([M3, M7, M9, P1, P4, P7, P8, S2, S3, S3, S4, S5, S8, S9], 3),
            ([M6, M7, M8, M8, M8, M9, M9, P3, P4, P5, S7, S8, N, N], 1),
            ([M6, M8, P3, P3, P6, P7, P7, P8, P9, S4, S4, S4, S5, S7], 1),
            ([M1, M2, M3, M5, M7, M8, M9, S4, S5, S8, S8, WD, WD, GD], 1),
            ([M5, P3, P7, S2, S3, S3, S8, S8, E, S, N, N, WD, GD], 3),
            ([P3, P4, P5, S1, S2, S3, S4, S4, S5, S5, S6, S, N, N], 0),
            ([M2, M3, M7, P4, P5, P6, P7, P9, S3, S6, S7, S8, E, WD], 2),
            ([M2, M2, M3, M3, M4, M5, M5, M5, M6, M8, M8, P9, S5, S], 2),
            ([M2, M3, M3, P1, P2, P9, P9, S1, S2, S3, S6, S7, S7, GD], 2),
            ([M1, M4, P2, P2, P5, P8, S3, S4, S6, S7, S8, N, WD, GD], 4),
            ([M5, M8, P1, P1, P2, P3, S2, S5, S5, S7, S9, WD, GD, RD], 4),
            ([M1, M2, M4, M4, M6, M9, P4, P7, S4, S8, E, S, S, WD], 4),
            ([M3, M3, M5, M6, P1, P4, P5, P6, P7, S1, S3, S8, S8, S8], 1),
            ([M3, M4, M6, P2, P2, P3, P5, S4, S4, S8, S8, WD, GD, GD], 2),
            ([M1, M3, M8, P1, P4, P5, P7, P7, P9, S5, S6, S6, S9, N], 4),
            ([M1, M5, M6, M7, P3, P4, P7, S1, S3, S4, S5, S9, E, S], 3),
            ([M4, P3, P4, P5, P6, P7, P8, S1, S1, S2, S4, S5, S5, S], 1),
            ([M3, M3, M3, M6, M8, P2, P4, P5, P6, P7, P7, S3, S3, S5], 1),
            ([M1, M1, M2, M3, M9, M9, P6, S5, S7, S7, S9, E, W, RD], 3),
            ([M4, M5, M7, M7, M8, P6, P6, P6, S5, S6, S8, S8, S9, WD], 2),
            ([M2, M2, M6, M7, P2, P5, P7, P8, S3, S5, S5, S7, N, N], 3),
            ([M4, M4, M7, M8, M9, P2, P4, P7, P8, S5, S6, S7, S7, N], 1),
            ([M6, M8, M8, P3, P4, P5, P6, P7, S3, S8, S8, S9, W, W], 2),
            ([M1, P4, P4, P7, P9, S1, S5, E, S, S, W, W, WD, RD], 3),
            ([M4, M4, M6, M7, P2, P4, P5, P7, S2, S4, S4, S6, S8, S], 3),
            ([M4, M4, M5, M7, M8, P9, S3, S4, S6, S6, S9, S, N, N], 3),
            ([M1, M2, M3, M3, P2, P3, P6, P6, P8, P8, S1, S2, S4, S8], 2),
            ([M1, M4, M6, P2, P6, P8, P9, S1, S2, S2, S8, W, WD, RD], 5),
            ([M4, M9, P3, P4, P4, P9, S4, S5, S5, S7, S8, S9, W, RD], 4),
            ([P2, P3, P5, P6, P7, S1, S2, S2, S3, W, W, W, RD, RD], 0),
            ([M1, M5, M9, P4, P6, P7, P7, P7, S1, S1, S2, S2, S4, S8], 3),
            ([M4, M6, P1, P4, P5, P6, P8, P9, P9, S2, S4, S5, S6, E], 2),
            ([M3, M8, M9, P6, P7, P8, S4, S5, S6, S6, S7, S8, S8, GD], 1),
            ([M2, M6, M7, P2, P3, P3, P4, P5, P5, S4, S4, S8, WD, WD], 2),
            ([M3, M3, M4, M6, S1, S3, S3, S4, S5, S6, S7, S8, S8, S9], 1),
            ([M1, M2, M3, M6, P1, P2, P3, P5, P8, P9, S1, S3, S5, S7], 2),
            ([M1, M2, M4, M5, M6, P4, P5, P8, S1, S3, S6, WD, GD, GD], 2),
            ([M9, P2, P4, P7, P8, P8, S1, S4, S5, S8, S9, S, W, RD], 4),
            ([M1, M1, M2, M2, M4, P3, P4, P4, P5, S2, S2, S3, S6, S], 2),
            ([M3, M7, M7, P1, P3, P6, P7, S3, S5, S6, S8, S9, S, GD], 3),
            ([M5, M8, P1, P2, P2, P4, P4, P5, S3, S5, S6, S8, S8, RD], 3),
            ([M1, M1, M1, M5, M6, P3, P5, P7, P8, P9, S3, S4, WD, WD], 1),
            ([M7, M7, P2, P2, P2, P6, P6, S6, S7, S8, S8, S, S, S], 0),
            ([M1, M8, P1, P1, P3, P4, P5, S2, S3, S5, S6, S6, WD, RD], 3),
            ([M4, M5, M7, M8, M9, P2, P4, S1, S1, S3, S5, S7, S, WD], 2),
            ([M1, M7, M7, P2, P2, P3, P7, P8, P9, S2, S4, S8, S9, WD], 2),
            ([M3, M4, M5, M9, M9, P2, P3, P4, P6, P6, S2, S8, S8, RD], 1),
            ([M3, M7, P1, P4, P6, P9, P9, S6, S8, S9, S9, W, GD, RD], 4),
            ([M5, M7, M7, P1, P2, P4, P6, P6, P7, P8, P9, S1, S6, E], 3),
            ([M2, M3, M4, P1, P1, S4, S5, S7, S8, S9, WD, WD, RD, RD], 1),
            ([M2, M3, M4, M5, M6, M8, P4, P7, S5, S7, E, GD, RD, RD], 3),
            ([M6, M8, P6, P7, P8, S2, S3, S7, S9, S, N, WD, GD, RD], 3),
            ([M3, M3, M7, M9, P3, P4, P7, P8, S2, S3, S3, S4, S6, S], 2),
            ([M4, M5, P2, P3, P3, P5, P6, P6, P8, P8, S7, S8, S8, S9], 2),
            ([M6, M7, P5, P6, P7, S1, S2, S2, S3, S5, S7, S7, W, W], 1),
            ([M4, M4, M6, M8, M9, P2, P7, S6, S6, W, W, WD, GD, RD], 3),
            ([M2, M6, M7, M8, P7, P7, S1, S2, S3, S4, S4, S6, S8, S9], 1),
            ([M1, M1, M3, M5, S1, S2, S2, S3, S3, S6, S7, S7, S8, E], 1),
            ([M7, M7, P1, P1, P4, P4, P5, P5, P6, P6, P8, P8, P9, P9], -1),
            ([M3, M7, M7, M8, P1, P3, P5, P8, P9, P9, S3, S5, S7, RD], 4),
            ([M1, M4, M4, M6, P3, P4, P5, P6, P6, P8, S2, S5, S8, S8], 3),
            ([M1, M3, M7, M8, M8, P3, P7, P8, S2, S4, S9, E, S, GD], 4),
            ([M4, M5, P5, P7, P8, S1, S1, S1, S5, S6, S7, S8, S9, WD], 2),
            ([M2, M3, M3, M6, M6, P5, P6, P6, S3, S3, S9, S9, E, RD], 1),
            ([M5, M6, P4, P5, P6, S1, S1, S1, S5, S6, S7, S9, S9, W], 0),
            ([M4, M6, M6, M7, M8, P3, P4, S2, S5, S6, S7, S9, E, E], 1),
            ([M2, M6, M6, P4, P7, P7, P7, P8, S1, S4, S6, S6, W, W], 2),
            ([M2, M3, M4, M7, M8, P1, P2, P2, P5, S4, S6, E, W, W], 2),
            ([M3, M5, M5, M7, M9, P3, P3, P4, P5, P6, P7, S1, S2, S6], 2),
            ([M3, M4, M5, P9, S1, S2, S2, S4, S4, S7, S7, S9, RD, RD], 2),
            ([M1, M2, M3, M6, M8, P3, P4, P4, P6, P8, P8, S3, S6, W], 2),
            ([M6, M7, S1, S2, S3, S4, S5, S6, S6, S8, S8, S8, N, N], 0),
            ([M1, M2, M4, M5, M6, P5, P8, P8, P9, S3, S4, S6, S6, S8], 2),
            ([M1, M2, M3, M7, M8, M9, P2, P4, P6, P8, S3, S4, S7, S8], 2),
            ([M1, M2, M3, M6, M6, M7, P2, P7, P9, S2, S5, S5, W, WD], 3),
            ([M2, M6, M6, M8, M9, M9, P2, P3, P7, S2, S6, S6, GD, RD], 3),
            ([M5, M7, M7, M7, P2, P7, P8, P9, S4, S5, S7, W, W, N], 2),
            ([M3, M5, M7, M8, P3, P7, P8, P9, S1, S3, S9, S9, S9, E], 2),
            ([M1, M4, M4, M5, P4, P4, P6, P7, S6, S6, S8, E, E, WD], 2),
            ([M2, M3, M8, M9, P2, P2, P5, P5, P6, P8, S1, S5, E, GD], 3),
            ([M2, M2, M2, M6, M6, M6, M8, M8, M9, P3, P4, P6, P6, WD], 1),
            ([M1, M3, M4, M5, M7, M7, M8, P2, P8, P9, S4, S6, S7, GD], 3),
            ([M4, P2, P2, P3, P4, P5, P6, S2, S4, S6, S7, S8, WD, GD], 2),
            ([M5, M5, M6, M8, M8, P1, P4, P4, P5, P5, P7, P7, S3, S6], 1),
            ([M1, M3, M4, M5, M7, M8, M8, P2, P4, S2, S5, S6, S6, GD], 3),
            ([M2, M4, M6, M7, P1, P2, P3, P5, P5, P6, P8, W, W, W], 1),
            ([M3, P2, P3, P3, P5, P6, P8, S1, S3, S3, S4, S5, S6, S6], 2),
            ([M5, M6, M7, M8, M8, P4, P6, P6, P7, P8, S3, S4, S5, S6], 0),
            ([M2, M3, M4, P4, S1, S2, S2, S4, S5, S7, S7, S8, E, RD], 3),
            ([M1, M5, M9, P6, P7, P7, P9, S2, S3, S4, S8, S9, S, S], 2),
            ([M4, M6, M7, M9, P1, P8, P9, S2, S4, S8, S9, S, WD, GD], 4),
            ([M2, M6, M8, P2, P3, P6, P6, P8, P8, S4, S7, S, W, N], 4),
            ([M4, M4, M5, M5, M6, M6, M8, M8, P1, P6, P8, S7, S8, S9], 0),
            ([M2, M4, M4, P3, P3, P4, P4, P5, S1, S2, S3, S5, S7, S9], 1),
            ([M2, M4, M5, M5, M8, P1, P1, S1, S2, S2, S3, S5, WD, WD], 2),
            ([M1, M4, M8, M9, P2, P8, S3, S8, S9, S9, S, W, GD, RD], 5),
            ([M1, M2, M2, M3, M4, M6, M8, M8, M9, P5, P6, S1, GD, GD], 2),
            ([M1, M2, M5, M8, M9, P3, P3, P7, S1, S3, S4, E, W, W], 3),
            ([M2, M3, M3, M4, M8, M9, P4, P5, P7, S3, S4, S6, GD, GD], 2),
            ([M2, M3, M4, M7, M8, P1, P6, S2, S2, S3, S3, S3, S6, S9], 2),
            ([M2, M4, M6, M7, M7, M8, M9, P4, P8, S, WD, GD, GD, RD], 3),
            ([M4, M6, M7, M7, M7, P5, P7, P9, S1, S4, W, N, WD, RD], 4),
            ([M1, M2, M4, M5, M6, P3, P3, P5, S7, S8, E, E, W, W], 2),
            ([M1, M1, M1, M2, M4, M4, P1, P2, P5, P5, P7, P7, P8, RD], 2),
            ([M1, M2, M3, M6, M8, P3, P4, P4, P6, P8, P8, S6, S, W], 2),
            ([M1, M1, M2, M7, M8, M9, P3, P6, P7, P8, S1, S1, S3, RD], 2),
            ([M2, M3, M4, M7, M8, P1, S4, S5, S5, S7, S7, N, WD, GD], 3),
            ([M2, M4, M5, P6, P6, P7, P7, P8, S1, S2, S3, S6, S6, S], 1),
            ([P6, P7, P8, S3, S4, S6, S7, S8, S, W, W, W, GD, GD], 0),
            ([M6, M7, M8, P2, P3, P7, P7, P8, S4, S5, S6, S8, S8, WD], 1),
            ([M3, M4, M6, M7, M7, M8, M8, P1, P2, P7, P9, S6, S8, S9], 3),
            ([M6, M7, M9, P1, P3, S1, S1, S2, S4, S4, S4, S6, S7, WD], 2),
            ([M3, M4, M9, P3, P3, P5, P6, P8, P9, S1, S5, S6, E, WD], 3),
            ([M7, M7, M8, M9, M9, P4, P4, P5, P5, P8, P8, S5, S6, S9], 1),
            ([M3, M3, M9, M9, P1, P1, P6, P6, P8, P8, P8, S2, S2, S3], 0),
            ([S2, S3, S3, S4, S5, S6, S6, S7, E, WD, WD, GD, RD, RD], 2),
            ([M3, M3, M4, M4, M4, M5, M5, M6, S3, S9, S9, W, W, W], 0),
            ([M3, M4, M4, M7, M8, M9, P4, P6, P7, P7, P8, S6, S7, N], 2),
            ([M3, M5, M6, M7, M8, P3, P5, P7, P8, S2, S2, S3, S3, S5], 2),
            ([M1, M2, M4, P1, P9, S3, S6, S6, S7, S8, S, WD, GD, RD], 5),
            ([M1, M4, M5, P1, P3, P7, P8, S5, S6, S9, S, W, W, WD], 3),
            ([M1, M2, M3, M6, P3, P4, P4, S4, S5, S6, S6, S6, S, S], 1),
            ([M1, M2, M3, M8, M8, P2, P6, P7, S1, S2, S3, S4, S5, S6], 0),
            ([M1, M2, M3, M7, M9, M9, P8, S4, S4, S6, S8, S8, N, WD], 3),
            ([M2, M6, M7, M8, M8, P2, P2, P4, P6, S3, S3, S6, N, N], 2),
            ([M3, M4, M6, M6, M6, P4, P9, S2, S3, S4, S8, S8, RD, RD], 1),
            ([M3, M4, P2, P2, P2, P3, P7, P8, S3, S3, S4, S4, S5, S6], 1),
            ([M5, M6, M6, P3, P3, P4, P4, P5, S3, S5, S5, S8, S8, S8], 1),
            ([M3, M3, M6, M7, M8, P7, P8, S1, S2, S3, S5, S8, S8, S9], 1),
            ([M6, M9, P6, P9, S1, S3, S5, S8, E, S, S, W, WD, GD], 4),
            ([M3, M4, M5, P2, P3, P4, S3, S4, S5, S6, S6, W, W, N], 0),
            ([M3, M4, M5, M6, M7, M9, M9, P4, P5, S4, S8, S9, S9, WD], 2),
            ([M2, M3, M6, M6, M7, P5, P5, P5, P7, P7, S3, S4, W, W], 2),
            ([M1, M4, M4, M8, P1, P4, P6, S2, S4, S5, S5, S7, W, RD], 4),
            ([M1, M5, M6, M7, P1, P3, P4, P7, S1, S3, S4, S5, S5, S], 3),
            ([M2, M2, M3, M4, M6, M7, M7, M9, P3, P4, P4, S3, S7, S8], 2),
            ([M3, M3, M5, M7, M7, M8, M9, P2, P4, P4, S3, S6, S8, E], 2),
            ([M9, P1, P3, P4, P6, P8, S2, S4, S4, S4, S7, S8, S8, S9], 2),
            ([M2, M2, M7, M7, P4, P4, P5, P5, P7, S4, S7, W, GD, GD], 1),
            ([M8, P4, P6, S1, S3, S4, S4, S6, S7, E, S, S, N, RD], 3),
            ([M3, M4, M5, P3, P5, P6, P6, P6, S2, S3, S4, S5, S6, S7], 0),
            ([M2, M3, M4, M5, M6, P5, P7, P8, P9, S1, S2, S3, S3, S6], 1),
            ([M1, M3, M3, P1, P1, P3, P4, P5, P6, P6, P9, S7, E, E], 2),
            ([M2, M3, M4, M7, M8, M9, P4, P5, P5, P6, P7, S4, S4, S6], 0),
            ([M2, M2, M4, M5, M7, M8, P2, P4, P6, S6, S6, S7, S8, E], 2),
            ([M1, M3, M3, M7, P6, P8, P9, S1, S1, S5, S7, E, WD, RD], 4),
            ([M1, M2, M3, M7, M8, M9, P5, P5, P6, P6, P8, S1, S2, S2], 1),
            ([M4, M4, M5, M6, M7, M8, P3, P4, P6, P6, P8, S4, S5, RD], 2),
            ([M3, M4, M5, M7, M9, S2, S3, S4, S4, S7, S8, S9, S9, S9], 0),
            ([M3, M4, M7, M9, P1, P2, P4, P5, P7, P8, P9, S6, S8, S8], 2),
            ([M3, M8, P1, P3, P4, P5, P7, P8, S3, S6, S7, S8, E, RD], 3),
            ([M4, M7, P1, P2, P2, P3, P3, S1, S2, S5, S6, S8, S, W], 3),
            ([M2, M4, M4, P3, P4, P4, P7, P8, S1, S3, S3, S4, S8, S8], 2),
            ([M1, M2, M3, M7, P1, P2, P3, P4, P6, P8, S1, S2, S8, S8], 1),
            ([M2, M8, M9, P3, P4, P7, P9, S2, S5, S8, W, W, WD, RD], 4),
            ([M1, M2, M3, M7, P3, P3, P5, P5, S1, S2, S3, S8, S8, S9], 1),
            ([M6, M7, M7, P1, P3, S2, S2, S3, S3, S4, S5, S7, S9, GD], 2),
            ([M2, M3, M4, P4, P4, P6, P8, S1, S2, S2, S3, S4, S4, S6], 1),
            ([M3, M4, M6, M7, P4, P4, P5, P5, P6, S2, S4, S6, S8, W], 3),
            ([M1, M2, M4, P1, P3, P4, P8, P9, P9, S1, S6, N, WD, GD], 5),
            ([M2, M3, M4, M7, M8, P4, P5, P6, S2, S3, S3, S4, S4, S5], 0),
            ([M3, M4, M6, M7, M8, P3, P4, P7, S3, S3, S6, S6, W, WD], 2),
            ([M2, M3, M5, M8, M8, P5, P5, P6, P6, P8, S1, S1, S2, S2], 1),
            ([M2, M9, P1, P2, P2, P3, P7, P7, P9, P9, S3, S3, S6, N], 2),
            ([M2, M2, M4, M5, M5, M7, M8, M9, P2, P3, P6, P7, P8, S3], 1),
            ([M1, M5, S1, S2, S4, S6, S9, S9, S, WD, WD, WD, GD, GD], 2),
            ([M2, M4, M7, M7, P5, P8, P8, S7, S8, S8, S9, S, N, N], 2),
            ([M5, M7, P1, P2, P4, P6, P6, P6, P9, S5, S5, S7, S7, W], 2),
            ([M2, M2, M2, M4, M7, P6, P8, S2, S4, S5, S6, S7, S7, RD], 2),
            ([M1, M3, M4, M7, M8, P1, P3, P7, P8, E, S, N, WD, RD], 4),
            ([M1, M5, M7, M8, M9, P4, P4, P5, P9, S3, S8, S9, S9, GD], 4),
            ([M1, M1, M5, M8, M8, P3, P3, P5, P8, S1, S1, S2, S6, S7], 2),
            ([M5, M6, M6, P1, P2, P3, P5, S3, S3, S5, S6, S6, S7, S7], 1),
            ([P4, P5, P6, P7, S2, S2, S3, S3, S3, S4, S5, S6, S8, S9], 0),
            ([M5, M5, M6, M7, M7, P1, P3, P4, P9, S4, S4, S7, S8, S9], 1),
            ([M1, M2, M3, M6, P2, P2, P2, P6, P7, S1, S2, S9, S9, WD], 1),
            ([M4, M4, M4, M6, M7, M7, P4, P5, P6, S6, S7, S7, S8, S9], 0),
            ([M1, M3, M7, M8, M9, M9, P9, S1, S1, S2, S2, S9, WD, GD], 3),
            ([M2, M3, M5, M8, M8, P4, P6, P6, P7, S4, S5, S5, S6, S7], 2),
            ([M1, M2, M3, P1, P2, P2, P7, P9, S1, S2, S5, S9, WD, WD], 2),
            ([M4, M5, P1, P6, P6, P8, S1, S2, S3, S4, S5, S7, S8, RD], 2),
            ([M3, M3, M7, M9, P2, P2, P3, P3, P4, P4, P8, S2, S4, S6], 1),
            ([M2, M3, P2, P4, P7, P9, P9, S1, S5, S6, S, WD, WD, RD], 3),
            ([M2, M4, M6, M7, M7, M9, P1, P2, S2, S5, E, E, S, RD], 3),
            ([M2, M4, M6, P1, P1, P3, P5, P5, P5, S1, S7, S8, S, S], 2),
            ([M4, M4, M5, M6, M7, M8, M8, P1, P4, P7, S3, S7, GD, GD], 3),
            ([M2, M3, M4, M6, M7, M9, M9, S3, S4, S5, S5, S6, S7, S8], 0),
            ([M5, M6, M7, M9, P2, P6, P8, S3, S5, S8, S8, GD, GD, RD], 2),
            ([M1, M1, M2, M2, M3, M6, P5, P5, P7, P7, P8, P8, S, S], 0),
            ([M1, M3, M3, M6, M7, M7, P4, P6, P8, S7, S9, S9, N, RD], 3),
            ([M1, M6, M8, P3, P4, P6, P6, S1, S1, S3, S6, WD, GD, RD], 4),
            ([M2, M3, M4, P1, P4, P4, P6, P8, S2, S2, S3, S4, S4, S6], 1),
            ([M1, M3, M4, M6, M6, M7, P2, P6, P7, S3, S6, S6, S6, W], 3),
            ([M4, M9, M9, P1, P4, P4, P5, P7, S1, S2, S3, S7, W, W], 2),
            ([M3, M4, M7, M8, M9, P3, P4, P5, S3, S4, S7, S9, E, E], 1),
            ([M1, M1, M2, M3, M4, M4, M5, P1, P2, P7, P7, P8, S1, S3], 2),
            ([M1, M2, M4, P1, P1, P5, P7, S2, S4, S4, S5, S6, E, S], 2),
            ([M3, M4, M6, M6, M7, M8, M9, P2, P3, P4, P5, P6, P7, RD], 0),
            ([M6, M8, P3, P4, P5, P7, P8, P9, P9, S5, S6, E, N, N], 1),
            ([M1, M3, M5, M6, M8, M8, P1, P2, P3, P9, S4, S5, S7, S8], 2),
            ([M2, P1, P2, P7, P8, S5, S6, S7, S7, S8, N, N, GD, RD], 2),
            ([M4, P1, P3, P3, P4, P5, S1, S2, S2, S2, S5, S5, S6, S7], 1),
            ([M8, P3, P5, P7, P7, P8, S1, S2, S3, S4, S4, S4, S5, W], 2),
            ([M2, M4, M4, M6, M7, M8, P1, P3, P5, P5, S6, S6, S7, S7], 2),
            ([M1, M2, M3, M4, P3, P4, P6, P7, P8, S2, S2, S3, S4, S5], 0),
            ([M3, M3, P1, P2, P2, P3, P3, P6, P6, S3, S3, S4, S6, S6], 0),
            ([M3, M6, P3, P4, P4, P5, P6, P8, S3, S4, S5, S9, S, S], 2),
            ([M6, M8, M8, P1, P2, P2, P7, S1, S3, S5, S6, S6, S6, S7], 2),
            ([M2, M3, M4, P4, P5, P6, P7, P9, P9, S3, S3, S4, S7, S8], 1),
            ([M7, M7, P4, P6, P7, P8, S2, S2, S7, S7, S9, E, E, E], 1),
            ([M4, M4, M4, M6, M8, P3, P4, P5, P5, P6, P7, S3, S3, S5], 0),
            ([M2, M4, M6, M6, P1, P2, P3, P5, P6, P7, S2, S4, S6, S8], 1),
            ([M3, M5, P1, P1, P3, P3, P4, P8, S3, S4, S5, S7, S8, S], 2),
            ([M2, M3, M3, M4, M7, M8, M8, P2, P5, S4, S4, S6, S6, S6], 2),
            ([M1, M2, M2, M2, M4, M5, M6, P2, P4, P6, P7, S2, S2, S2], 1),
            ([M1, M1, M5, M6, P7, P8, P9, S1, S2, S3, W, RD, RD, RD], 0),
            ([M1, M3, M5, P1, P2, P7, P7, P9, S2, S5, S5, S6, S9, W], 4),
            ([M2, M3, M3, M3, M4, P3, P4, S4, S5, S5, S6, S6, S7, S9], 0),
            ([M2, M4, M6, M7, M8, M8, P2, P2, P4, P4, P5, S3, S3, GD], 2),
            ([M6, M7, P2, P2, P4, P4, P5, P7, P7, S2, S3, S8, N, N], 2),
            ([M6, M7, M8, P1, P7, P7, S1, S2, S3, S4, S4, S6, S8, S9], 1),
            ([M3, M6, P4, P6, P6, P7, P8, P8, S4, S5, S5, S8, N, N], 2),
            ([M2, M3, M3, M4, M4, M6, M6, P8, S2, S3, S4, S6, S7, S8], 0),
            ([M6, P4, P4, P5, P6, P6, S2, S4, S5, S7, N, N, N, GD], 2),
            ([M3, M4, M7, M8, P2, P5, P8, P8, P9, P9, S4, S4, S6, S8], 3),
            ([M3, M4, M5, M6, M8, M9, P4, S2, S4, S5, S7, E, S, S], 2),
            ([M3, M3, M3, P1, P2, P3, P5, P5, S2, S3, S4, S6, S7, S7], 0),
            ([M3, M7, M9, P3, P6, P8, P9, S1, S3, S8, E, S, WD, GD], 5),
            ([M3, M4, M5, M6, M7, M8, P2, P3, P4, P6, S2, S7, S8, S9], 0),
            ([M2, M2, M4, P2, P3, P4, P4, P6, P8, S4, S5, S7, S7, S8], 2),
            ([M1, M2, M7, M8, P5, P5, P6, P6, S8, S8, S9, RD, RD, RD], 2),
            ([M1, M2, M3, M6, M6, M7, M8, M9, S4, S5, S6, S7, S8, S9], -1),
            ([M2, M5, M5, M7, M7, M9, M9, P4, P6, P8, S4, S8, W, N], 3),
            ([M1, M2, M3, P5, P5, P8, P8, P8, S2, S3, S4, WD, WD, WD], -1),
            ([M2, M4, M7, M8, M9, P3, P5, S3, S6, S7, S7, S9, GD, GD], 2),
            ([M2, M4, M4, M5, M8, M9, P2, P6, S3, S5, S7, S8, S9, GD], 3),
            ([M2, M3, M3, M4, M6, M7, M9, M9, S3, S4, S5, S6, S7, S8], 0),
            ([M5, M7, M9, P3, P6, P8, S3, S3, S4, S5, S6, S7, S9, E], 2),
            ([M1, M6, M6, P1, P2, P6, P8, S1, S2, S8, E, W, WD, RD], 4),
            ([M4, M5, M7, M8, M8, P1, S5, S6, S6, S8, S8, S9, E, RD], 3),
            ([M3, M3, M5, P3, P4, P6, P7, P7, P7, S6, S8, N, N, RD], 2),
            ([M2, M3, M3, M8, P4, P5, P6, P6, P7, S1, S1, S3, S5, WD], 2),
            ([M2, M3, M5, M7, M7, P5, S4, S5, W, WD, WD, RD, RD, RD], 2),
            ([M1, M2, M3, M6, M9, P3, P4, P4, S4, S5, S6, S6, S6, S], 2),
            ([M2, M5, M6, M6, M8, P2, P3, P4, P6, S2, S4, S5, S7, W], 3),
            ([M2, M2, M3, P4, P5, P6, S5, S5, S7, S7, S7, S8, N, N], 1),
            ([M2, M3, M3, M5, M7, P1, P3, P5, P7, P8, S2, S5, S8, W], 4),
            ([M2, M2, M5, M9, P7, P9, S4, S6, S8, S8, E, W, N, GD], 4),
            ([M4, M5, P1, P2, P3, P4, P5, P7, P8, P9, S2, S5, S5, S9], 1),
            ([M3, M4, M5, M8, M9, P1, P2, P6, P8, P8, S2, S4, S6, S9], 2),
            ([M2, M5, M6, P1, P2, P2, P4, S1, S2, S3, E, S, S, GD], 2),
            ([M3, M5, M5, M7, M8, P4, P5, P6, P7, S1, S2, S4, S5, RD], 2),
            ([M2, M3, M4, M8, P1, S4, S5, S7, S8, S9, WD, WD, RD, RD], 1),
            ([M3, M4, M5, M9, P4, P4, P5, P6, S7, S8, S9, N, N, N], 0),
            ([M3, M4, M5, P1, P3, P5, P6, P6, P7, S1, S2, S3, GD, GD], 0),
            ([M2, M6, P4, P5, P5, P6, P7, S1, S6, S7, E, S, N, N], 3),
            ([M2, M3, M8, M9, P4, P4, P8, P9, S2, S3, S5, S7, S9, N], 3),
            ([S3, S4, S5, S5, S6, S6, S6, S8, S9, S9, S, S, WD, WD], 1),
            ([M1, M2, M3, M6, M7, P4, P8, S1, S3, S3, S5, S6, S7, S7], 2),
            ([M2, M4, M4, M5, M5, M7, M9, P6, S2, S5, S6, S7, S, N], 3),
            ([M3, M5, M5, M8, M9, P1, P5, P6, P6, P8, P8, P8, GD, RD], 3),
            ([M2, M2, M7, P3, P4, P5, P7, P7, S1, S1, S7, S8, RD, RD], 2),
            ([M2, M2, M7, M8, M9, P5, P5, P7, P8, P9, S1, S2, S8, E], 1),
            ([M7, M8, P1, P4, S3, S5, S6, S9, E, S, W, WD, RD, RD], 5),
            ([M1, M8, M9, M9, M9, P1, P2, P4, P4, P7, S1, S4, S5, S6], 2),
            ([M3, M4, P2, P7, P9, S4, S5, E, E, E, S, W, W, W], 2),
            ([M1, M5, M5, M6, M7, M7, M9, P2, P4, P6, S2, S2, WD, WD], 2),
            ([M3, M4, M5, P4, P5, P6, P7, S1, S1, S1, S3, S4, N, N], 0),
            ([M1, M8, M9, M9, M9, P4, P4, P5, P5, P6, P6, S2, S4, S8], 1),
            ([M4, M7, M8, M8, P2, P2, P5, P6, P7, P7, P8, S8, E, WD], 3),
            ([M1, M6, P1, P6, P6, P7, P8, S2, S3, S6, S9, S9, E, RD], 4),
            ([M2, M3, M3, M4, M5, P5, P6, P7, P7, S2, S5, S5, S, S], 1),
            ([M1, M1, M1, M2, M5, P3, P3, P5, S2, S6, E, E, E, N], 3),
            ([M1, M3, M3, M8, M9, P4, P5, P5, P9, S4, S6, S7, W, WD], 4),
            ([M2, M4, M5, M5, M9, P2, P3, P5, P8, P8, S6, S6, S7, GD], 3),
            ([M1, M3, M8, M9, P3, P7, P8, S5, S7, S8, E, WD, RD, RD], 3),
            ([M2, M5, M9, M9, P4, S1, S1, S2, S3, S4, S5, S6, S7, RD], 2),
            ([M4, P1, P3, P4, P5, P8, P9, S2, S3, S5, S5, S6, N, N], 2),
            ([M4, M5, M6, M6, P4, P5, P5, P6, P7, P7, P7, S3, S5, S5], 1),
            ([M2, M3, M4, P3, P4, P5, P5, P6, P7, S4, S4, S4, S5, S5], -1),
            ([M1, M2, M3, M4, M6, M7, M8, P3, P3, P4, P6, S7, S8, S9], 0),
            ([M2, M3, M4, M5, M5, M7, P3, P7, S4, S4, S5, S7, GD, GD], 2),
            ([M1, M2, M3, M4, M5, M8, M9, S2, S, W, N, N, N, GD], 2),
            ([M1, M2, M3, M7, M9, P1, P5, P6, P9, S2, S4, S7, S7, GD], 2),
            ([M1, M1, M2, M9, P2, P6, P6, S1, S5, S6, S7, S7, S9, E], 3),
            ([M6, M8, P2, P3, P7, P8, S1, S2, S3, S3, S5, S5, S7, S8], 2),
            ([M3, M5, P1, P1, P2, P3, P4, P6, P6, S4, S6, S7, S9, S], 2),
            ([M4, M6, M8, P2, P7, P9, S3, S4, S5, S6, S7, S8, S8, S9], 2),
            ([M4, M5, M6, M8, M9, P2, P3, P4, P6, P8, P8, S5, S6, S7], 0),
            ([M5, M7, M8, P5, P5, P6, P6, P8, S8, S8, S9, RD, RD, RD], 2),
            ([M2, M3, M4, M6, M7, M8, P3, P3, P4, P5, S3, S3, WD, WD], 0),
            ([M2, M3, M8, P2, P3, P4, P4, P5, P7, P8, P9, S1, S2, S5], 2),
            ([M1, M3, M5, M6, M7, M8, P5, P6, P8, P8, S1, S2, S3, S4], 1),
            ([M4, M5, M6, P2, P4, P4, P5, P6, S2, S2, S6, S7, S8, GD], 0),
            ([M2, M3, M4, P1, P2, P3, P3, P4, P5, P7, S1, S1, S7, S8], 0),
            ([M4, M4, M5, M6, M7, M8, P4, P5, P5, S3, S4, S5, GD, GD], 1),
            ([M2, M9, P4, P7, P8, P8, P9, S6, S9, E, S, W, N, GD], 5),
            ([M2, M6, M8, M9, P7, P8, P9, S1, S3, S7, S9, E, WD, RD], 3),
            ([M3, M4, M5, M7, M7, P1, P3, P3, P3, S3, S4, S5, S6, S6], 0),
            ([M3, M6, M6, M9, P1, P5, P7, P7, P7, P9, S8, S9, S, S], 3),
            ([M2, M4, M6, M6, M7, P6, P6, P7, S2, S3, S7, S9, W, W], 3),
            ([M1, M3, M7, M7, M9, P3, P5, P8, S2, S3, S6, S7, S, GD], 3),
            ([M1, M3, M5, M8, M8, P3, P4, P5, P6, P7, P8, S3, S4, S6], 1),
            ([M1, M1, M2, M2, M3, P2, P2, P6, P7, P7, P8, S2, S7, S8], 1),
            ([M6, P1, P1, P2, S6, S7, S7, S8, S9, E, E, E, N, N], 1),
            ([M1, M2, M3, M4, M5, M5, M6, S1, S2, S3, S4, S5, S9, S9], 0),
            ([M3, M7, M9, P6, P7, P8, S1, S2, S3, S4, S8, GD, RD, RD], 2),
            ([M1, M2, M4, M5, M6, P5, P6, P7, S2, S3, S6, E, GD, RD], 2),
            ([M2, M4, P1, P1, P2, P3, P3, P8, P9, S4, S5, S6, S, S], 1),
            ([M6, M6, P1, P2, P3, P6, P7, S2, S3, S4, S4, S7, S7, S8], 1),
            ([M5, M6, P6, P8, S2, S3, S6, S8, S8, S9, S9, S9, S, W], 2),
            ([M1, M2, M2, M7, M8, M8, M9, P4, P8, S3, S5, WD, RD, RD], 3),
            ([M5, M6, M6, M8, P2, P3, P4, P6, S2, S4, S5, S7, S8, W], 3),
            ([M1, M2, M2, M3, M7, M7, M8, M8, P4, P5, P6, S2, W, W], 1),
            ([M2, M4, M6, M7, M8, M9, M9, P8, P9, S4, S5, S6, S6, S7], 1),
            ([P1, P4, P5, P6, P7, P8, P8, S6, S8, E, S, W, GD, RD], 4),
            ([M3, M4, M5, P3, P4, P5, P8, P8, P8, S3, S5, S8, S8, E], 0),
            ([M2, M6, M6, M7, M9, P1, S3, S3, S3, S4, S5, S7, E, N], 3),
            ([M2, M4, M5, M5, P1, P7, P8, P9, S1, S1, S3, S5, S8, E], 2),
            ([M5, M6, M7, M8, M9, P1, P2, P3, P6, P8, S3, S7, S, RD], 2),
            ([M6, M8, P3, P4, P5, P6, P6, P7, P8, P8, S4, S6, S9, N], 2),
            ([M5, M6, P1, P4, P6, P7, P7, P8, P8, P8, S1, S1, S7, S], 2),
            ([M1, M2, M4, M5, M6, P6, P7, P9, S1, S3, S4, S6, RD, RD], 2),
            ([M2, M4, M5, M7, M7, M9, P2, P3, P7, S1, S2, S9, RD, RD], 3),
            ([M2, M6, M8, M9, M9, P2, P3, P6, P7, S3, S4, S7, S7, S8], 3),
            ([M2, M3, M8, M9, M9, P1, P5, P8, P9, S4, S4, S5, S6, S6], 2),
            ([M4, M9, M9, P3, P6, P7, P7, P8, S3, S3, S5, S6, S9, GD], 3),
            ([M3, M4, P4, P5, P6, P7, P9, S3, S3, S5, E, N, WD, WD], 2),
            ([M6, M6, M7, M7, P5, P8, P8, P9, S1, S1, S2, S2, S3, S9], 1),
            ([M1, M2, M7, M7, M7, P3, P5, P9, S5, S7, E, WD, GD, RD], 3),
            ([M2, M3, M4, P4, P5, P5, P5, P6, P8, P9, S3, S3, S5, S6], 1),
            ([M2, M2, M7, M7, M8, P7, P7, S3, S3, S5, S5, S8, W, WD], 1),
            ([M7, M8, M9, M9, P1, P2, P2, P7, P9, S8, S9, WD, RD, RD], 2),
            ([M1, M3, M5, P1, P3, P8, S6, S8, S, N, WD, WD, GD, RD], 4),
            ([M2, M2, M3, P1, P3, P8, S1, S7, S8, S9, S, WD, WD, RD], 3),
            ([M3, M4, M6, M7, P2, P6, P7, P8, S2, S3, S4, S8, S8, S9], 1),
            ([M4, M5, M8, M8, P2, P2, P5, P7, S1, S2, S3, S9, GD, GD], 2),
            ([M2, M6, P1, P2, P2, P4, P8, S1, S2, S3, E, S, GD, RD], 4),
            ([M1, M1, M5, P2, P2, P4, P6, P6, P8, S1, S5, S7, GD, RD], 3),
            ([M3, M4, M4, M5, M5, P5, P6, S2, S2, S4, S6, S6, S7, N], 2),
            ([M5, M7, M9, P1, P2, P4, P6, P9, S2, S6, S8, S, S, W], 3),
            ([M7, M9, M9, P2, P5, P6, P8, P9, S3, S4, E, WD, WD, GD], 3),
            ([M2, M3, M5, M7, M9, S4, S5, S6, S6, S7, S8, S8, S8, GD], 1),
            ([M3, M4, M6, M7, P5, P9, S2, S2, S4, S7, S8, S, W, WD], 4),
            ([M5, M5, M7, M7, M8, P4, P6, S2, S3, S3, S9, S, WD, GD], 3),
            ([M2, M4, M5, M6, M7, P3, P4, P5, S2, S4, S4, S6, S, S], 1),
            ([M5, M5, M6, P3, P4, S2, S2, S3, S3, S4, S4, S6, W, GD], 2),
            ([M3, M8, M9, P2, P3, P4, P5, P6, P8, P8, P9, S4, S5, RD], 2),
            ([M2, M3, M8, M9, P1, P3, P7, P7, P8, S2, S3, S4, S5, W], 2),
            ([M2, P2, P6, P8, S2, S3, S5, S6, S6, S7, E, WD, WD, GD], 3),
            ([M2, M4, M6, P1, P1, P2, P3, P3, S4, S5, S6, S6, S, S], 1),
            ([M2, M4, M7, M8, M9, P2, P3, P8, P8, P8, S1, S2, S3, WD], 1),
            ([M6, M8, P1, P1, P6, P7, P8, S2, S3, S4, S5, S6, S7, S9], 0),
            ([M4, M7, M8, P6, P7, P8, P9, S3, S4, S5, S7, S8, WD, GD], 2),
            ([M4, M6, M8, P5, P6, P6, S1, S1, S2, S3, S3, S5, S7, S8], 2),
            ([M5, M8, P8, S2, S2, S2, S3, S3, S4, S5, S7, S8, WD, GD], 3),
            ([M1, M2, M3, M5, P5, S2, S4, S5, S7, S7, S8, S8, S9, S9], 1),
            ([M3, M4, M4, M6, M8, P6, S5, S6, S6, S7, S8, S9, S, S], 2),
            ([M2, M2, M2, M4, M4, M4, M6, M6, S1, S2, S3, S3, S4, W], 0),
            ([M2, M5, M9, P4, P6, P9, S2, S2, S5, S7, E, W, WD, RD], 5),
            ([M3, M4, M5, M5, M7, M7, M8, M8, S4, S5, S6, S6, S7, E], 1),
            ([M3, M5, M8, M9, M9, P3, P6, S1, S3, W, N, GD, GD, RD], 4),
            ([M3, M5, M6, M8, M9, P4, P5, P6, P7, S2, S5, E, W, W], 3),
            ([M4, M5, M5, M6, M7, P3, P5, P7, P7, S3, S5, S6, S7, E], 1),
            ([M3, M4, M5, M5, M8, M9, P2, P3, P5, P6, S2, S4, S5, S6], 2),
            ([M3, M5, M7, M8, P5, P6, P6, P8, P8, S7, S8, S9, S9, GD], 2),
            ([M2, M3, M6, M7, P1, P1, P1, P8, S1, S1, S2, S2, S3, S3], 1),
            ([M1, M2, M2, M5, M8, M9, P6, P7, P8, P9, S2, S6, E, GD], 4),
            ([M1, M4, M5, M7, M9, P1, P1, P2, P4, P6, P6, S7, S7, S9], 3),
            ([M2, M3, M3, M4, M7, M7, M8, M9, M9, P2, P2, P5, S5, S8], 2),
            ([M3, M3, M6, M6, M6, P3, P4, P5, P6, S3, S4, S5, S6, S7], 0),
            ([M4, M5, M6, M6, M7, M8, P6, P8, S1, S2, S3, S4, S4, S8], 0),
            ([M1, M2, M5, M6, M8, P5, P5, P7, P7, P8, P8, S6, S, W], 3),
            ([M5, M9, P1, P2, P3, P6, S1, S3, S4, S6, S7, S, W, RD], 4),
            ([M3, M4, M5, P1, P2, P3, P3, P7, P8, S4, S4, S7, S8, WD], 1),
            ([M2, M5, M5, M6, M7, M7, P1, P6, P8, P8, S7, E, N, GD], 3),
            ([M2, M5, M6, M6, M6, M9, P2, P4, P5, S8, GD, GD, GD, RD], 3),
            ([M1, M3, M6, M7, M9, P3, P3, P4, P9, S1, S4, S6, S6, WD], 4),
            ([M2, M3, M4, P2, P2, P4, P4, P5, P8, P8, P8, S5, S6, W], 1),
            ([M1, M2, M2, M3, M5, M6, M9, P6, P6, S7, S8, S, S, W], 2),
            ([M8, M8, M9, P1, P2, P3, P3, P4, P8, S4, S5, WD, WD, GD], 2),
            ([M2, M3, M6, M7, P6, P7, S2, S3, S3, S4, S4, S8, S8, W], 2),
            ([M7, M7, P1, P2, P3, P5, P5, P6, S7, S7, S, W, N, WD], 3),
            ([M1, M2, M7, M8, P3, P5, P6, P7, P8, S6, S8, S9, E, E], 2),
            ([M5, M7, M8, P7, P8, P9, S1, S2, S2, S5, S9, W, W, GD], 3),
            ([M1, M5, M5, P3, P4, P6, S1, S2, S4, S9, W, WD, WD, RD], 4),
            ([M1, M2, M2, M6, P2, P7, P7, S2, S3, S4, S6, E, N, RD], 4),
            ([M2, M4, M5, M6, M8, P1, P3, P4, S2, S2, S5, S7, S, WD], 3),
            ([M1, M1, M3, M7, P5, P6, P8, P8, S3, S5, E, GD, GD, RD], 3),
            ([M5, M5, M6, P2, P2, S3, S3, S4, S4, S6, S6, S8, S8, S9], 0),
            ([M1, M1, M2, M4, M5, P2, P2, P2, S2, S3, S4, S7, S8, S9], 0),
            ([M3, M6, M9, P1, P4, P5, P8, P9, S1, S3, S7, S9, GD, GD], 3),
            ([M2, M3, M4, M5, M6, M7, P1, P4, P6, P8, P8, S1, S8, GD], 2),
            ([P2, P2, P3, P4, P4, P5, P6, P6, P8, S4, S5, W, W, W], 1),
            ([M1, M4, P2, P9, S4, S5, S8, E, E, S, W, W, WD, RD], 4),
            ([M1, M1, M3, M6, M8, P3, P5, P5, P5, S7, S8, S, WD, RD], 3),
            ([M3, M9, M9, P1, P5, P7, P7, P9, S4, S5, S6, S6, S6, S8], 2),
            ([M8, S1, S3, S5, S6, S7, S9, E, S, W, WD, GD, RD, RD], 4),
            ([M1, M2, M3, M4, M5, M5, M6, P6, P7, P8, S3, S4, RD, RD], 0),
            ([M4, P2, P7, P9, S1, S3, S4, S5, S8, S8, S9, S, N, GD], 4),
            ([M5, M7, P1, P4, P6, P6, P6, P9, S5, S5, S9, W, W, GD], 3),
            ([M5, M5, M8, P2, P3, P5, P5, S4, S5, S6, S6, S6, S8, E], 2),
            ([M1, M3, M6, P5, P7, S3, S5, S7, S9, S9, E, E, WD, RD], 3),
            ([M1, M1, M1, M5, M7, P4, P5, P5, S4, S5, S8, S8, RD, RD], 2),
            ([M3, M4, M6, M7, M8, P2, P4, P4, P8, S3, S6, S8, GD, RD], 3),
            ([M1, M2, M8, M9, P4, P5, S3, S4, S9, E, S, WD, GD, RD], 4),
            ([M1, M2, M8, M8, M9, P2, P3, P9, S1, S2, S3, S6, S8, RD], 2),
            ([M1, M1, M2, M2, M3, P5, P7, S1, S2, S3, S4, N, WD, WD], 1),
            ([M1, M1, M3, M7, M7, M9, M9, P5, P6, P6, P8, S5, E, W], 2),
            ([M3, M3, M9, P3, P5, P7, P7, P8, S2, S6, S7, S, S, S], 2),
            ([M1, M1, M2, M6, M8, P3, P5, P7, P7, P8, P9, S6, S7, S7], 2),
            ([M5, M6, M9, P6, P8, P9, S2, S4, S6, S9, E, S, W, WD], 5),
            ([M2, M2, M4, M4, M7, M8, M8, P6, P6, P9, S4, S5, N, WD], 2),
            ([M4, M4, M6, P1, S1, S3, S5, S5, S6, S7, S7, S8, E, E], 2),
            ([M4, M5, M6, M7, M8, P8, P9, S6, S7, S9, E, E, N, N], 2),
            ([M6, M7, M7, P4, P4, P6, P6, P7, P7, P8, S7, S7, S8, S8], 0),
            ([M2, M4, M7, M8, M8, M9, P2, P7, S2, S9, S9, S, N, GD], 4),
            ([M1, M2, M7, M8, M9, P2, P4, S1, S2, S4, S5, S9, W, RD], 3),
            ([M4, M5, M5, P1, P2, P3, P3, P6, P7, S5, S5, S8, S9, S9], 2),
            ([M2, M2, M4, P5, P6, P7, P7, P8, P9, S3, S4, S5, S9, S9], 0),
            ([M4, M6, M6, P1, P6, P7, P8, P9, P9, S2, S2, S3, N, N], 2),
            ([M4, M5, M6, M7, M8, M8, M9, P1, P2, P3, P4, P4, S3, S4], 0),
            ([M2, M3, M3, M8, P1, P1, P2, S2, S2, S2, S4, S4, E, WD], 2),
            ([M1, M4, M7, M9, P4, P7, P7, P8, S1, S2, S3, S5, W, W], 3),
            ([M4, M8, M9, M9, P5, P7, S4, S4, S, S, WD, GD, RD, RD], 2),
            ([M2, M4, P5, P5, P6, P7, P7, P8, P9, S5, S6, S7, S8, S], 1),
            ([M4, M5, M7, M8, M8, P8, S4, S5, S6, S6, S8, S8, E, RD], 3),
            ([M6, P3, P4, P4, P5, P6, S3, S5, S8, S9, S9, S, S, RD], 2),
            ([M3, M4, M5, M5, M5, M6, M7, M7, P1, P2, P3, S4, S5, S6], 0),
            ([M5, M6, M7, M7, P6, P6, P6, S2, S4, S6, S6, E, N, N], 1),
            ([M2, M2, P4, P4, P6, P8, S4, S5, S6, S8, S8, W, WD, WD], 2),
            ([M1, M2, M9, P2, P3, P5, P5, S2, S3, S8, S8, S8, E, S], 2),
            ([M7, M9, P1, P1, P2, P3, P3, P5, P9, S4, S7, S8, S8, S], 3),
            ([M6, M8, P5, P6, P6, P7, P8, S1, S2, S3, S6, S6, S8, S8], 1),
            ([M4, M6, P1, P2, P3, P4, S2, S3, S4, S5, S5, S6, S7, S8], 0),
            ([M1, M1, M3, M3, M5, M8, M9, P5, P6, S3, S4, S8, S9, S9], 3),
            ([M1, M3, M4, M4, M5, M7, M8, M9, S2, S3, S4, S6, S8, E], 1),
            ([M1, M1, M3, M8, P8, P8, S2, S3, S4, S6, S8, E, RD, RD], 2),
            ([M5, M5, M6, M8, M8, P3, P6, S3, S4, S5, S6, S8, W, N], 3),
            ([M3, M5, M8, M9, P1, P2, P3, P4, P5, P7, S1, S7, E, WD], 3),
            ([M3, M4, M5, M5, M6, P3, P4, S3, S4, S5, S6, S6, S8, E], 1),
            ([M4, M4, M5, M7, M7, M8, P1, P2, P3, P7, S8, W, WD, GD], 3),
            ([M3, M7, M9, P1, P1, P2, P3, P5, S2, S4, S7, N, N, RD], 3),
            ([M1, M1, M2, M3, M5, M6, M8, M8, P2, P3, P5, P8, S7, E], 3),
            ([M1, M5, M9, P3, P6, P8, P9, P9, S1, S1, S4, E, GD, RD], 4),
            ([M4, M5, M6, M7, M7, P2, P3, P4, S2, S3, S6, S6, S7, S8], 0),
            ([M5, P1, P1, P2, P5, P8, S5, S8, S8, S9, E, WD, GD, RD], 4),
            ([M3, M6, M7, P6, P7, P9, P9, S2, S3, S5, S7, E, E, S], 3),
            ([M1, M1, M3, M5, M8, M8, P3, P5, P6, P7, P9, S2, S7, S8], 2),
            ([M5, M5, M6, M6, P3, P6, P7, S1, S2, S3, S3, S6, S9, S9], 2),
            ([M1, M9, M9, P2, P6, P9, S3, S4, S4, S5, S6, S7, W, RD], 4),
            ([M5, M7, P1, P3, P5, P5, S5, S7, S9, E, N, WD, GD, GD], 3),
            ([M1, M6, M7, M7, M7, M8, P3, P4, P6, P7, P8, S5, S6, S7], 0),
            ([M3, M3, M7, M8, M9, P2, P4, P5, P6, P7, P9, P9, S5, S8], 1),
            ([M1, M9, M9, P1, P3, P3, P6, P6, P9, P9, S4, S6, S8, W], 2),
            ([M4, M4, M5, M7, P3, P4, P4, P7, P9, P9, S3, S3, GD, GD], 1),
            ([M2, M2, M5, M6, M8, P6, P7, P8, S4, S5, E, WD, WD, WD], 1),
            ([M1, M2, M3, M5, M6, M7, M7, M8, M9, M9, P1, P2, S4, S5], 1),
            ([M3, M4, M4, M5, M6, P1, P2, P2, P7, P8, S7, S9, S9, S9], 1),
            ([M4, M5, M6, M7, M7, M8, P3, P4, P6, P7, S1, S3, S5, GD], 2),
            ([M1, M2, M3, M3, M7, M8, P5, P9, S1, S1, S2, S5, S7, S8], 3),
            ([M2, M4, P4, P4, P6, P7, P8, S1, S2, S3, W, GD, GD, GD], 0),
            ([M1, M6, P2, P2, P6, P8, P9, S1, S2, S3, S8, W, WD, RD], 4),
            ([M1, M2, M3, M6, M7, M9, M9, P7, P8, P8, S5, S, WD, RD], 3),
            ([M1, M2, M2, M3, M3, P9, P9, P9, S1, S5, S6, S6, S7, S8], 1),
            ([M4, M9, P2, P2, P5, P5, S1, S3, S3, S4, E, S, WD, WD], 2),
            ([M2, M2, M4, M6, P2, P4, P5, P7, P8, S3, S4, S5, S9, S9], 2),
            ([M2, M3, M4, M5, M7, M9, P4, P6, P7, P8, P9, S2, S2, S8], 1),
            ([M4, M5, M6, M7, M8, M9, P4, P5, P6, P9, P9, S1, S2, S3], -1),
            ([M1, M4, M5, M5, M8, M9, P4, P4, P5, P8, S4, S5, S7, S], 4),
            ([M5, M5, M7, M8, M8, P3, P4, P6, P6, S4, S5, S5, N, N], 1),
            ([M2, M4, M4, P3, P3, P5, P5, P6, P7, P8, S1, S2, S3, E], 1),
            ([M5, M8, P4, P6, P8, P9, P9, S5, S6, S8, S8, S9, N, N], 3),
            ([M3, M4, M4, M9, M9, P6, P9, S1, S1, S2, S3, S5, S6, S], 3),
            ([M4, M5, P1, P2, P2, P5, P6, S1, S2, S2, S3, S4, S5, S7], 2),
            ([M5, M5, M7, M8, M9, P2, P3, P6, S1, S4, S5, S7, S8, S9], 1),
            ([M3, M3, M4, M6, M6, M8, M8, M8, S2, S2, S3, S7, S8, S9], 1),
            ([M1, M2, M3, M7, M9, P2, P4, S1, S4, S5, S6, S6, S7, S8], 1),
            ([M2, M5, M6, M6, M6, M9, P2, P4, P5, S8, E, GD, GD, GD], 3),
            ([M1, M2, M3, M8, M9, M9, M9, P4, P5, S1, S3, S3, S4, S5], 1),
            ([P2, P3, P6, P7, P8, S1, S2, S3, S5, S5, S6, S7, S8, E], 0),
            ([M3, M4, M5, M9, P4, P5, P5, P7, P9, S2, S6, S7, S8, GD], 2),
            ([M2, M4, M6, P5, P6, P7, P7, P8, P9, S3, S4, S5, S9, S9], 0),
            ([M3, M9, P1, P1, P3, P3, P5, P7, S5, S8, S9, S9, S9, GD], 3),
            ([M2, M3, M6, M7, P1, P2, P5, P6, P7, P9, S3, S4, S7, E], 3),
            ([P2, P3, P5, P5, P5, P8, P8, P8, S1, S2, S4, S6, S8, GD], 2),
            ([M4, M4, M8, P2, P2, P3, P4, P6, P7, P8, S4, S6, S7, S8], 1),
            ([M5, M6, M7, M9, M9, P1, P5, P5, P5, P6, P7, S4, S5, S6], 0),
            ([M4, M4, M4, M6, M6, M6, P2, P4, S2, S4, S5, S7, S8, S9], 1),
            ([M7, M7, P3, P4, P5, P8, P8, S1, S3, S5, S5, S7, S8, S], 2),
            ([M1, M1, M2, M3, M4, M4, M6, M8, P3, P5, P7, S1, S2, S4], 2),
            ([M7, M7, M8, M9, P1, P3, P6, P7, P8, S1, S3, S7, S9, WD], 2),
            ([M1, M1, M2, M4, M5, M6, P1, P1, P8, S4, S6, S7, S8, GD], 2),
            ([M3, M5, P1, P1, P3, P4, P7, P8, S3, S4, S5, S7, S8, N], 2),
            ([M5, M6, M6, P5, P6, P7, P8, P8, S5, S6, S7, S7, S7, S8], 1),
            ([M6, M6, P2, P3, S1, S1, S1, S3, S4, S5, S6, GD, GD, GD], 0),
            ([M1, M3, M5, M6, M8, M8, P1, P2, P3, S4, S5, S7, S8, W], 2),
            ([M2, M4, M5, M9, P5, P6, P6, P7, S4, S9, E, N, N, RD], 4),
            ([M2, M3, M3, M4, M7, M7, P3, P4, P5, S2, S3, S7, S8, S9], 0),
            ([M3, M5, P1, P1, P2, P4, P9, S1, S5, S8, N, GD, GD, RD], 4),
            ([M4, M7, M9, P5, S3, S3, S4, S6, S7, S7, S9, S9, S9, GD], 2),
            ([M1, M1, M2, M3, M8, M9, P2, P6, P6, P9, S1, S2, S2, GD], 3),
            ([M2, M2, M3, M7, M8, M9, M9, P4, P4, S4, S8, S9, E, RD], 3),
            ([M8, P1, P1, P2, P7, P8, S5, S5, S6, S7, S7, S7, N, N], 2),
            ([M8, P1, P4, P4, P5, P6, P8, P9, S2, S3, S5, S7, WD, GD], 3),
            ([M2, M5, M6, P2, P4, P6, P8, P8, P8, S4, S6, S7, S9, GD], 3),
            ([M2, M5, M8, P1, P1, P3, P5, P7, P8, S1, S4, S9, WD, GD], 5),
            ([M4, M8, P1, P3, P4, P7, S4, S5, S5, E, S, N, WD, RD], 5),
            ([M5, P4, P6, P7, S1, S2, S5, S6, S8, E, N, N, WD, GD], 4),
            ([M1, M1, M3, M6, M7, M8, P4, P6, P7, S4, S5, S6, W, GD], 2),
            ([M2, M3, M3, M5, M6, M7, P4, P5, S3, S3, S3, S8, W, W], 1),
            ([M5, M7, M8, M9, P2, P5, P7, P8, S5, S7, S8, S9, W, W], 2),
            ([M2, M3, P2, P4, P4, P5, P6, S3, S4, S6, S6, S6, RD, RD], 1),
            ([M2, M5, M6, P1, P2, P4, P4, S2, S3, S8, S9, S, N, GD], 3),
            ([M7, P3, P3, P4, P4, P5, S5, S5, S5, S6, S7, S7, S8, S9], 0),
            ([M1, M2, M2, M5, M7, M8, M9, M9, M9, P3, P5, S4, S6, E], 2),
            ([M3, M4, M6, M7, M8, M9, P3, P4, P5, P6, P6, P7, S2, S7], 2),
            ([M5, M5, M7, M8, P3, P4, P8, S4, S5, S7, S7, S8, W, W], 3),
            ([M4, M7, M8, M9, M9, P1, P3, P3, P4, P4, P5, P8, E, W], 3),
            ([M3, M4, M5, M5, M6, P2, P4, S3, S4, S6, S7, S, S, N], 2),
            ([M6, M9, M9, P4, P5, P7, P7, S1, S2, S3, S3, S4, W, WD], 2),
            ([M8, M8, P3, P3, P6, P6, P7, P8, P8, S2, S2, S7, S, N], 1),
            ([M2, M3, M4, M5, M5, M6, P1, P1, P3, S4, S5, S6, RD, RD], 1),
            ([M1, M2, M4, M4, M4, M6, P2, P7, P8, P9, S5, S8, S9, GD], 2),
            ([P2, P3, P4, P5, P5, P5, S1, S1, S6, S7, S8, WD, RD, RD], 0),
            ([M2, M3, M3, M5, M5, M6, M7, M8, S3, S3, S4, S4, S5, GD], 1),
            ([M3, M4, M5, M5, P1, P7, P8, S1, S4, S4, S5, S6, S, N], 3),
            ([M2, M3, M4, M5, M8, P2, P3, P4, P8, S2, S2, S4, N, N], 2),
            ([M2, P1, P1, P2, P7, P8, S5, S6, S7, S7, S8, N, N, RD], 2),
            ([M2, M3, M4, M6, M7, M9, P1, S5, S6, S7, S7, S, WD, RD], 3),
            ([M2, M4, M6, P3, P3, P3, P4, P4, P6, P7, P7, P7, P7, P8], 0),
            ([M3, M3, M4, P1, P5, P5, P6, P7, S3, S7, S8, E, N, WD], 4),
            ([M2, M2, M3, M4, M5, M6, P2, P2, P5, P5, S8, S9, WD, RD], 2),
            ([M1, M8, M9, P1, P3, P4, P5, P5, P7, S4, S8, S9, S, GD], 3),
            ([M6, M8, M8, P2, P2, P4, P5, P6, P6, P8, S3, S3, S8, N], 2),
            ([M1, M1, M3, M6, M8, P5, P7, P8, S5, S6, S7, N, N, N], 1),
            ([M2, M3, M5, M7, M7, M8, M8, M9, P5, P6, P7, S9, W, W], 1),
            ([M4, M5, M5, M7, M8, P2, P6, S3, S4, S5, S7, S8, S9, GD], 2),
            ([M2, M6, P4, P5, P8, P9, S4, S4, S5, S6, S6, S7, S9, WD], 3),
            ([M3, M4, M5, P4, P5, P7, P7, S3, S5, S6, S6, E, N, GD], 2),
            ([M5, M5, M7, M7, M8, M9, P3, S1, S3, S4, S5, S6, S7, S8], 1),
            ([M2, M2, M4, M7, M8, M9, P6, S1, S2, S3, S5, E, RD, RD], 2),
            ([M4, M6, M9, P1, P5, P7, P8, P9, S2, S4, S9, E, S, N], 4),
            ([P4, P5, P6, S1, S1, S1, S2, S2, S4, S5, S6, S6, S7, S8], -1),
            ([M2, M2, M3, M4, M9, M9, P4, P6, P8, S1, S2, S3, S3, S5], 1),
            ([M3, M4, M5, M5, M6, M7, M8, P6, P7, P8, S6, S6, S7, S8], 0),
            ([M3, M4, M5, M6, M6, P2, P3, P4, P7, P8, S8, S8, E, WD], 1),
            ([M3, M7, M7, P5, P8, P8, P9, P9, S1, S2, S5, S6, S7, E], 2),
            ([M1, M3, M3, M9, M9, P3, P5, P6, P7, S3, S7, N, N, WD], 3),
            ([M3, M4, M4, M5, M5, M6, P3, P4, P5, S2, S2, S6, S7, S8], -1),
            ([M1, M2, M3, M8, M8, M8, P4, P5, P6, P7, P8, P8, P8, E], 0),
            ([M1, M5, M5, M5, M6, P2, P3, P8, P9, P9, S5, S5, S7, S7], 2),
            ([M1, M3, M3, M6, M7, M8, P1, P2, P3, S4, S4, S5, S6, S7], 0),
            ([M5, M7, M7, P1, P2, S5, S5, S7, S7, S8, S9, S, WD, GD], 3),
            ([M3, P1, P2, P3, P7, P8, P9, S1, S5, S9, WD, WD, GD, RD], 3),
            ([M1, M5, M6, M8, P4, P7, P7, P9, S1, S2, S7, E, W, RD], 5),
            ([M2, M3, M3, M4, M9, M9, P2, P7, P8, S3, S3, S7, S9, S], 2),
            ([M6, M7, M8, P1, P2, P3, P3, P6, P6, P8, S6, S8, GD, GD], 1),
            ([M4, M5, M7, M7, M8, P3, P3, P4, P8, S1, S1, S2, N, N], 2),
            ([M5, M5, M6, M6, M9, P3, P4, P7, S1, S2, S2, S3, E, WD], 3),
            ([M5, M7, M8, M9, P4, P6, P7, S1, S7, S8, S8, S9, S, S], 2),
            ([M2, M2, M4, P3, P4, P5, P7, P7, P9, S1, S2, S7, S8, S9], 1),
            ([M1, M1, M2, M8, M9, P2, P4, S2, S4, S4, S4, S7, S8, S9], 1),
            ([M2, M4, M5, M7, M9, P3, P4, P5, P5, P6, P6, P7, S4, S4], 1),
            ([P2, P5, P6, P6, P7, P7, P8, P9, S1, S1, S2, W, N, GD], 3),
            ([M7, M7, P2, P5, P5, P7, P7, S4, S4, S6, S9, E, E, GD], 1),
            ([M1, M3, M4, M7, M9, P1, P2, P5, P8, P9, S6, S8, E, WD], 4),
            ([M1, M2, M4, M6, M6, P3, P7, S1, S2, S3, S4, S8, S9, W], 3),
            ([M1, M3, M3, M4, M5, M5, M6, P7, P8, S3, S4, S4, S5, S6], 1),
            ([M2, M6, P2, P4, P4, P6, P9, S2, S3, S5, S9, E, N, GD], 5),
            ([M2, M3, M3, M6, M8, P1, P3, P8, S5, S6, S8, S9, S, RD], 3),
            ([M1, M2, M3, M6, P9, P9, S1, S2, S2, S2, S5, S6, RD, RD], 1),
            ([M5, M8, M8, P2, P3, P3, P4, P8, P9, P9, S4, S8, S8, S8], 2),
            ([M2, M4, M4, M8, P1, P2, S2, S3, S7, S7, S8, S9, WD, GD], 3),
            ([M2, M4, M8, P3, P7, P8, P9, S6, S6, S7, S8, S, RD, RD], 2),
            ([M1, M5, M5, M7, M7, P4, P5, P7, P8, S1, S1, S2, S4, S5], 3),
            ([M4, M5, P7, P7, P8, P9, S3, S4, S5, S7, S7, S9, GD, GD], 1),
            ([M2, M2, M3, M5, M6, P1, P2, P3, P4, P7, P7, S3, S5, S7], 2),
            ([M4, M4, M7, P8, P8, S4, S5, S8, S9, S9, S, S, W, GD], 2),
            ([M3, M3, M6, M7, M8, P4, P5, P7, P8, P9, S1, S6, S6, S7], 1),
            ([M2, M3, M3, M5, M6, M6, P5, P6, P7, P7, S1, S2, S2, S4], 2),
            ([M9, P1, P2, P3, P5, P6, P7, P8, P9, S2, S2, E, E, E], 0),
            ([M2, M3, M7, P9, S1, S2, S3, S6, S7, S8, S9, S9, GD, GD], 1),
            ([M5, M5, M5, M7, P1, P1, P3, P5, P5, S3, S6, S7, N, WD], 3),
            ([M9, P2, P5, P5, P6, S1, S3, S5, S8, S9, E, E, WD, RD], 4),
            ([P4, P5, S1, S1, S3, S4, S4, S5, S6, S7, S8, S9, W, W], 1),
            ([M7, M7, M8, M8, P2, P4, P4, P6, P7, P8, S2, S4, S6, W], 2),
            ([M2, M3, M3, M6, M8, P1, P1, P3, P5, P6, P6, S6, WD, WD], 2),
            ([M7, M9, P4, P5, P6, P8, P9, P9, S3, S8, S8, S9, S9, S9], 1),
            ([M1, M5, M8, M8, P4, S1, S3, S3, S3, S4, S, S, N, WD], 3),
            ([M2, M4, M4, M6, P5, P6, P6, P7, S1, S2, S3, S6, W, W], 1),
            ([M6, M9, P2, P3, P4, P4, P6, P6, P7, S4, E, WD, RD, RD], 3),
            ([M2, M3, M4, M5, M6, M7, M7, P2, P3, P4, P7, P8, P9, S4], 0),
            ([M4, M5, M6, M7, M9, P4, P8, P8, P9, S4, S6, S, W, GD], 3),
            ([M1, M3, M4, M7, M8, P1, P2, P2, P5, S4, S6, S, W, RD], 4),
            ([M8, M9, P4, P6, P9, P9, S3, S4, S4, S5, S5, S6, S7, S9], 1),
            ([M6, M6, M8, M8, M8, P5, P7, P9, S2, S4, E, S, GD, RD], 3),
            ([M4, M6, P2, P2, P4, P6, S1, S2, S4, S6, S8, E, N, GD], 3),
            ([M3, M3, M3, M6, M7, M7, M8, P8, P8, S4, S6, S7, S8, N], 1),
            ([M2, M2, M4, M4, M7, M8, M9, P3, P4, P4, P6, P7, S1, S3], 2),
            ([M4, M5, M8, P2, P2, P2, P4, P5, P8, S2, S2, S5, S7, S7], 2),
            ([M1, M1, M2, M2, M3, M3, M5, S4, S4, S5, S6, S6, WD, WD], 0),
            ([M3, M4, M4, M7, M8, P2, P6, P8, S, S, W, N, GD, RD], 4),
            ([M1, M1, M2, M3, M4, M5, M5, M6, S3, S3, S5, S6, S6, S7], 1),
            ([M2, M3, M3, M3, P6, P8, P9, S4, S4, S5, S5, S6, E, S], 2),
            ([M2, M3, M4, P2, P4, P5, P6, P7, S1, S2, S3, S3, S4, S5], 0),
            ([M4, P3, P4, P5, P8, P9, S2, S3, S5, S5, S6, N, N, GD], 2),
            ([M6, M7, M8, P4, P5, P6, P7, P7, P9, P9, S5, S8, S8, S9], 1),
            ([M4, M4, M4, M4, M6, M8, P5, P7, S3, S4, S8, S, W, N], 3),
            ([M2, M2, M4, M7, M8, M9, P6, S1, S2, S3, S5, E, RD, RD], 2),
            ([M3, M4, M5, M6, M7, M7, M8, M8, M8, M9, P7, P9, S2, S2], 0),
            ([M1, M4, P2, P2, P3, P6, P7, P8, P9, S2, S4, E, W, RD], 4),
            ([M2, M3, M5, M7, M8, P6, P7, P7, P8, S3, S5, GD, GD, GD], 2),
            ([M6, M6, M8, P3, P4, P6, P6, P6, S2, S4, S5, S7, W, WD], 2),
            ([M4, M5, M7, M8, M9, P1, P3, P4, P6, P8, P9, P9, S1, S3], 2),
            ([M3, M7, P1, P2, P3, P4, P8, P9, P9, S3, S5, S, W, N], 4),
            ([M3, M5, M6, M7, M7, M8, M8, P2, P2, P5, P5, S2, S5, S6], 2),
            ([M4, M5, M6, M7, M7, M8, S1, S2, S3, S6, S7, S7, N, N], 1),
            ([M1, M4, M8, M8, P2, P5, P7, P8, P8, S2, S3, S6, S6, S8], 3),
            ([M4, M5, M5, M7, M8, P2, P6, S4, S5, S7, S8, S9, WD, GD], 3),
            ([M5, M6, M8, M9, P2, S3, S4, S4, S5, S5, S9, WD, GD, GD], 2),
            ([M1, M5, M7, P1, P1, P3, P7, P7, P9, S4, S5, S6, S9, W], 3),
            ([M6, M7, P4, P5, P7, S1, S3, S4, S4, S5, S5, S6, S8, S8], 1),
            ([M1, M3, M4, M5, M5, M6, M7, P1, P2, P3, S4, S4, S5, S6], 0),
            ([M1, M6, M8, P1, P3, P5, P6, S4, S5, S8, S8, E, N, WD], 3),
            ([M5, M5, M6, M7, M7, M8, P3, P5, P7, S3, S3, S3, S6, S8], 1),
            ([M3, M4, M7, M9, P2, P2, P3, P4, P4, P4, P6, S5, S5, S8], 2),
            ([M3, M4, M5, M7, P2, P4, P5, P7, P9, S1, S2, S6, S7, S], 3),
            ([M3, M7, M9, P2, P4, P7, P8, S2, S3, S3, S4, S5, S8, S9], 3),
            ([M3, M9, M9, M9, P1, P1, P2, S1, S1, S7, S8, S9, N, N], 1),
            ([M5, M5, M7, M8, P2, P2, P4, P7, P8, P9, S4, S8, E, E], 2),
            ([M3, M4, M7, P1, P3, P6, P7, S3, S4, S7, S8, E, E, S], 3),
            ([M5, M6, M9, P3, P4, P5, P8, P8, S2, S4, S4, S5, S7, S9], 2),
            ([M2, M4, M6, P3, P4, P4, P7, P8, P9, S3, S5, S6, N, RD], 3),
            ([M1, M2, M3, M4, M5, P5, P5, P6, P7, P8, S5, S6, S7, S7], 0),
            ([M2, M4, P1, P1, P5, P7, P7, S4, S4, S5, S5, S6, E, S], 2),
            ([M3, P3, P4, P6, P6, P7, S2, S6, S6, S8, N, N, GD, GD], 2),
            ([M3, M6, M7, M8, M9, P1, P3, P4, P5, P6, P7, S2, S8, RD], 3),
            ([M3, M4, M5, M7, M7, M9, M9, P1, P1, P4, P5, S2, S3, S4], 1),
            ([M4, M4, M5, M5, P1, P2, P3, P5, S5, S9, WD, WD, GD, GD], 2),
            ([M2, M4, M4, M4, P2, P3, P4, P4, P5, S3, S6, S7, S8, S9], 1),
            ([M3, M4, M6, M6, P1, P3, P4, P4, S1, S2, S6, S7, S8, GD], 2),
            ([M1, M1, M3, M8, P2, P3, P3, P6, S1, S7, W, W, GD, GD], 2),
            ([P1, P1, P2, P5, P5, P6, P7, P8, P9, S7, S8, S9, WD, WD], 1),
            ([M2, M3, M4, M4, P3, P4, P6, S3, S4, S5, S8, W, W, RD], 2),
            ([M1, M4, M5, M7, M8, M9, P3, P3, P3, P5, P5, P7, S7, S7], 1),
            ([M3, M3, M5, M5, P4, P5, P6, S1, S2, S3, S4, S4, S5, S9], 1),
            ([M2, M3, M6, M6, M8, M8, P2, P3, P4, P6, P6, P8, S, S], 2),
            ([M5, P2, P3, P5, P6, P9, P9, S2, S3, S4, S6, S7, S7, S8], 1),
            ([M3, M4, M5, M6, M7, M7, M8, M8, M9, P7, P9, S1, S2, S2], 0),
            ([M2, M3, M4, M5, M6, M7, M7, P2, P3, P7, P8, P9, S2, S3], 1),
            ([M4, M7, M9, P1, P1, P2, P5, S1, S2, S3, S4, S4, S5, RD], 3),
            ([M9, M9, P1, P1, P2, P4, P9, S1, S1, S1, S8, S9, W, N], 2),
            ([M1, M1, M3, M5, M6, P1, P3, P7, S7, S8, S9, GD, RD, RD], 2),
            ([M8, P2, P2, P5, P5, P6, P8, P8, P8, S4, S5, W, W, GD], 2),
            ([M3, M3, M4, M6, M6, M7, P3, P3, P6, P8, S2, S4, S7, S8], 3),
            ([M1, M2, M3, M4, M4, M7, M8, M8, M8, M9, P5, S5, E, WD], 2),
            ([M1, M1, M4, M5, P3, P3, S1, S2, S3, S3, S4, S6, GD, GD], 2),
            ([M3, M4, M6, M7, M8, P5, P6, P7, P8, P8, S1, S1, S8, E], 1),
            ([M2, M2, M2, M3, M3, M4, M5, M5, M6, M8, P2, P3, S5, GD], 2),
            ([M1, M2, M5, M6, M7, M9, P4, P5, S5, S6, S7, S9, S9, E], 1),
            ([M2, M3, M6, M7, M8, P2, P4, P7, S2, S3, S3, S3, S7, S7], 1),
            ([M1, M1, M1, M4, M5, P5, P7, P9, P9, S2, S6, S8, S9, WD], 2),
            ([M2, P4, P5, P9, S1, S1, S2, S3, S4, S8, S9, WD, WD, GD], 2),
            ([M2, M3, M4, M5, M5, M5, P1, P2, P2, P9, P9, P9, N, N], 0),
            ([M4, M6, M7, P2, P2, P4, P7, P8, S2, S2, S3, S5, S6, W], 3),
            ([M4, M5, M7, M7, P3, P5, P6, P7, P8, S1, S3, S7, S9, RD], 2),
            ([M1, M2, M2, M5, M6, M8, P5, P6, P9, P9, S2, S3, S5, WD], 3),
            ([M2, M5, M6, M7, P3, P4, S6, S7, S, W, W, WD, RD, RD], 2),
            ([M2, M3, M3, M4, M4, M5, M7, M8, S7, S8, S9, N, RD, RD], 0),
            ([M1, M3, P2, P2, P4, P4, P5, P6, P6, P8, P8, S5, S6, S9], 2),
            ([M9, M9, P2, P5, P6, S3, S5, S6, S7, S7, S8, S8, S, S], 2),
            ([M3, M5, M6, M8, P3, P4, P8, P8, S3, S5, S8, S8, S, S], 3),
            ([M2, M2, M4, M5, M5, M7, M8, P3, P3, P5, P6, P7, S1, S2], 2),
            ([M3, M3, M3, M6, P3, P3, P4, P5, P6, P6, S1, S6, S6, S6], 1),
            ([M6, M7, P2, P4, P6, P7, P7, S6, S6, S6, S7, S8, S9, N], 1),
            ([M3, M4, M5, P2, P3, P5, P6, S2, S3, S6, S6, S8, E, E], 2),
            ([M4, M5, M6, M9, P4, P5, P6, S2, S2, S3, S5, S7, S7, N], 1),
            ([M5, M6, P7, P8, P9, S2, S2, S5, E, W, W, W, N, GD], 2),
            ([M3, P3, P5, P5, P6, P6, P9, P9, S1, S1, S4, S6, S8, E], 2),
            ([M4, P4, P5, P6, P6, P7, P8, P8, S1, S1, S1, S5, S6, GD], 1),
            ([M2, M2, M7, M8, M8, P7, P7, P9, S5, S6, S7, S9, S, GD], 3),
            ([M1, M2, M3, M7, M7, P4, P8, S3, S3, S5, S6, S7, E, E], 1),
            ([M3, M8, M9, P2, P3, P4, P5, P6, P8, P8, P9, S4, S5, WD], 2),
            ([M1, M2, M3, M5, M8, P1, P3, P6, P6, P7, S1, S2, WD, GD], 3),
            ([M4, M6, M7, M7, M8, P5, P6, P9, P9, S2, S3, S4, W, WD], 2),
            ([M2, M3, M3, M6, M7, M8, M9, P3, P4, P5, S4, S6, N, N], 1),
            ([M7, M7, P1, P2, P5, P5, P7, P7, S2, S4, S4, E, E, GD], 1),
            ([M2, M2, M6, M7, M7, M9, P2, P5, P6, P6, S1, S4, GD, RD], 3),
            ([M4, M9, P1, P5, P7, P7, S2, S3, S4, S6, S8, S9, S, W], 4),
            ([M2, M3, M4, M5, M5, M5, P2, P2, P9, P9, P9, S, N, N], 0),
            ([M3, M4, M4, M7, M8, M9, M9, P8, S3, S6, S8, S, W, N], 4),
            ([M2, M6, M8, P1, P3, P7, S2, S2, S5, S6, S6, S7, WD, WD], 2),
            ([M2, M4, M5, M5, M6, M9, P3, P6, P6, P9, S5, S6, S7, S8], 3),
            ([M7, P2, P7, S1, S3, S4, S4, S5, S6, S7, S8, S9, S9, S], 3),
            ([M4, P2, P2, P2, P3, P5, P6, P9, S3, S4, S5, S5, S6, S6], 2),
            ([M2, M3, M4, M5, M6, P4, P4, P7, P9, S3, S6, S7, RD, RD], 2),
            ([M3, M4, M5, M6, M7, M8, M9, P2, P3, P8, P8, S5, S7, S7], 1),
            ([M2, M3, M4, M5, M6, M7, P4, P5, P6, P7, P8, P9, S1, S1], -1),
            ([M7, M8, M9, P1, P3, S1, S2, S3, S6, S7, S7, S7, S8, RD], 0),
            ([M2, M4, M4, M7, P5, P6, P6, P9, S6, S8, S8, S9, WD, GD], 3),
            ([M2, M3, M4, M4, M5, M6, P5, P6, S1, S1, S1, S2, S2, E], 0),
            ([M1, M2, M7, P1, P2, P3, P7, P8, P9, S5, S6, S7, E, E], 0),
            ([M1, M2, M2, M3, M3, M3, M4, M5, M9, M9, P9, S4, S5, S6], 0),
            ([M1, M2, M3, M8, P2, P5, P7, P8, P9, S4, S6, S, RD, RD], 2),
            ([M2, M3, M4, M5, M6, P4, P5, P8, S2, S4, E, E, N, WD], 2),
            ([M4, M5, P5, P6, P9, S3, S3, S4, S8, W, W, WD, GD, GD], 3),
            ([M2, M2, M2, M3, M4, M7, M7, M7, M8, P6, P7, S3, S4, S5], 0),
            ([P2, P4, P7, P8, P9, S1, S2, S3, S4, S6, S7, E, E, E], 1),
            ([P5, P6, P7, P9, P9, S2, S3, S4, S6, S6, S7, S7, S8, S9], 0),
            ([M1, M2, M6, P5, P7, S2, S2, S3, S5, S5, S6, S7, S7, RD], 2),
            ([M4, M7, P2, P3, P4, P7, P9, S1, S3, N, WD, WD, GD, GD], 2),
            ([M8, M8, M9, M9, P6, P6, P8, S2, S3, S5, S5, S7, S8, WD], 2),
            ([M5, P1, P2, P4, P6, P8, P9, S2, S4, S6, S6, S7, RD, RD], 3),
            ([M4, M4, M6, M6, M6, M7, P4, P5, P6, P6, S1, S1, S7, S8], 1),
            ([M1, M4, M5, M7, M8, M8, P2, P4, P5, P8, P9, S4, S, N], 4),
            ([M2, M7, M8, M9, M9, P5, S3, S6, S7, S7, S9, WD, GD, GD], 3),
            ([M5, P1, P1, P2, P3, P4, P6, P7, S4, S4, S4, S5, S6, S8], 1),
            ([M5, M5, M7, P5, P6, P6, P7, P7, S1, S1, S4, S4, S4, W], 1),
            ([M3, M6, P5, P8, P8, S2, S2, S6, S7, E, E, S, WD, RD], 3),
            ([M3, M3, M4, M6, M6, M7, P2, P4, P6, P7, P8, S6, S7, S8], 1),
            ([M6, M7, M8, M8, M8, M9, M9, P3, P4, P5, S4, S4, S5, S6], 0),
            ([M4, M6, P3, P3, P3, P4, P4, P6, P7, P7, P7, P7, P8, S6], 0),
            ([M1, M1, M5, M6, M6, M7, P6, P7, P8, P9, P9, S3, S8, WD], 2),
            ([M1, M4, M5, M5, M5, M6, M6, M7, P3, P6, P7, S1, S3, S5], 2),
            ([M4, M5, M7, P4, P4, P6, P7, P8, P9, P9, P9, S5, S6, S7], 0),
            ([M8, P2, P7, S1, S4, S4, S7, S8, S8, S9, S9, S9, GD, RD], 3),
            ([M4, P2, P2, P3, P5, P6, P6, P7, P8, S2, S5, S7, E, WD], 3),
            ([M3, M4, M7, M9, P1, P3, P4, P4, P5, P5, P7, P8, P9, S4], 2),
            ([M1, M2, M7, M7, M8, P2, P5, P5, P6, P9, P9, S4, S6, S], 3),
            ([S1, S1, S1, S1, S3, S4, S5, S7, S7, S8, S9, RD, RD, RD], 0),
            ([M1, M2, M3, M5, P1, P3, P5, P6, P7, P9, P9, S2, S2, S4], 1),
            ([M2, M4, M7, P3, P3, P4, P4, S1, S1, S2, S3, S4, S7, RD], 2),
            ([M3, M4, M4, P1, P4, P6, P8, S1, S4, S7, S7, E, S, W], 4),
            ([M4, M5, M5, M9, P5, P6, P6, P7, P7, S2, N, WD, RD, RD], 2),
            ([M1, M1, M2, M4, M6, M8, M8, M9, P8, P9, S3, S4, W, W], 3),
            ([M4, M4, M5, M5, M7, M9, S2, S3, E, S, N, WD, GD, RD], 4),
            ([M2, M5, M6, P5, P7, P8, P9, S1, S2, S3, S6, S7, E, WD], 2),
            ([M2, M3, M3, M4, M4, M5, P3, P3, S2, S3, S6, S6, S6, S8], 0),
            ([M3, M5, M6, P2, P8, S1, S2, S5, S6, S, W, N, WD, GD], 5),
            ([M6, M7, P8, P9, S1, S1, S2, S4, S4, S6, S6, W, WD, WD], 2),
            ([M2, M3, M4, M6, M6, M7, P2, P6, P7, S3, S6, S6, S6, W], 2),
            ([M4, M5, M5, M6, P5, P6, P7, P7, S1, S5, S6, S6, S7, W], 2),
            ([P3, P3, P3, P3, P5, P6, S4, S5, S6, S6, S8, S8, N, N], 1),
            ([M2, M3, M4, M4, M5, P3, P6, P7, S1, S6, S6, S7, S8, S9], 1),
            ([M1, M2, M3, M3, M6, M9, P6, S2, S3, S8, N, N, WD, RD], 4),
            ([M1, M2, M3, M4, M7, M8, M9, S4, S4, S5, S5, S6, S6, S8], 0),
            ([M1, M1, M3, M4, M5, M8, M9, P2, P4, P5, P5, P6, P6, P7], 0),
            ([M2, M4, M6, P5, P6, P6, P8, S1, S2, S3, S6, E, W, W], 2),
            ([M2, M6, M6, P1, P3, P7, P7, P8, P9, P9, S4, S5, S5, S6], 1),
            ([S1, S2, S3, S4, S5, S6, S8, S8, E, E, E, W, W, WD], 0),
            ([M3, M4, M4, M8, P1, P4, P9, S2, S3, S6, S7, S9, W, N], 5),
            ([M2, M3, M3, M4, M4, M6, M8, P3, P3, P4, P7, S2, S4, S4], 2),
            ([M1, P3, P4, P5, P6, P7, P7, P9, P9, S3, S4, S4, S6, S7], 2),
            ([M1, M4, P5, P6, P6, P8, S4, S5, S8, S9, S9, E, W, W], 3),
            ([M5, M5, M5, M8, P6, P6, P6, S6, S6, S7, S8, S9, WD, WD], 0),
            ([M3, M3, M4, M7, M8, M9, P3, P4, P4, S2, S2, S4, S5, S6], 1),
            ([M6, P6, P7, S1, S1, S3, S4, S5, S6, S7, S8, S, W, W], 1),
            ([M1, M3, M5, P2, P3, P5, P5, P5, P7, S2, S5, S8, S8, S8], 2),
            ([M5, M7, P1, P1, P3, P3, P4, S1, S5, S7, S7, GD, RD, RD], 2),
            ([M3, M6, P3, P4, P4, P5, P5, P6, P7, P8, S3, S4, S5, S5], 1),
            ([P4, P6, P7, P7, S1, S2, S4, S4, S5, S5, S7, S8, S9, GD], 2),
            ([M2, M3, M3, M3, P4, P7, P7, P9, P9, S6, S8, S9, S9, WD], 2),
            ([M3, M4, M5, M6, M6, M6, P8, P8, S4, S5, S6, S7, S8, S8], 0),
            ([M3, M4, M4, M5, M7, P1, P1, P1, P2, S1, S5, S5, S6, S8], 2),
            ([M2, M2, M2, M7, M8, M9, P5, P5, P7, P8, P9, S7, S8, S9], -1),
            ([M3, M4, M4, M5, M7, P3, P4, P5, P5, P6, P7, S5, S6, S7], 0),
            ([M1, M3, P1, P2, P4, P4, P5, P5, S3, S3, S5, W, N, N], 2),
            ([M5, M7, M7, M7, P5, P8, P8, S3, S6, S7, S7, S9, E, S], 3),
            ([M2, M2, M3, M4, M7, M7, P2, P4, P5, P6, S2, S6, W, W], 2),
            ([M1, M4, M5, M8, P3, P3, P6, P6, P8, P8, S7, S8, S8, S9], 2),
            ([M4, M5, M5, M5, M6, M9, P3, S2, S8, S, N, WD, GD, RD], 5),
            ([M7, M8, M8, P1, P1, P8, P8, P9, S3, S3, S5, S6, S6, S7], 1),
            ([M3, M4, M4, P4, P5, P7, P7, P9, S4, S4, S4, S6, S6, S7], 2),
            ([M1, M2, M3, M4, M5, P5, P5, P6, P7, P8, S5, S6, S7, E], 0),
            ([M1, M2, M5, M9, P4, P6, P9, S2, S2, S5, S7, E, WD, RD], 4),
            ([M3, M6, P2, P4, P5, P6, P7, P7, S7, S8, N, WD, RD, RD], 3),
            ([M2, M4, M5, M6, P3, P3, P4, P5, S4, S5, S5, S6, S8, S8], 1),
            ([M3, M3, M4, M7, M7, P8, P8, S3, S3, S3, S5, S6, S7, RD], 1),
            ([M1, M2, M3, M6, P1, P2, P3, S1, S2, S6, S7, S9, S9, S9], 1),
            ([M3, M3, M4, M7, P3, P4, P5, P6, S1, S6, S7, S, GD, RD], 4),
            ([M3, M5, P4, P4, P5, P7, P9, S1, S6, S6, S8, GD, GD, RD], 3),
            ([M3, M4, M6, P1, P2, P8, S4, S4, S4, S5, S6, S8, S9, S9], 2),
            ([M2, M4, P2, P3, P4, P6, P7, P8, S2, S4, S9, S9, S9, N], 1),
            ([M4, P1, P4, P6, P7, P9, S1, S5, S7, S, WD, GD, RD, RD], 4),
            ([M5, P1, P8, S1, S1, S2, S2, S3, S3, S4, S6, E, S, GD], 3),
            ([M5, P2, P3, P3, P4, S2, S4, S5, S5, S8, S9, E, N, GD], 3),
            ([M5, M6, P7, P8, P9, S2, S2, S3, S5, S6, W, W, W, N], 1),
            ([M2, M2, M3, M4, M4, M6, M8, P4, P4, P5, S4, S4, S8, S9], 2),
            ([M1, M2, M2, M2, M6, M7, M9, P2, P5, P5, S4, S5, S7, S7], 2),
            ([M2, M4, P5, P6, P7, P7, P7, P9, S2, S3, S5, E, WD, RD], 3),
            ([M2, M5, M6, M9, P4, P5, S2, S3, S3, S4, S4, N, WD, RD], 3),
            ([M2, M2, M2, M9, P8, P8, P9, P9, S1, S7, S7, N, WD, WD], 1),
            ([M2, M3, M3, M4, P7, S1, S1, S4, S4, E, S, N, GD, RD], 3),
            ([M2, M7, M9, P1, P4, P6, P6, P7, P8, S2, S3, S6, N, GD], 3),
            ([M3, M4, M5, M6, M7, M7, P3, P3, P7, P8, P8, P9, P9, S8], 1),
            ([M2, M8, M9, P1, P2, P3, P3, P4, P7, P9, S5, S7, S9, GD], 3),
            ([M1, M3, P1, P2, P2, P4, S1, S4, S5, S5, S7, S8, GD, RD], 3),
            ([M4, M5, M6, M6, P3, P3, P4, P7, S4, S4, S5, S8, N, GD], 3),
            ([M4, M5, M5, M7, M7, P5, P8, P9, S2, S3, S3, S5, S5, WD], 2),
            ([M1, M3, M5, M6, M6, M9, M9, P2, P8, S4, S5, E, S, W], 4),
            ([M5, M6, M6, P3, P3, P3, S4, S6, S6, S6, S8, S8, RD, RD], 1),
            ([M3, M3, M3, M5, M6, P2, P3, P7, P8, P9, S4, S5, S6, WD], 1),
            ([M3, M6, P4, S1, S2, S4, S5, S5, S9, N, N, WD, WD, WD], 3),
            ([M1, M3, M3, M7, P3, P6, P8, P9, S1, S1, S1, S5, S7, S8], 3),
            ([M2, M3, M4, M7, M8, P2, P2, P5, S4, S4, S6, S6, S6, S7], 1),
            ([M1, M2, M3, P6, P7, P7, S1, S2, S2, S3, S4, S7, S8, N], 1),
            ([M6, M6, P2, P4, P8, P8, S2, S2, S4, S4, S5, S5, S6, S6], 0),
            ([M5, M5, M6, M7, M7, M8, M8, M9, S3, S4, S5, S6, S7, S8], -1),
            ([M1, M2, M6, P2, P3, P3, P5, P6, P7, S1, S4, S5, GD, GD], 2),
            ([M3, M4, M5, M6, P5, P6, P7, P7, P8, S5, S, S, S, W], 1),
            ([M1, M3, M3, M4, P7, S5, S6, S8, S, W, W, N, GD, RD], 4),
            ([M2, M3, P2, P4, P4, P5, P6, S3, S4, S6, S6, S7, RD, RD], 2),
            ([M1, M2, M3, P1, P2, P3, S1, S2, S4, S6, S7, S9, S9, S9], 1),
            ([M4, M5, M6, M7, M8, M9, P2, P3, P4, P5, S2, S2, S7, S8], 0),
            ([M7, P1, P2, P3, P4, P5, P8, S4, S6, S6, W, W, GD, RD], 3),
            ([M3, M4, M5, P4, P4, P6, P7, P7, P8, S1, S3, S4, S6, RD], 1),
            ([M4, M5, M5, M7, M8, P4, P5, P5, P6, P6, P9, S4, S5, S6], 1),
            ([M1, M2, M7, M8, M8, M9, P5, P7, S1, S2, S4, S5, S6, RD], 2),
            ([M4, M6, M7, P1, P2, P4, S3, S5, S7, N, N, WD, GD, RD], 4),
            ([M4, M4, M5, M6, M7, M8, M9, P3, P4, P6, P6, P8, S4, S5], 1),
            ([P3, S1, S1, S3, S4, S4, S4, S5, S6, S7, S8, S9, W, W], 1),
            ([M3, M4, M7, M7, M9, P2, P7, P7, S2, S3, S4, S7, S7, S7], 1),
            ([M3, M3, M6, P2, P2, P8, S2, S3, S3, S4, S4, S4, S8, S8], 1),
            ([M3, M4, M6, M9, M9, P3, S2, S2, S4, S5, S6, S6, N, GD], 3),
            ([M2, M2, M5, M6, M9, P5, P7, P8, P9, S3, S3, S5, S5, S6], 2),
            ([M2, M3, M4, M8, P3, P4, P5, P7, S4, S7, S7, S9, S9, E], 2),
            ([M1, M2, M3, M5, M5, M6, P4, P7, S2, S3, S4, S5, S9, E], 3),
            ([M2, M4, M5, M5, M7, P6, S3, S4, S6, S6, S7, WD, WD, GD], 3),
            ([M3, M4, M5, M6, M7, M9, M9, P5, P5, P6, S2, S2, S2, N], 1),
            ([M1, M2, M3, P1, P2, P3, P5, P6, P6, P8, P9, S1, S7, S8], 1),
            ([M1, M2, M3, M3, M5, M8, M8, M8, P3, P3, P4, S4, S4, GD], 1),
            ([M1, M1, M3, M4, M6, M7, M7, M8, P2, P3, S, S, S, GD], 1),
            ([M2, M5, M6, M7, M8, M9, P1, P2, P5, P6, S4, S7, S9, S9], 2),
            ([M6, M6, M7, M8, M9, P1, P2, P5, P6, P9, S2, S3, S5, S], 2),
            ([M4, M4, M5, M6, M6, M8, P6, S1, S1, S2, S9, E, WD, GD], 3),
            ([M2, M4, M7, M8, M9, P1, P4, P6, S3, S7, S7, WD, GD, RD], 3),
            ([M2, M7, M7, M8, M8, M9, M9, P1, P2, P3, S6, S6, S7, S9], 0),
            ([M4, M5, M6, M8, P2, P3, P6, S1, S3, S4, S5, S7, S8, S9], 1),
            ([M2, M3, M4, M4, M5, M7, M7, M8, M8, P2, P6, P6, S6, S8], 2),
            ([M2, M3, P1, P1, P2, S1, S2, S3, S5, S6, E, S, S, GD], 2),
            ([M3, M5, M7, P3, P3, P7, P8, S1, S2, S3, S4, S4, S5, S6], 1),
            ([M6, M8, P1, P1, P5, P5, P7, P9, S4, S5, S8, W, WD, GD], 3),
            ([M3, M3, M4, M7, M8, P2, P5, P8, P8, S3, S6, S6, S8, WD], 3),
            ([M1, M2, M6, M7, M8, P3, P4, P4, P5, P5, P6, P7, P8, P9], 0),
            ([M1, M2, M3, M5, M6, P4, S1, S1, S5, S6, S6, E, S, GD], 3),
            ([M2, M6, M6, M9, M9, P3, P9, P9, S2, S2, S5, S7, S7, W], 1),
            ([M3, M4, M6, M6, P2, P4, P5, P6, P7, P8, S5, S6, S7, RD], 1),
            ([M3, M4, M4, M5, M7, P3, P6, S1, S6, S8, S9, W, W, RD], 4),
            ([M1, M3, M5, M6, M8, M8, P2, S2, S4, S4, S7, S8, N, RD], 3),
            ([P2, P3, P4, P6, P7, P7, P9, S1, S2, S5, S6, S7, S8, S9], 1),
            ([M1, M3, M4, M6, M8, P3, P7, P8, P9, S2, S8, E, E, S], 3),
            ([M3, M7, P2, P3, P5, P7, P8, S1, S2, S3, S3, S, WD, WD], 3),
            ([M6, M6, P2, P3, P4, P7, P8, S1, S2, S3, S7, S7, S7, RD], 0),
            ([M2, M5, M8, P2, P3, P8, P9, S3, S3, S3, S5, S5, S8, WD], 3),
            ([M3, M4, M4, M5, M7, M8, M9, P3, P5, P7, S7, S7, E, W], 2),
            ([M3, M4, M6, M9, M9, P2, P2, P4, P4, P5, P6, P8, S9, RD], 3),
            ([M3, M3, M5, M6, M6, M6, M7, M8, M9, M9, P6, S5, S6, S7], 1),
            ([M3, M4, M5, M7, M8, P2, P3, P6, P7, S1, E, E, N, RD], 2),
            ([M8, M8, M8, M9, P4, P4, P5, P5, P6, P7, P8, S5, S6, S7], 0),
            ([M3, M4, M7, M8, P2, P6, P8, S2, S9, S, S, W, N, RD], 4),
            ([M5, M5, P2, P4, P7, S1, S2, S8, S8, WD, WD, GD, GD, RD], 2),
            ([M2, M3, P1, P2, P4, P5, P8, P9, S6, S7, S7, S9, S, W], 3),
            ([M2, M4, M6, P1, P1, P2, P2, P5, P6, P7, P7, S6, WD, GD], 3),
            ([M6, M6, P2, P3, P4, P5, P7, S2, S2, S3, S7, S7, S8, S], 2),
            ([M1, M3, M3, M7, M8, P5, P6, S1, S3, S3, S5, S5, S6, S], 3),
            ([M9, M9, P3, P3, P7, P8, P9, S4, S, W, WD, GD, GD, RD], 3),
            ([M1, M6, M7, P5, P6, P7, S1, S2, S3, S5, S7, S7, W, W], 1),
            ([M5, M6, M6, P1, P2, P2, P4, P6, S1, S2, S2, S2, S6, RD], 3),
            ([M8, M8, P1, P3, P4, P8, P8, S3, S3, S6, S7, GD, RD, RD], 2),
            ([M2, M2, M5, M5, M6, P3, P3, P3, P6, S2, S4, S6, S8, GD], 2),
            ([M2, M3, M5, M5, M6, P2, P4, P6, P7, S1, S4, S6, E, S], 3),
            ([M2, M3, M4, M7, P2, P9, S1, S5, S7, S8, S9, W, N, RD], 4),
            ([M1, S3, S4, S4, S4, S6, S6, S7, E, W, N, WD, GD, RD], 4),
            ([M2, M6, M8, M9, P3, S3, S4, S4, S5, S6, S7, S8, S9, S], 2),
            ([M2, M3, M3, M7, P8, S5, S6, S7, S8, E, N, WD, GD, RD], 5),
            ([M1, M7, M7, M8, P3, P4, P6, S3, S5, S5, S6, S7, W, W], 2),
            ([M8, M9, P5, P5, P7, P7, P7, S4, S5, S5, S6, S6, S7, RD], 0),
            ([M6, M7, M7, M9, P2, P4, S1, S1, S2, S5, S7, S9, N, N], 3),
            ([M3, M4, P6, P6, P7, P8, P8, P8, S6, S7, S7, E, E, GD], 2),
            ([M2, M4, M5, M6, M8, P1, P2, P4, P6, P7, S5, S5, S8, GD], 3),
            ([M2, M4, M5, M7, P4, P6, P7, P8, P9, P9, P9, S5, S6, E], 2),
            ([M2, M3, M4, M5, M6, M8, P6, S2, S2, S4, S6, W, N, GD], 3),
            ([M1, M8, P3, P8, P9, S1, S2, S3, S7, S9, S, WD, WD, GD], 3),
            ([M2, M3, M3, M3, M7, M8, P3, P4, P7, P9, S4, S5, S7, S9], 3),
            ([M5, M7, M8, M9, P6, P8, P8, S4, S5, S6, E, S, WD, GD], 3),
            ([M4, M6, P2, P3, P4, P5, P6, P6, P7, P8, S4, S6, N, N], 1),
            ([M5, M7, M8, M9, P8, P8, S4, S5, S6, S9, E, S, WD, GD], 3),
            ([M1, M1, M4, M9, P1, P3, P5, P6, S4, S5, S7, E, N, GD], 4),
            ([M2, M3, M8, M8, M9, P1, P1, P3, P3, P4, P7, S7, GD, GD], 2),
            ([M4, M5, M5, M6, M6, M8, P3, P8, S1, S5, S7, S8, E, S], 4),
            ([M4, M4, M8, P6, P8, P8, P9, P9, S1, S1, S2, S3, S4, E], 2),
            ([M7, M8, P3, P4, P5, P5, P6, P7, S3, S6, S8, S8, S8, RD], 1),
            ([M4, M7, P2, P4, P5, P6, P7, P9, P9, S3, S4, S6, S6, RD], 2),
            ([M1, M2, M2, M5, M9, P7, P7, P8, P9, S2, S3, S6, E, WD], 4),
            ([M4, M4, M6, M6, M6, P4, P5, P6, P9, S1, S1, S7, S8, E], 1),
            ([M3, M6, P4, P7, P7, P8, P9, S3, S6, S8, S8, S9, S9, S9], 3),
            ([M1, M1, M2, M3, M4, M5, M8, M9, P4, P5, P5, P6, P6, P7], 0),
            ([M3, M8, P1, P2, P5, P5, S4, S4, S5, S6, S7, S, W, RD], 3),
            ([M1, M1, M2, M3, M6, P6, P9, S1, S3, S3, S6, S6, S9, S9], 2),
            ([M3, M8, P3, P5, P6, P8, P9, S1, S1, S1, S3, S6, N, N], 3),
            ([M5, M6, P3, P5, P7, P7, P8, S2, S3, S4, S5, S6, WD, WD], 2),
            ([M2, M3, M6, P5, P6, P9, S2, E, E, S, W, N, GD, GD], 4),
            ([P5, P6, P7, P8, P9, S2, S3, S3, S4, S5, S6, S7, S8, S9], 0),
            ([M2, M3, M3, P2, P4, P5, P5, P7, P7, S1, S5, S7, S9, S9], 2),
            ([M4, M4, M4, M6, M6, M6, P2, P4, S4, S5, S7, S8, S8, S9], 1),
            ([M2, M3, M5, M6, M6, M8, M8, P4, P6, P7, P7, P8, S5, S5], 2),
            ([M3, M4, M5, M6, M6, M7, P2, P3, S1, S2, S3, S9, S9, E], 1),
            ([M1, M2, M5, M5, M6, P2, P3, P3, P4, S3, S4, S4, S6, S8], 2),
            ([M5, M6, M9, P1, P2, P4, P5, P6, S3, S4, S4, S, S, S], 1),
            ([M2, M3, M6, P1, P2, P2, P5, P5, P9, S6, S8, S9, W, WD], 4),
            ([M1, M2, M2, M3, M3, M4, M5, M6, E, WD, WD, GD, RD, RD], 1),
            ([M3, M4, M7, P1, P3, P3, P6, P7, P8, P8, S5, S6, GD, GD], 2),
            ([M1, M5, M6, M6, M8, M8, P3, P9, S3, S3, S7, N, RD, RD], 2),
            ([M1, M2, M3, M5, P1, P2, P6, P8, S3, S3, S6, S7, E, WD], 2),
            ([M2, M4, M4, M4, M9, M9, P2, P3, P9, S1, S2, S3, N, GD], 2),
            ([M2, M5, M7, P2, P2, P3, S7, S9, E, E, WD, GD, RD, RD], 3),
            ([M6, M7, M7, M7, M8, M8, M8, P2, P3, P6, P7, E, E, S], 1),
        ];

        let input = hands.map(|hand| {
            let mut ret = (
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0,
                ],
                hand.1,
            );
            for v in hand.0 {
                ret.0[(v as i8 - 1) as usize] += 1;
            }
            ret
        });

        for v in input {
            assert_eq!(calc_shanten(&Vec::from(v.0)), v.1);
        }
    }

    #[test]
    pub fn shanten_works_debug() {
        // Interesting cases from https://github.com/MahjongRepository/mahjong/commit/e8ba9fe306148d47ae9fb5e94d1e4dd4c08d7c1e

        let hands = [
            ([M1, M1, S, S, S, S, W, W, W, W, N, N, N, N], 2),
            ([M2, M3, S, S, S, S, W, W, W, W, N, N, N, N], 2),
            ([E, E, E, E, S, S, S, S, W, W, W, N, N, N], 1),
        ];

        // 13-tiles cases
        let hands_13_tiles = [
            ([M1, M2, M3, M4, M5, M6, M7, M8, M9, E, E, E, E], 1),
            ([M1, M2, M3, M4, M5, M6, M7, M8, M9, P1, P1, P1, P1], 1),
            ([P1, P2, P3, S1, S1, S2, S2, S3, S3, M1, M1, M1, M1], 1),
            ([E, E, E, E, S, S, S, W, W, W, N, N, N], 1),
            ([M1, M1, E, E, E, E, S, S, S, S, W, W, W], 2),
            ([M2, M3, E, E, E, E, S, S, S, S, W, W, W], 2),
            ([E, E, E, E, S, S, S, S, W, W, W, W, N], 3),
        ];

        let input = hands.map(|hand| {
            let mut ret = (
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0,
                ],
                hand.1,
            );
            for v in hand.0 {
                ret.0[(v as i8 - 1) as usize] += 1;
            }
            ret
        });

        let input_13_tiles = hands_13_tiles.map(|hand| {
            let mut ret = (
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0, 0, 0, //
                    0, 0, 0, 0, 0, 0, 0,
                ],
                hand.1,
            );
            for v in hand.0 {
                ret.0[(v as i8 - 1) as usize] += 1;
            }
            ret
        });

        for v in input {
            assert_eq!(calc_shanten(&Vec::from(v.0)), v.1);
        }

        for v in input_13_tiles {
            assert_eq!(calc_shanten(&Vec::from(v.0)), v.1);
        }
    }

    #[test]
    pub fn hairi_tempai_hand() {
        let hand = [
            2, 2, 2, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 1, 0, 0, //
            0, 1, 1, 0, 0, 2, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0,
        ];
        let result = hairi(Vec::from(hand).as_mut());
        assert!(!result.is_none());
        let res = result.unwrap();
        assert_eq!(res.now, 0);
        assert_eq!(res.wait, [18, 21]);
        assert_eq!(res.waits_after_discard, []);
    }

    #[test]
    pub fn hairi_non_tempai_hand() {
        let hand = [
            2, 2, 2, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 1, 0, 0, //
            0, 1, 0, 0, 1, 2, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0,
        ];
        let result = hairi(Vec::from(hand).as_mut());
        assert!(!result.is_none());
        let res = result.unwrap();
        assert_eq!(res.now, 1);
        assert_eq!(res.wait, [18, 19, 20, 21, 22, 23, 24]);
    }

    #[test]
    pub fn hairi_riichi_hand() {
        let hand = [
            2, 2, 2, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 1, 1, 1, 0, 0, //
            0, 0, 1, 1, 1, 1, 1, 0, 0, //
            0, 0, 0, 0, 0, 0, 0,
        ];
        let result = hairi(Vec::from(hand).as_mut());
        assert!(!result.is_none());
        let res = result.unwrap();
        assert_eq!(res.now, 0);
        assert_eq!(
            res.waits_after_discard,
            [
                (20, vec![21, 24]),
                (21, vec![20]),
                (23, vec![24]),
                (24, vec![20, 23])
            ]
        );
    }

    #[test]
    pub fn hairi_partial_hand() {
        let hand = [
            2, 2, 3, 2, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            1, 1, 1, 0, 2, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0,
        ];
        let result = hairi(Vec::from(hand).as_mut());
        assert!(!result.is_none());
        let res = result.unwrap();
        assert_eq!(res.now, 0);
        assert_eq!(
            res.waits_after_discard,
            [(0, vec![1, 4]), (2, vec![0, 3, 22]), (3, vec![1, 4])]
        );
    }
}
