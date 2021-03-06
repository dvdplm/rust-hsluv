#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod tests {
  extern crate hsluv;
  extern crate serde;
  extern crate serde_json;

  use self::serde_json::Error;
  use std::collections::HashMap;


  #[derive(Deserialize, Debug)]
  struct ColorTest {
      rgb: (f64, f64, f64),
      xyz: (f64, f64, f64),
      luv: (f64, f64, f64),
      lch: (f64, f64, f64),
      hsluv: (f64, f64, f64),
      hpluv: (f64, f64, f64),
  }

  use std::fs::File;
  use std::io::Read;

  static TOLERANCE : f64 = 1e-11;

  fn assert_is_close_enough(val: (f64, f64, f64), expected: (f64, f64, f64)) {
    let (v1, v2, v3) = val;
    let (e1, e2, e3) = expected;

    let dev1 = (v1 - e1).abs();
    let dev2 = (v2 - e2).abs();
    let dev3 = (v3 - e3).abs();
    if dev1 >= TOLERANCE || dev2 >= TOLERANCE || dev3 >= TOLERANCE {
      // println!("\nValue is deviating.\nvalue:    {:?}\nexpected: {:?}\ndeviation: {:?}", val, expected, (dev1, dev2, dev3))
      panic!("value {:?} deviates too much from the expected: {:?}", val, expected);
    }
  }

  fn load_test_json_data() -> Result<HashMap<String, ColorTest>, Error> {
    const DATA_FILENAME : &str = "./tests/snapshot-rev4.json";
    let mut file = File::open(DATA_FILENAME).expect(&format!("Can't load '{:?}'.", DATA_FILENAME));

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let colors: HashMap<String, ColorTest> = serde_json::from_str(&data)?;
    Ok(colors)
  }

  #[test]
  fn test_reference_data() {
    let colors = match load_test_json_data() {
      Ok(colors) => {
        println!("Loaded data for {:?} colors", colors.len());
        colors
      },
      Err(err) => panic!("Ouch, not ok. Error: {:?}", err)
    };

    for (hex, c) in colors {
      assert_is_close_enough(hsluv::rgb_to_xyz(c.rgb), c.xyz);
      assert_is_close_enough(hsluv::xyz_to_luv(c.xyz), c.luv);
      assert_is_close_enough(hsluv::luv_to_lch(c.luv), c.lch);
      assert_is_close_enough(hsluv::lch_to_hsluv(c.lch), c.hsluv);

      // backward
      assert_is_close_enough(hsluv::lch_to_hpluv(c.lch), c.hpluv);
      assert_is_close_enough(hsluv::hpluv_to_lch(c.hpluv), c.lch);
      assert_is_close_enough(hsluv::hsluv_to_lch(c.hsluv), c.lch);
      assert_is_close_enough(hsluv::lch_to_luv(c.lch), c.luv);
      assert_is_close_enough(hsluv::luv_to_xyz(c.luv), c.xyz);
      assert_is_close_enough(hsluv::xyz_to_rgb(c.xyz), c.rgb);

      // Others
      assert_eq!(hsluv::hsluv_to_hex(c.hsluv), hex);
      assert_eq!(hsluv::hpluv_to_hex(c.hpluv), hex);
      assert_is_close_enough(hsluv::hex_to_hsluv(&hex), c.hsluv);
      assert_is_close_enough(hsluv::hex_to_hpluv(&hex), c.hpluv);
    }
  }
}
