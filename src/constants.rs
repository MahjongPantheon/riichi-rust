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

impl Mul<i8> for Suit {
    type Output = usize;

    fn mul(self, rhs: i8) -> Self::Output {
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

pub fn kokushi_idx() -> [i8; 13] {
    [
        Tiles::M1 as i8,
        Tiles::M9 as i8,
        Tiles::P1 as i8,
        Tiles::P9 as i8,
        Tiles::S1 as i8,
        Tiles::S9 as i8,
        Tiles::E as i8,
        Tiles::S as i8,
        Tiles::W as i8,
        Tiles::N as i8,
        Tiles::WD as i8,
        Tiles::GD as i8,
        Tiles::RD as i8,
    ]
}

pub fn sum(haipai: &Vec<i8>) -> i8 {
    haipai.iter().fold(0, |acc, v| acc + v)
}

pub fn ceil100(val: i32) -> i32 {
    100 * (val as f32 / 100.0).ceil() as i32
}

pub fn ceil10(val: i32) -> i32 {
    10 * (val as f32 / 10.0).ceil() as i32
}

pub fn slice_by_suit(haipai: &Vec<i8>) -> Vec<Vec<i8>> {
    vec![
        haipai[Suit::Man * 9..Suit::Man * 9 + 9].to_vec(),
        haipai[Suit::Pin * 9..Suit::Pin * 9 + 9].to_vec(),
        haipai[Suit::Sou * 9..Suit::Sou * 9 + 9].to_vec(),
        haipai[Suit::Honor * 9..Suit::Honor * 9 + 7].to_vec(),
    ]
}

pub fn is19(tile: i8) -> bool {
    kokushi_idx().contains(&tile)
}

pub fn is_proper_open_set(arr: &Vec<i8>) -> bool {
    if arr.len() > 4 || arr.len() < 2 {
        false
    } else {
        let set: HashSet<&i8> = HashSet::from_iter(arr.as_slice());
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

pub fn digest(decomposition: &Vec<Vec<i8>>) -> String {
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

pub static GREENS: [i8; 6] = [
    Tiles::S2 as i8,
    Tiles::S3 as i8,
    Tiles::S4 as i8,
    Tiles::S6 as i8,
    Tiles::S8 as i8,
    Tiles::GD as i8,
];
pub static WINDS: [i8; 4] = [
    Tiles::E as i8,
    Tiles::S as i8,
    Tiles::W as i8,
    Tiles::N as i8,
];
pub static HONORS: [i8; 7] = [
    Tiles::E as i8,
    Tiles::S as i8,
    Tiles::W as i8,
    Tiles::N as i8,
    Tiles::GD as i8,
    Tiles::RD as i8,
    Tiles::WD as i8,
];
pub static TERMINALS: [i8; 6] = [
    Tiles::M1 as i8,
    Tiles::M9 as i8,
    Tiles::S1 as i8,
    Tiles::S9 as i8,
    Tiles::P1 as i8,
    Tiles::P9 as i8,
];
pub static TERMINALS_AND_HONORS: [i8; 13] = [
    Tiles::M1 as i8,
    Tiles::M9 as i8,
    Tiles::S1 as i8,
    Tiles::S9 as i8,
    Tiles::P1 as i8,
    Tiles::P9 as i8,
    Tiles::E as i8,
    Tiles::S as i8,
    Tiles::W as i8,
    Tiles::N as i8,
    Tiles::GD as i8,
    Tiles::RD as i8,
    Tiles::WD as i8,
];
pub static CHI_START: [i8; 9] = [
    Tiles::M1 as i8,
    Tiles::M4 as i8,
    Tiles::M7 as i8,
    Tiles::P1 as i8,
    Tiles::P4 as i8,
    Tiles::P7 as i8,
    Tiles::S1 as i8,
    Tiles::S4 as i8,
    Tiles::S7 as i8,
];
pub static SIMPLE_TILES: [i8; 21] = [
    Tiles::M2 as i8,
    Tiles::M3 as i8,
    Tiles::M4 as i8,
    Tiles::M5 as i8,
    Tiles::M6 as i8,
    Tiles::M7 as i8,
    Tiles::M8 as i8,
    Tiles::P2 as i8,
    Tiles::P3 as i8,
    Tiles::P4 as i8,
    Tiles::P5 as i8,
    Tiles::P6 as i8,
    Tiles::P7 as i8,
    Tiles::P8 as i8,
    Tiles::S2 as i8,
    Tiles::S3 as i8,
    Tiles::S4 as i8,
    Tiles::S5 as i8,
    Tiles::S6 as i8,
    Tiles::S7 as i8,
    Tiles::S8 as i8,
];
