A Rust implementation of [HSLuv](http://www.hsluv.org).

## Demo

![Demo](http://i.imgur.com/GTsNT8u.gif)

Demo link: http://www.hsluv.org/syntax/#006925

## Installation

Add this line to your application's Cargo.toml:

```toml
[dependencies]
hsluv = "0.1.2"
```

## Usage

- `hue` is a 64bit float between 0 and 360
- `saturation` is a 64bit float between 0 and 100
- `lightness` is a 64bit float between 0 and 100
- `hex` is the hexadecimal format of the color
- `red` is a 64bit float between 0 and 1
- `green` is a 64bit float between 0 and 1
- `blue` is a 64bit float between 0 and 1

```rust
extern crate hsluv;

use hsluv::*;

fn main() {
  let hex = "#ab3912";
  let hsluv = hex_to_hsluv(hex);
  let rgb = hsluv_to_rgb(hsluv);

  println!("Convert HEX {:?} to HSLUV: {:?}", hex, hsluv);
  println!("Convert HSLUV {:?} to RGB: {:?}", hsluv, rgb);
  println!("Convert RGB {:?} to HEX: {:?}", rgb, rgb_to_hex(rgb));
}
```

All API calls take and return a 3-tuple of 64bit floats, `(f64, f64, f64)` except the `*_to_hex()` functions: `hsluv_to_hex()`, `hpluv_to_hex()`, `rgb_to_hex()` which return `String`.

#### hsluv::hsluv_to_hex((hue, saturation, lightness)) -> color as a hex string

```rust
hsluv::hsluv_to_hex((12.177, 100.0, 53.23)) // => #ff0000
```

#### hsluv::hsluv_to_rgb((hue, saturation, lightness)) -> color as rgb

```rust
hsluv::hsluv_to_rgb(12.177, 100.0, 53.23) // => (0.9998643703868711, 0.00000000000006849859221502719, 0.000008791283550555612)
```

#### hsluv::hex_to_hsluv(hex) -> list of floats as defined above

```rust
hsluv::hex_to_hsluv("#ff0000") // => (12.177050630061776, 100.0000000000022, 53.23711559542933)

```

#### hsluv::rgb_to_hsluv(rgb) -> list of floats as defined above

```rust
hsluv::rgb_to_hsluv((0.99, 6.84e-14, 8.79e-16)) // => (12.17705063006216, 100.00000000000209, 52.711595266911985)
```

For HPLuv (the pastel variant), use:

  - `hpluv_to_hex`
  - `hpluv_to_rgb`
  - `hex_to_hpluv`
  - `rgb_to_hpluv`

## Testing

Run `cargo test`.

## Contributing

1. Fork it ( http://github.com/dvdplm/hsluv-rust )
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request