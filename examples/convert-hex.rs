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