## Riichi Rust library

Small library to calculate hands and yaku for japanese (riichi) mahjong.

For WebAssembly sources and builds, see the [repo](https://github.com/MahjongPantheon/riichi-rs) and npm packages for [node](https://npmjs.com/package/riichi-rs-node) and [bundlers](https://npmjs.com/package/riichi-rs-bundlers).

### Usage

Install the library by adding the following in your Cargo.toml:

```toml
[dependencies]
riichi-rust = { git = "https://github.com/MahjongPantheon/riichi-rust.git", version = "1.0.0" }
```

Use the library:

```rust
use riichi_rust::{RiichiHand, RiichiOptions, RiichiResult, Yaku, Tiles, calc_riichi};

pub fn main() {
    let mut options = RiichiOptions {
        dora: vec![Tiles::M3 as i8], // actual dora tiles (not indicators value)
        aka_count: 0, // count of akadora in hand
        first_take: false, // if this hand is completed on first take
        riichi: false, // if there was riichi declared
        ippatsu: false, // if ippatsu happened
        double_riichi: false, // if there was riichi declared on first turn.
        // Note that ankans are allowed, but checking if the ankan was declared before double riichi
        // (and voiding double riichi consequently) is responsibility of external code.

        after_kan: false, // chankan (on ron) or rinshan (on tsumo)
        tile_discarded_by_someone: -1, // Tile the hand won on. If tsumo, pass -1
        bakaze: Tiles::South as i8, // Round wind
        jikaze: Tiles::East as i8, // Seat wind
        allow_aka: true, // if akadora is allowed
        allow_kuitan: true, // if open tanyao is allowed
        with_kiriage: false, // if 4/30 and 3/60 hands are treated as mangan
        disabled_yaku: vec![Yaku::Renhou as i8], // List of yaku to be disabled
        local_yaku_enabled: vec![Yaku::Daisharin as i8], // List of local yaku to be enabled
        all_local_yaku_enabled: false, // pass true here to enable all supported local yaku
        allow_double_yakuman: false, // if double yakuman is allowed
        last_tile: false // haitei or houtei
    };

    let result = calc_riichi(
        RiichiHand {
            open_part: vec![
                Tiles::M1 as i8,
                Tiles::M2 as i8,
                Tiles::M3 as i8,
                Tiles::M4 as i8,
                Tiles::M5 as i8,
                Tiles::M6 as i8,
                Tiles::P9 as i8,
                Tiles::P9 as i8
            ],
            closed_part: vec![
                (true, vec![
                    Tiles::M7 as i8,
                    Tiles::M8 as i8,
                    Tiles::M9 as i8
                ]),
                (false, vec![
                    Tiles::P5 as i8,
                    Tiles::P5 as i8,
                    Tiles::P5 as i8,
                    Tiles::P5 as i8
                ])
            ],
        },
        &options,
        false, // calc_hairi:  if completions and discard variants should be calculated
    );

    match result {
        Ok(data) => {
            /*  do something with data; format of output matches RiichiResult struct */
        }
        Err(e) => println!("{}", e)
    }
}
```

### Credits

Inspired by and partially taken from following repositories:

- https://github.com/takayama-lily/riichi
- https://github.com/takayama-lily/syanten
- https://github.com/takayama-lily/agari

### Testing on real games data

The library was tested against over 19 million of hands from real-life game logs from Tenhou.net phoenix lobby. Though
we don't supply these logs in the repo, you can still download it on Tenhou.net and use it for testing. Please refer
to [this external file](https://github.com/MahjongPantheon/riichi-ts/blob/main/riichi_realdata_rs.test.ts) which uses
wasm-compiled version of this library for testing.
