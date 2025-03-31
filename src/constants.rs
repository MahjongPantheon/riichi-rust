use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::{Add, Mul};

#[derive(Clone)]
pub enum Suit {
    Man = 0,
    Pin = 1,
    Sou = 2,
    Honor = 3,
}

#[derive(Clone)]
pub enum Val {
    N1 = 1,
    N2 = 2,
    N3 = 3,
    N4 = 4,
    N5 = 5,
    N6 = 6,
    N7 = 7,
    N8 = 8,
    N9 = 9,
}

#[derive(Clone)]
pub enum Retval {
    Ok = -1,
    Fail = -2,
    WrongCount = -3,
}

impl Mul<i32> for Suit {
    type Output = usize;

    fn mul(self, rhs: i32) -> Self::Output {
        self as usize * rhs as usize
    }
}

impl Add<Val> for usize {
    type Output = usize;

    fn add(self, rhs: Val) -> Self::Output {
        self + rhs as usize
    }
}

#[derive(Clone)]
pub enum Tiles {
    M1 = 1,
    M2 = 2,
    M3 = 3,
    M4 = 4,
    M5 = 5,
    M6 = 6,
    M7 = 7,
    M8 = 8,
    M9 = 9,

    P1 = 10,
    P2 = 11,
    P3 = 12,
    P4 = 13,
    P5 = 14,
    P6 = 15,
    P7 = 16,
    P8 = 17,
    P9 = 18,

    S1 = 19,
    S2 = 20,
    S3 = 21,
    S4 = 22,
    S5 = 23,
    S6 = 24,
    S7 = 25,
    S8 = 26,
    S9 = 27,

    E = 28,
    S = 29,
    W = 30,
    N = 31,
    WD = 32,
    GD = 33,
    RD = 34,
}

#[derive(Clone, PartialOrd, PartialEq)]
pub enum Yaku {
    Kokushimusou13Sides = 0,
    Kokushimusou = 1,
    Chuurenpoto9Sides = 2,
    Chuurenpoto = 3,
    SuuankouTanki = 4,
    Suuankou = 5,
    Daisuushi = 6,
    Shosuushi = 7,
    Daisangen = 8,
    Tsuuiisou = 9,
    Ryuuiisou = 10,
    Chinroutou = 11,
    Suukantsu = 12,
    Tenhou = 13,
    Chihou = 14,
    Renhou = 15,
    Daisharin = 16,
    Chinitsu = 17,
    Honitsu = 18,
    Ryanpeikou = 19,
    Junchan = 20,
    Chanta = 21,
    Toitoi = 22,
    Honroutou = 23,
    Sankantsu = 24,
    Shosangen = 25,
    SanshokuDoukou = 26,
    Sanankou = 27,
    Chiitoitsu = 28,
    DaburuRiichi = 29,
    Ittsu = 30,
    Sanshoku = 31,
    Tanyao = 32,
    Pinfu = 33,
    Iipeikou = 34,
    Menzentsumo = 35,
    Riichi = 36,
    Ippatsu = 37,
    Rinshan = 38,
    Chankan = 39,
    Haitei = 40,
    Houtei = 41,
    RoundWindEast = 42,
    RoundWindSouth = 43,
    RoundWindWest = 44,
    RoundWindNorth = 45,
    OwnWindEast = 46,
    OwnWindSouth = 47,
    OwnWindWest = 48,
    OwnWindNorth = 49,
    Haku = 50,
    Hatsu = 51,
    Chun = 52,

    Dora = 53,
    Uradora = 54,
    Akadora = 55,
}

impl Yaku {
    pub(crate) fn from_i32(value: i32) -> Yaku {
        match value {
            0 => Yaku::Kokushimusou13Sides,
            1 => Yaku::Kokushimusou,
            2 => Yaku::Chuurenpoto9Sides,
            3 => Yaku::Chuurenpoto,
            4 => Yaku::SuuankouTanki,
            5 => Yaku::Suuankou,
            6 => Yaku::Daisuushi,
            7 => Yaku::Shosuushi,
            8 => Yaku::Daisangen,
            9 => Yaku::Tsuuiisou,
            10 => Yaku::Ryuuiisou,
            11 => Yaku::Chinroutou,
            12 => Yaku::Suukantsu,
            13 => Yaku::Tenhou,
            14 => Yaku::Chihou,
            15 => Yaku::Renhou,
            16 => Yaku::Daisharin,
            17 => Yaku::Chinitsu,
            18 => Yaku::Honitsu,
            19 => Yaku::Ryanpeikou,
            20 => Yaku::Junchan,
            21 => Yaku::Chanta,
            22 => Yaku::Toitoi,
            23 => Yaku::Honroutou,
            24 => Yaku::Sankantsu,
            25 => Yaku::Shosangen,
            26 => Yaku::SanshokuDoukou,
            27 => Yaku::Sanankou,
            28 => Yaku::Chiitoitsu,
            29 => Yaku::DaburuRiichi,
            30 => Yaku::Ittsu,
            31 => Yaku::Sanshoku,
            32 => Yaku::Tanyao,
            33 => Yaku::Pinfu,
            34 => Yaku::Iipeikou,
            35 => Yaku::Menzentsumo,
            36 => Yaku::Riichi,
            37 => Yaku::Ippatsu,
            38 => Yaku::Rinshan,
            39 => Yaku::Chankan,
            40 => Yaku::Haitei,
            41 => Yaku::Houtei,
            42 => Yaku::RoundWindEast,
            43 => Yaku::RoundWindSouth,
            44 => Yaku::RoundWindWest,
            45 => Yaku::RoundWindNorth,
            46 => Yaku::OwnWindEast,
            47 => Yaku::OwnWindSouth,
            48 => Yaku::OwnWindWest,
            49 => Yaku::OwnWindNorth,
            50 => Yaku::Haku,
            51 => Yaku::Hatsu,
            52 => Yaku::Chun,
            53 => Yaku::Dora,
            54 => Yaku::Uradora,
            55 => Yaku::Akadora,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

pub fn kokushi_idx() -> [i32; 13] {
    [
        Tiles::M1 as i32,
        Tiles::M9 as i32,
        Tiles::P1 as i32,
        Tiles::P9 as i32,
        Tiles::S1 as i32,
        Tiles::S9 as i32,
        Tiles::E as i32,
        Tiles::S as i32,
        Tiles::W as i32,
        Tiles::N as i32,
        Tiles::WD as i32,
        Tiles::GD as i32,
        Tiles::RD as i32,
    ]
}

pub fn idx(suit: Suit, val: Val) -> usize {
    suit as usize * 9 + (val as i32 - 1) as usize
}

pub fn sum(haipai: &Vec<i32>) -> i32 {
    haipai.iter().fold(0, |acc, v| acc + v)
}

pub fn ceil100(val: i32) -> i32 {
    100 * (val as f32 / 100.0).ceil() as i32
}

pub fn ceil10(val: i32) -> i32 {
    10 * (val as f32 / 10.0).ceil() as i32
}

pub fn slice_by_suit(haipai: &Vec<i32>) -> Vec<Vec<i32>> {
    vec![
        haipai[Suit::Man * 9..Suit::Man * 9 + 9].to_vec(),
        haipai[Suit::Pin * 9..Suit::Pin * 9 + 9].to_vec(),
        haipai[Suit::Sou * 9..Suit::Sou * 9 + 9].to_vec(),
        haipai[Suit::Honor * 9..Suit::Honor * 9 + 7].to_vec(),
    ]
}

pub fn is19(tile: i32) -> bool {
    kokushi_idx().contains(&tile)
}

pub fn is_proper_open_set(arr: &Vec<i32>) -> bool {
    if arr.len() > 4 || arr.len() < 2 {
        false
    } else {
        let set: HashSet<&i32> = HashSet::from_iter(arr.as_slice());
        if set.len() == 1 {
            true
        } else {
            if set.len() != 3 {
                false
            } else {
                let minus1 = arr[1] - arr[0];
                let minus2 = arr[2] - arr[1];
                if minus1 != minus2 || minus1 != 1 {
                    false
                } else {
                    true
                }
            }
        }
    }
}

pub fn digest(decomposition: &Vec<Vec<i32>>) -> String {
    let mut arr = decomposition
        .iter()
        .map(|set| {
            format!(
                "|{}|",
                set.iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        })
        .collect::<Vec<String>>();
    arr.sort();
    arr.join("#")
}

pub fn digest_all(decompositions: Vec<Vec<Vec<i32>>>) -> String {
    decompositions
        .iter()
        .map(|set| digest(&set.clone()))
        .collect::<Vec<String>>()
        .join("$")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn idx_works() {
        let hand = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, //
            10, 11, 12, 13, 14, 15, 16, 17, 18, //
            19, 20, 21, 22, 23, 24, 25, 26, 27, //
            28, 29, 30, 31, 32, 33, 34,
        ];

        assert_eq!(hand[idx(Suit::Man, Val::N1)], 1);
        assert_eq!(hand[idx(Suit::Man, Val::N7)], 7);
        assert_eq!(hand[idx(Suit::Pin, Val::N7)], 16);
        assert_eq!(hand[idx(Suit::Pin, Val::N9)], 18);
        assert_eq!(hand[idx(Suit::Sou, Val::N3)], 21);
        assert_eq!(hand[idx(Suit::Sou, Val::N7)], 25);
        assert_eq!(hand[idx(Suit::Honor, Val::N2)], 29);
        assert_eq!(hand[idx(Suit::Honor, Val::N6)], 33);
    }
}

pub static GREENS: [i32; 6] = [
    Tiles::S2 as i32,
    Tiles::S3 as i32,
    Tiles::S4 as i32,
    Tiles::S6 as i32,
    Tiles::S8 as i32,
    Tiles::GD as i32,
];
pub static WINDS: [i32; 4] = [
    Tiles::E as i32,
    Tiles::S as i32,
    Tiles::W as i32,
    Tiles::N as i32,
];
pub static HONORS: [i32; 7] = [
    Tiles::E as i32,
    Tiles::S as i32,
    Tiles::W as i32,
    Tiles::N as i32,
    Tiles::GD as i32,
    Tiles::RD as i32,
    Tiles::WD as i32,
];
pub static TERMINALS: [i32; 6] = [
    Tiles::M1 as i32,
    Tiles::M9 as i32,
    Tiles::S1 as i32,
    Tiles::S9 as i32,
    Tiles::P1 as i32,
    Tiles::P9 as i32,
];
pub static TERMINALS_AND_HONORS: [i32; 13] = [
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
pub static CHI_START: [i32; 9] = [
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
pub static SIMPLE_TILES: [i32; 21] = [
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
