use crate::constants::{Suit, Val, digest, kokushi_idx, slice_by_suit, sum};

pub fn check7(haipai: &Vec<i32>) -> bool {
    let mut s = 0;
    for i in 0..haipai.len() {
        if haipai[i] > 0 && haipai[i] != 2 {
            return false;
        }
        s += haipai[i];
    }
    s == 14
}

pub fn check13(haipai: &Vec<i32>) -> bool {
    let arr = kokushi_idx().map(|i| haipai[i as usize - 1]).to_vec();
    !arr.contains(&0) && sum(&arr) == 14
}

fn check_internal(haipai: &Vec<i32>, is_jihai: bool) -> bool {
    let mut tmp = haipai.clone();
    let haipai_c: &mut Vec<i32> = tmp.as_mut();
    let s = sum(haipai_c);
    if s == 0 {
        return true;
    }

    if s % 3 == 2 {
        for i in 0..haipai_c.len() {
            if haipai_c[i] >= 2 {
                haipai_c[i] -= 2;
            } else {
                continue;
            }

            if !check_internal(haipai_c, is_jihai) {
                haipai_c[i] += 2;
            } else {
                return true;
            }
        }
        return false;
    }

    for i in 0..haipai_c.len() {
        if haipai_c[i] == 0 {
            continue;
        }

        if haipai_c[i] == 3 {
            haipai_c[i] = 0;
        } else {
            if is_jihai || i >= 7 {
                return false;
            }

            if haipai_c[i] == 4 {
                haipai_c[i] -= 3;
            }

            haipai_c[i + 1] -= haipai_c[i];
            haipai_c[i + 2] -= haipai_c[i];

            if haipai_c[i + 1] < 0 || haipai_c[i + 2] < 0 {
                return false;
            }

            haipai_c[i] = 0;
        }
    }

    true
}

pub fn check(haipai: &Vec<i32>) -> bool {
    let mut j = 0;

    for i in 0..3 {
        // Summing by suit except honors
        let slice_sum = sum(&Vec::from(&haipai[i * 9..(i + 1) * 9]));

        if slice_sum % 3 == 1 {
            return false;
        }

        j += if slice_sum % 3 == 2 { 1 } else { 0 };
    }

    // Adding honors as well
    let slice_sum = sum(&Vec::from(&haipai[3 * 9..3 * 9 + 7]));

    if slice_sum % 3 == 1 {
        return false;
    }

    j += if slice_sum % 3 == 2 { 1 } else { 0 };

    let slices = slice_by_suit(haipai);
    j == 1
        && check_internal(&slices[0], false)
        && check_internal(&slices[1], false)
        && check_internal(&slices[2], false)
        && check_internal(&slices[3], true)
}

pub fn check_all(haipai: &Vec<i32>) -> bool {
    check7(haipai) || check13(haipai) || check(haipai)
}

// Finds indices in hand where kotsu is detected
// Doesn't find kantsu.
// Mutates original array!
pub fn find_kotsu(haipai: &mut Vec<i32>) -> Vec<Vec<i32>> {
    let mut res: Vec<Vec<i32>> = Vec::new();
    for i in 0..haipai.len() {
        if haipai[i] >= 3 {
            haipai[i] -= 3;
            if check(haipai) {
                res.push(Vec::from([i as i32 + 1, i as i32 + 1, i as i32 + 1]));
            } else {
                haipai[i] += 3;
            }
        }
    }

    res
}

// Finds arrays of indices in hand where shuntsu is detected
// Mutates original array!
pub fn find_shuntsu(haipai: &mut Vec<i32>) -> Vec<Vec<i32>> {
    let mut res: Vec<Vec<i32>> = Vec::new();

    // Don't consider honors (last 7).
    for i in 0..haipai.len() - 7 {
        // Also skip last two suit tiles, because there can't be any shuntsu starting from 8 or 9.
        if i + 1 == Suit::Man as usize * 9 + Val::N8 as usize
            || i + 1 == Suit::Man as usize * 9 + Val::N9 as usize
            || i + 1 == Suit::Pin as usize * 9 + Val::N8 as usize
            || i + 1 == Suit::Pin as usize * 9 + Val::N9 as usize
            || i + 1 == Suit::Sou as usize * 9 + Val::N8 as usize
            || i + 1 == Suit::Sou as usize * 9 + Val::N9 as usize
        {
            continue;
        }

        while haipai[i] >= 1 && haipai[i + 1] >= 1 && haipai[i + 2] >= 1 {
            haipai[i] -= 1;
            haipai[i + 1] -= 1;
            haipai[i + 2] -= 1;

            if check(haipai) {
                res.push(Vec::from([
                    i as i32 + 1,
                    (i + 1) as i32 + 1,
                    (i + 2) as i32 + 1,
                ]));
            } else {
                haipai[i] += 1;
                haipai[i + 1] += 1;
                haipai[i + 2] += 1;
                break;
            }
        }
    }

    res
}

// Finds index of first set of repeated tiles or -1 otherwise
// Skip excluded index - it's used below as fake pair
pub fn find_janto(haipai: &Vec<i32>, exclude: i32) -> i32 {
    for i in 0..haipai.len() {
        if haipai[i] >= 2 && i as i32 != exclude {
            return i as i32;
        }
    }
    -1
}

// Find hand split variant
// Skip excluded index - it's used below as fake pair
pub fn calc(haipai: &Vec<i32>, exclude: i32, real_pair: i32) -> Vec<Vec<Vec<i32>>> {
    let mut res: Vec<Vec<Vec<i32>>> = Vec::new();

    // First pass: find kotsu, then shuntsu
    let mut clone = haipai.clone();
    let mut kotsu = find_kotsu(clone.as_mut());
    if sum(&clone) == 2 {
        // toitoi-like
        let janto = if real_pair != -1 {
            real_pair
        } else {
            find_janto(&clone, exclude)
        };
        kotsu.extend_from_slice(&[Vec::from([janto + 1, janto + 1])]);
        res.push(kotsu);
    } else if kotsu.len() > 0 {
        let shuntsu = find_shuntsu(clone.as_mut());
        let janto = if real_pair != -1 {
            real_pair
        } else {
            find_janto(&clone, exclude)
        };
        kotsu.extend_from_slice(&shuntsu);
        kotsu.extend_from_slice(&[Vec::from([janto + 1, janto + 1])]);
        res.push(kotsu);
    }

    // Second pass: find shuntsu, then kotsu
    clone = haipai.clone();
    let mut shuntsu = find_shuntsu(clone.as_mut());
    if sum(&clone) == 2 {
        let janto = if real_pair != -1 {
            real_pair
        } else {
            find_janto(&clone, exclude)
        };
        // pinfu-like
        shuntsu.extend_from_slice(&[Vec::from([janto + 1, janto + 1])]);
        res.push(shuntsu);
    } else {
        let kotsu = find_kotsu(clone.as_mut());
        let janto = if real_pair != -1 {
            real_pair
        } else {
            find_janto(&clone, exclude)
        };
        shuntsu.extend_from_slice(&kotsu);
        shuntsu.extend_from_slice(&[Vec::from([janto + 1, janto + 1])]);
        res.push(shuntsu);
    }

    res
}

pub fn find_all_agari_patterns(haipai: &Vec<i32>) -> Vec<Vec<Vec<i32>>> {
    let mut res: Vec<Vec<Vec<i32>>> = Vec::new();

    let mut clone = haipai.clone();

    let can_be_kokushi = check13(&clone);
    let can_be_chiitoitsu = check7(&clone);
    let can_be_basic_form = check(&clone);

    if !can_be_kokushi && !can_be_chiitoitsu && !can_be_basic_form {
        return res;
    }

    // only a pair left in closed part -> try to detect and return it
    if sum(&clone) == 2 {
        let found = find_janto(&clone, -1);
        if found != -1 {
            res.push(Vec::from([Vec::from([found + 1, found + 1])]));
        }
        return res;
    }

    // Check kokushi separately
    if can_be_kokushi {
        let mut vals: Vec<i32> = Vec::new();
        for i in 0..clone.len() {
            if clone[i] > 0 {
                vals.push(i as i32 + 1);
                if clone[i] > 1 {
                    vals.push(i as i32 + 1);
                }
            }
        }
        res.push(Vec::from([vals]));
    }

    // Some questionable code below :)

    let mut fake_pair_index = -1;
    for i in Suit::Honor as i32 * 9..34 {
        if clone[i as usize] == 0 {
            // found first honor tile that is absent in hand
            clone[i as usize] += 2; // add two fake tiles there so calc() would think the hand is valid when another pair is excluded.
            fake_pair_index = i; // save fake pair index to avoid processing it below
            break;
        }
    }

    // Here we try to iterate over hand and try to exclude any found pair from there.
    // If hand is still valid, this means we found another proper valid hand decomposition.
    // Fake pair added above is required to keep proper tiles count in hand.

    for i in 0..clone.len() {
        if i as i32 == fake_pair_index {
            // Don't process fake pair
            continue;
        }

        if clone[i] >= 2 {
            clone[i] -= 2;
            if check(&clone) {
                let calc_res = calc(&clone, fake_pair_index, i as i32);
                res.extend_from_slice(&calc_res);
            }
            clone[i] += 2;
        }
    }

    if fake_pair_index != -1 {
        clone[fake_pair_index as usize] -= 2;
    }

    if can_be_chiitoitsu {
        let mut vals: Vec<Vec<i32>> = Vec::new();
        for i in 0..clone.len() {
            if clone[i] == 2 {
                vals.push(Vec::from([i as i32 + 1, i as i32 + 1]));
            }
        }
        res.push(vals);
    }

    // Finally we try to find and eliminate duplicate decompositions.
    let mut final_res: Vec<Vec<Vec<i32>>> = Vec::new();

    for i in 0..res.len() {
        let mut is_duplicate = false;
        let v_digest = digest(&res[i]);
        for ii in 0..final_res.len() {
            if v_digest == digest(&final_res[ii]) {
                is_duplicate = true;
            }
        }
        if !is_duplicate {
            final_res.push(res[i].clone());
        }
    }

    final_res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::digest_all;

    #[test]
    pub fn test_kokushimusou() {
        assert_eq!(
            check13(&Vec::from([
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 1, 2, 1, 1, 1, 1
            ])),
            true
        );

        assert_eq!(
            check13(&Vec::from([
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 1, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 1, 1, 1, 1, 1, 1
            ])),
            false
        );

        assert_eq!(
            check13(&Vec::from([
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 2, 0, 2, 0, 0, //
                2, 0, 0, 0, 0, 2, 0, 0, 0, //
                0, 0, 2, 0, 2, 0, 0
            ])),
            false
        );
    }

    #[test]
    pub fn test_chiitoitsu() {
        assert_eq!(
            check7(&Vec::from([
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 2, 0, 2, 0, 0, //
                2, 0, 0, 0, 0, 2, 0, 0, 0, //
                0, 0, 2, 0, 2, 0, 0
            ])),
            true
        );

        assert_eq!(
            check7(&Vec::from([
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 2, 0, 2, 0, 0, //
                1, 1, 0, 0, 0, 2, 0, 0, 0, //
                0, 0, 2, 0, 2, 0, 0
            ])),
            false
        );
    }

    #[test]
    pub fn test_basic_form() {
        let hands = [
            Vec::from([
                2, 2, 0, 2, 0, 0, 2, 2, 2, //
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 2, 1, 1, 1, 1, 1,
            ]),
            Vec::from([
                0, 0, 0, 0, 0, 2, 2, 2, 0, //
                0, 0, 0, 0, 0, 0, 1, 1, 1, //
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 3, 0,
            ]),
            Vec::from([
                2, 2, 2, 2, 0, 0, 2, 2, 2, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
        ];

        assert_eq!(check(&hands[0]), false);
        assert_eq!(check(&hands[1]), false);
        assert_eq!(check(&hands[2]), true);
        assert_eq!(check(&hands[3]), true);

        assert_eq!(check7(&hands[0]), true);
        assert_eq!(check7(&hands[1]), false);
        assert_eq!(check7(&hands[2]), false);
        assert_eq!(check7(&hands[3]), true);

        assert_eq!(check13(&hands[0]), false);
        assert_eq!(check13(&hands[1]), true);
        assert_eq!(check13(&hands[2]), false);
        assert_eq!(check13(&hands[3]), false);

        assert_eq!(check_all(&hands[0]), true);
        assert_eq!(check_all(&hands[1]), true);
        assert_eq!(check_all(&hands[2]), true);
        assert_eq!(check_all(&hands[3]), true);
    }

    #[test]
    pub fn test_find_kotsu() {
        let mut hand = Vec::from([
            0, 0, 0, 0, 3, 0, 0, 0, 0, //
            0, 0, 0, 3, 0, 0, 0, 2, 0, //
            0, 0, 3, 0, 0, 0, 0, 0, 0, //
            0, 3, 0, 0, 0, 0, 0,
        ]);
        let expected = Vec::from([
            Vec::from([5, 5, 5]),    //
            Vec::from([13, 13, 13]), //
            Vec::from([21, 21, 21]), //
            Vec::from([29, 29, 29]),
        ]);
        assert_eq!(find_kotsu(hand.as_mut()), expected);
    }

    #[test]
    pub fn test_find_shuntsu() {
        let mut hands = [
            Vec::from([
                1, 1, 1, 0, 0, 0, 1, 1, 1, //
                0, 0, 0, 1, 1, 1, 0, 2, 0, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 1, 2, 3, 2, 1, 0, 2, 0, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                0, 0, 0, 0, 1, 1, 1, 0, 1, //
                1, 1, 0, 1, 1, 1, 0, 2, 0, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 2, 0, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                0, 3, 3, 3, 0, 0, 0,
            ]),
        ];

        let expected = [
            Vec::from([
                Vec::from([1, 2, 3]),    //
                Vec::from([7, 8, 9]),    //
                Vec::from([13, 14, 15]), //
                Vec::from([21, 22, 23]),
            ]),
            Vec::from([
                Vec::from([11, 12, 13]), //
                Vec::from([12, 13, 14]), //
                Vec::from([13, 14, 15]), //
                Vec::from([21, 22, 23]),
            ]),
            Vec::from([]),
            Vec::from([Vec::from([21, 22, 23])]),
        ];

        assert_eq!(find_shuntsu(hands[0].as_mut()), expected[0]);
        assert_eq!(find_shuntsu(hands[1].as_mut()), expected[1]);
        assert_eq!(find_shuntsu(hands[2].as_mut()), expected[2]);
        assert_eq!(find_shuntsu(hands[3].as_mut()), expected[3]);
    }

    #[test]
    pub fn test_find_janto() {
        let hands = [
            Vec::from([
                1, 1, 1, 0, 0, 0, 1, 1, 1, //
                0, 0, 0, 1, 1, 1, 0, 2, 0, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                1, 1, 1, 0, 0, 0, 1, 1, 1, //
                0, 0, 0, 1, 1, 1, 0, 1, 1, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
        ];

        assert_eq!(find_janto(&hands[0], -1), 16);
        assert_eq!(find_janto(&hands[1], -1), -1);
    }

    #[test]
    pub fn test_find_all_agari_patterns() {
        let test_cases = [
            Vec::from([
                2, 2, 0, 2, 0, 0, 2, 2, 2, //
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 0, 0, 0, 0, 0, 0, 0, 1, //
                1, 2, 1, 1, 1, 1, 1,
            ]),
            Vec::from([
                0, 0, 0, 0, 0, 2, 2, 2, 0, //
                0, 0, 0, 0, 0, 0, 1, 1, 1, //
                0, 0, 2, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 3, 0,
            ]),
            Vec::from([
                2, 2, 2, 2, 0, 0, 2, 2, 2, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                0, 0, 0, 0, 0, 2, 2, 2, 2, //
                0, 0, 0, 0, 0, 0, 1, 1, 1, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 3, 0,
            ]),
            Vec::from([
                4, 4, 4, 2, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                3, 1, 1, 3, 0, 0, 0, 0, 0, //
                3, 0, 0, 0, 0, 0, 0, 0, 0, //
                3, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            Vec::from([
                0, 2, 2, 2, 2, 2, 2, 2, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0, 0, 0, //
                0, 0, 0, 0, 0, 0, 0,
            ]),
            // incomplete hand should be fine too
            Vec::from([
                1, 1, 1, 0, 0, 0, 0, 0, 0, //
                0, 1, 1, 1, 0, 0, 0, 0, 0, //
                0, 0, 1, 1, 1, 0, 0, 0, 0, //
                2, 0, 0, 0, 0, 0, 0, //
            ]),
        ];

        let expected = [
            digest_all(Vec::from([Vec::from([
                Vec::from([1, 1]),
                Vec::from([2, 2]),
                Vec::from([12, 12]),
                Vec::from([4, 4]),
                Vec::from([7, 7]),
                Vec::from([8, 8]),
                Vec::from([9, 9]),
            ])])),
            digest_all(Vec::from([Vec::from([Vec::from([
                1, 9, 10, 18, 19, 27, 28, 29, 29, 30, 31, 32, 33, 34,
            ])])])),
            digest_all(Vec::from([Vec::from([
                Vec::from([21, 21]),
                Vec::from([6, 7, 8]),
                Vec::from([6, 7, 8]),
                Vec::from([33, 33, 33]),
                Vec::from([16, 17, 18]),
            ])])),
            digest_all(Vec::from([
                Vec::from([
                    Vec::from([1, 1]),
                    Vec::from([2, 3, 4]),
                    Vec::from([2, 3, 4]),
                    Vec::from([7, 8, 9]),
                    Vec::from([7, 8, 9]),
                ]),
                Vec::from([
                    Vec::from([1, 2, 3]),
                    Vec::from([1, 2, 3]),
                    Vec::from([4, 4]),
                    Vec::from([7, 8, 9]),
                    Vec::from([7, 8, 9]),
                ]),
                Vec::from([
                    Vec::from([1, 1]),
                    Vec::from([2, 2]),
                    Vec::from([3, 3]),
                    Vec::from([4, 4]),
                    Vec::from([7, 7]),
                    Vec::from([8, 8]),
                    Vec::from([9, 9]),
                ]),
            ])),
            digest_all(Vec::from([
                Vec::from([
                    Vec::from([6, 6]),
                    Vec::from([33, 33, 33]),
                    Vec::from([7, 8, 9]),
                    Vec::from([7, 8, 9]),
                    Vec::from([16, 17, 18]),
                ]),
                Vec::from([
                    Vec::from([6, 7, 8]),
                    Vec::from([6, 7, 8]),
                    Vec::from([33, 33, 33]),
                    Vec::from([16, 17, 18]),
                    Vec::from([9, 9]),
                ]),
            ])),
            digest_all(Vec::from([
                Vec::from([
                    Vec::from([1, 1]),
                    Vec::from([1, 2, 3]),
                    Vec::from([1, 2, 3]),
                    Vec::from([2, 3, 4]),
                    Vec::from([2, 3, 4]),
                ]),
                Vec::from([
                    Vec::from([1, 1, 1]),
                    Vec::from([1, 2, 3]),
                    Vec::from([2, 2, 2]),
                    Vec::from([3, 3, 3]),
                    Vec::from([4, 4]),
                ]),
                Vec::from([
                    Vec::from([1, 2, 3]),
                    Vec::from([1, 2, 3]),
                    Vec::from([1, 2, 3]),
                    Vec::from([1, 2, 3]),
                    Vec::from([4, 4]),
                ]),
            ])),
            digest_all(Vec::from([
                Vec::from([
                    Vec::from([1, 1]),
                    Vec::from([1, 2, 3]),
                    Vec::from([10, 10, 10]),
                    Vec::from([19, 19, 19]),
                    Vec::from([4, 4, 4]),
                ]),
                Vec::from([
                    Vec::from([1, 1, 1]),
                    Vec::from([10, 10, 10]),
                    Vec::from([19, 19, 19]),
                    Vec::from([2, 3, 4]),
                    Vec::from([4, 4]),
                ]),
            ])),
            digest_all(Vec::from([
                Vec::from([
                    Vec::from([2, 2]),
                    Vec::from([3, 4, 5]),
                    Vec::from([3, 4, 5]),
                    Vec::from([6, 7, 8]),
                    Vec::from([6, 7, 8]),
                ]),
                Vec::from([
                    Vec::from([2, 3, 4]),
                    Vec::from([2, 3, 4]),
                    Vec::from([5, 5]),
                    Vec::from([6, 7, 8]),
                    Vec::from([6, 7, 8]),
                ]),
                Vec::from([
                    Vec::from([2, 3, 4]),
                    Vec::from([2, 3, 4]),
                    Vec::from([5, 6, 7]),
                    Vec::from([5, 6, 7]),
                    Vec::from([8, 8]),
                ]),
                Vec::from([
                    Vec::from([2, 2]),
                    Vec::from([3, 3]),
                    Vec::from([4, 4]),
                    Vec::from([5, 5]),
                    Vec::from([6, 6]),
                    Vec::from([7, 7]),
                    Vec::from([8, 8]),
                ]),
            ])),
            digest_all(Vec::from([Vec::from([
                Vec::from([1, 2, 3]),
                Vec::from([11, 12, 13]),
                Vec::from([21, 22, 23]),
                Vec::from([28, 28]),
            ])])),
        ];

        for i in 0..8 {
            assert_eq!(
                digest_all(find_all_agari_patterns(&test_cases[i])),
                expected[i]
            );
        }
    }
}
