#[macro_use]
extern crate log;

pub mod hsluv {
  use std::f64::consts::PI;
  use std::cmp::Ordering;
  use std::str; 

  const M: [[f64;3];3] = [
    [3.240969941904521, -1.537383177570093, -0.498610760293],
    [-0.96924363628087, 1.87596750150772, 0.041555057407175],
    [0.055630079696993, -0.20397695888897, 1.056971514242878],
  ];

  const M_INV: [[f64;3];3] = [
    [0.41239079926595, 0.35758433938387, 0.18048078840183],
    [0.21263900587151, 0.71516867876775, 0.072192315360733],
    [0.019330818715591, 0.11919477979462, 0.95053215224966],
  ];

  // TODO: this shows up in the Ruby source but is not used. What is it for?
  // const REF_X : f64   = 0.95045592705167;
  const REF_Y : f64   = 1.0;
  // TODO: this shows up in the Ruby source but is not used. What is it for?
  // const REF_Z : f64   = 1.089057750759878;
  const REF_U : f64   = 0.19783000664283;
  const REF_V : f64   = 0.46831999493879;
  const KAPPA : f64   = 903.2962962;
  const EPSILON : f64 = 0.0088564516;

  pub fn hsluv_to_hex(h: f64, s: f64, l: f64) -> String {
    // let hsl = (h, s, l);
    // println!("[hsluv_to_hex] input: {:?}", hsl);
    // let rgb = hsluv_to_rgb(hsl);
    // println!("[hsluv_to_rgb] out: {:?}", rgb);
    // let hex = rgb_to_hex(rgb);
    // println!("[hsluv_to_rgb] out: {:?}", hex);
    // hex

    rgb_to_hex(
      hsluv_to_rgb( (h, s, l) )
    )
  }

  pub fn hpluv_to_hex(h: f64, s: f64, l: f64) -> String {
    rgb_to_hex(
      hpluv_to_rgb( (h, s, l) )
    )
  }

  pub fn hex_to_hsluv(hex: &str) -> (f64, f64, f64) {
    rgb_to_hsluv(
      hex_to_rgb(hex)
    )
  }

  pub fn hsluv_to_lch(hsl: (f64, f64, f64)) ->(f64, f64, f64) {
    let (h, s, l) = hsl;
    match l {
      l if l > 99.9999999 => (100.0, 0.0, h),
      l if l < 0.00000001 => (0.0, 0.0, h),
      _ => {
        let mx = max_chroma_for(l, h);
        let c = mx/100.0 * s;
        (l, c, h)
      }
    }
  }

  pub fn hpluv_to_lch(hpl: (f64, f64, f64) ) -> (f64, f64, f64) {
    let (h, p, l) = hpl;
    match l {
      l if l > 99.9999999 => (100.0, 0.0, h),
      l if l < 0.00000001 => (0.0, 0.0, h),
      _ => {
        let mx = max_safe_chroma_for(l);
        let c = mx/100.0 * p;
        (l, c, h)
      }
    }
  }

  pub fn lch_to_luv(lch: (f64, f64, f64)) -> (f64, f64, f64) {
    let (l, c, h) = lch;
    let hrad = degrees_to_radians(h);
    let u = hrad.cos() * c;
    let v = hrad.sin() * c;

    (l, u, v)
  }

  pub fn lch_to_hsluv(lch: (f64, f64, f64)) -> (f64, f64, f64) {
    let (l, c, h) = lch;
    match l {
      l if l > 99.9999999 => (h, 0.0, 100.0),
      l if l < 0.00000001 => (h, 0.0, 0.0),
      _ => {
        let mx = max_chroma_for(l, h);
        // if l == 68.40444179723978 {
        // println!("[lch_to_hsluv] l: {:?}, h: {:?} -> mx: {:?}", l, h, mx);
        // println!("[lch_to_hsluv] c: {:?}, s: {:?}", c, (c / mx * 100.0));
        // }
        let s = c / mx * 100.0;
        (h, s, l)
      }
    }
  }

  pub fn lch_to_hpluv(lch: (f64, f64, f64)) -> (f64, f64, f64) {
    let (l, c, h) = lch;
    match l {
      l if l > 99.9999999 => (h, 0.0, 100.0),
      l if l < 0.00000001 => (h, 0.0, 0.0),
      _ => {
        let mx = max_safe_chroma_for(l);
        let s = c / mx * 100.0;
        (h, s, l)
      }
    }
  }

  pub fn lch_to_rgb(lch: (f64, f64, f64)) -> (f64, f64, f64) {
    xyz_to_rgb(
      luv_to_xyz(
        lch_to_luv(lch)
      )
    )
  }
  
  pub fn hsluv_to_rgb(hsl: (f64, f64, f64)) -> (f64, f64, f64) {
    xyz_to_rgb(
      luv_to_xyz(
        lch_to_luv(
          hsluv_to_lch(hsl)
        )
      )
    )
  }

  pub fn hpluv_to_rgb(hsl: (f64, f64, f64)) -> (f64, f64, f64) {
    lch_to_rgb(
      hpluv_to_lch(hsl)
    )
  }

  // TODO Switch to use a proper triplet type? Use Vec<f64>? Use arrays?
  pub fn xyz_to_rgb(xyz: (f64, f64, f64)) -> (f64, f64, f64) {
    let xyz_vec = vec![xyz.0, xyz.1, xyz.2];
    let abc: Vec<f64> = M.iter().map(|i| from_linear(dot_product(&i.to_vec(), &xyz_vec))).collect();
    (abc[0], abc[1], abc[2])
  }

  pub fn luv_to_xyz(luv: (f64, f64, f64)) -> (f64, f64, f64) {
    let (l, u, v) = luv;

    if l == 0.0 {
      return (0.0, 0.0, 0.0);
    }

    let var_y = f_inv(l);
    let var_u = u / (13.0 * l) + REF_U;
    let var_v = v / (13.0 * l) + REF_V;

    let y = var_y * REF_Y;
    let x = 0.0 - (9.0 * y * var_u) / ((var_u - 4.0) * var_v - var_u * var_v);
    let z = (9.0 * y - (15.0 * var_v * y) - (var_v * x)) / (3.0 * var_v);

    (x, y, z)
  }

  pub fn xyz_to_luv(xyz: (f64, f64, f64)) -> (f64, f64, f64) {
    let (x, y, z) = xyz;
    let l = f(y);

    if l == 0.0 || (xyz == (0.0, 0.0, 0.0)) {
      return (0.0, 0.0, 0.0);
    }

    let var_u = (4.0 * x) / (x + (15.0 * y) + (3.0 *z));
    let var_v = (9.0 * y) / (x + (15.0 * y) + (3.0 *z));
    let u = 13.0 * l * (var_u - REF_U);
    let v = 13.0 * l * (var_v - REF_V);
    
    (l, u, v)
  }

  pub fn rgb_to_hsluv(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
    lch_to_hsluv(rgb_to_lch(rgb))
  }

  pub fn rgb_to_lch(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
    luv_to_lch(
      xyz_to_luv(
        rgb_to_xyz(rgb)
      )
    )
  }

  fn rgb_to_xyz(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
    let rgbl = vec![to_linear(rgb.0), to_linear(rgb.1), to_linear(rgb.2)];
    let mapping : Vec<f64> = M_INV.iter().map(|i| dot_product(&i.to_vec(), &rgbl)).collect();
    (mapping[0], mapping[1], mapping[2])
  }

  fn luv_to_lch(luv: (f64, f64, f64)) -> (f64, f64, f64) {
    let (l, u, v) = luv;
    let c= (u.powi(2) + v.powi(2)).powf(0.5);
    let hrad = v.atan2(u);
    let mut h = radians_to_degrees(hrad);
    if h < 0.0 {
      h += 360.0;
    }
    (l, c, h)
  }

  fn f_inv(t: f64) -> f64 {
    if t > 8.0 {
      REF_Y * ( (t + 16.0) / 116.0 ).powf(3.0)
    } else {
      REF_Y * t / KAPPA
    }
  }

  fn to_linear(c: f64) -> f64 {
    if c > 0.04045 {
      ( (c + 0.055) / 1.055).powf(2.4)
    } else {
      c / 12.92
    }
  }

  fn from_linear(c: f64) -> f64 {
    if c <= 0.0031308 {
      12.92 * c
    } else {
      1.055 * (c.powf(1.0/2.4)) - 0.055
    }
  }

  fn f(t:f64) -> f64 {
    if t > EPSILON {
      116.0 * ( (t / REF_Y).powf(1.0/3.0) ) - 16.0
    } else {
      t / REF_Y * KAPPA
    }
  }
  
  fn dot_product(a: &Vec<f64>, b: &Vec<f64> ) -> f64 {
    a.iter().zip(b.iter()).map(|(i, j)| i * j).sum()
  }

  fn rgb_to_hex(rgb: (f64, f64, f64)) -> String {
    let (r,g,b) = rgb_prepare(rgb);
    // println!("[rgb_to_hex] as hex: #{:02x}{:02x}{:02x}", r,g,b);
    // println!("[rgb_to_hex] as dec: #{:?} {:?} {:?}", r,g,b);
    format!("#{:02x}{:02x}{:02x}", r,g,b)
  }

  fn rgb_prepare(rgb: (f64, f64, f64)) ->  (u8, u8, u8) {
    (clamp(rgb.0), clamp(rgb.1), clamp(rgb.2))
  }

  fn clamp(v: f64) -> u8 {
    let mut rounded = (v * 1000.0).round() / 1000.0;
    if rounded < 0.0 {
      rounded = 0.0;
    }
    if rounded > 1.0 {
      rounded = 1.0;
    }
    (rounded * 255.0).round() as u8
  }

  fn max_chroma_for(l: f64, h: f64) -> f64 {
    let hrad = h / 360.0 * PI * 2.0;

    // if l == 68.40444179723978 {
    //   let b = get_bounds(l);
    //   println!("Bounds: {:?}\nHrad: {:?}", b, hrad);
    // }
    let mut lengths : Vec<f64> = get_bounds(l)
      .iter()
      .map(|line| length_of_ray_until_intersect(hrad, line))
      .filter(|length| length > &0.0 )
      .collect::<Vec<f64>>();

      lengths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal) );
      lengths[0]
  }

  fn max_safe_chroma_for(l: f64) -> f64 {
    let mut lengths = Vec::new();
    get_bounds(l).iter().for_each(|line| {
      let x = intersect_line_line((line.0, line.1), (-1.0/line.0, 0.0) );
      lengths.push(distance_from_pole((x, line.1 + x * line.0)));
    });
    lengths.sort_by(|a,b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    lengths[0]
  }

  fn intersect_line_line(line1: (f64, f64), line2: (f64, f64)) -> f64 {
    (line1.1 - line2.1) / (line2.0 - line1.0)
  }

  fn distance_from_pole(point: (f64, f64)) -> f64 {
    (point.0.powi(2) + point.1.powi(2)).sqrt()
  }

  fn get_bounds(l: f64) -> Vec<(f64, f64)> {
    let sub1 = ((l + 16.0).powi(3))/ 1560896.0;
    let sub2 = match sub1 {
      s if s > EPSILON => s,
      _ => l / KAPPA
    };
    // if l == 68.40444179723978 {
    //   println!("\n[get_bounds] START for l: {:?}", l);
    //   println!("[get_bounds] sub1: {:?}, sub2: {:?}", sub1, sub2);
    // }

    let mut retval : Vec<(f64, f64)> = Vec::new();
    
    for ms in M.iter() {
      let (m1, m2, m3) = (ms[0], ms[1], ms[2]);
      for t in 0..2 {
        let top1 = (284517.0 * m1 - 94839.0 * m3) * sub2;
        let top2 = (838422.0 * m3 + 769860.0 * m2 + 731718.0 * m1) * l * sub2 - 769860.0 * (t as f64) * l;
        let bottom = (632260.0 * m3 - 126452.0 * m2) * sub2 + 126452.0 * (t as f64);
        // if l == 68.40444179723978 {
        //   println!("[get_bounds, {:?}] top1: {:?}, top2: {:?}, bottom: {:?} –> bound: {:?}", t, top1, top2, bottom, (top1/bottom, top2/bottom));
        // }

        retval.push((top1/bottom, top2/bottom));
      }
    }
    // println!("\n[get_bounds] END for l: {:?} –> {:?}\n", l, retval);
    retval
  }

  fn length_of_ray_until_intersect(theta: f64, line: &(f64, f64)) -> f64 {
    let (m1, b1) = *line;
    let length = b1 / (theta.sin() - m1 * theta.cos());
    if length < 0.0 {
      -0.0001
    } else {
      length
    }
  }

  fn radians_to_degrees(rad: f64) -> f64 {
    rad * 180.0 / PI
  }

  fn degrees_to_radians(deg: f64) -> f64 {
    deg * PI / 180.0
  }

  pub fn hex_to_rgb(raw_hex: &str) -> (f64, f64, f64) {
    let hex = raw_hex.trim_left_matches("#");
    // info!("Raw hex: {:?}; hex: {:?}", raw_hex, hex);
    if hex.len() != 6 {
      warn!("Not a hex string!");
      return (0.0,0.0,0.0)
    }
    let mut chunks = hex.as_bytes().chunks(2);
    let red = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
    let green = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
    let blue = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
    // info!("Chunks: {:?}, red: {:?}, green: {:?}, blue: {:?}", chunks, red, green, blue);
    ( (red.unwrap_or(0) as f64) / 255.0, (green.unwrap_or(0) as f64) / 255.0, (blue.unwrap_or(0) as f64) / 255.0 )
  }
  // hsluvToRgb([H, S, L]) -> [R, G, B]
  // hpluvToRgb([H, S, L]) -> [R, G, B]
  // rgbToHsluv([R, G, B]) -> [H, S, L]
  // rgbToHpluv([R, G, B]) -> [H, S, L]
}

extern crate spectral;

#[cfg(test)]
mod tests {
  extern crate env_logger;

  use spectral::prelude::*;
  use hsluv::*;

  static TOLLERANCE : f64 = 1e-11;

  #[cfg(test)]
  fn assert_is_close_enough(val: (f64, f64, f64), expected: (f64, f64, f64)) {
    let (v1, v2, v3) = val;
    let (e1, e2, e3) = expected;

    assert_that(&(v1 - e1).abs()).is_close_to(0.0, TOLLERANCE);
    assert_that(&(v2 - e2).abs()).is_close_to(0.0, TOLLERANCE);
    assert_that(&(v3 - e3).abs()).is_close_to(0.0, TOLLERANCE);
  }



  #[test]
  fn hsluv_to_hex_test() {
    // let out = hsluv_to_rgb((209.10944220554316, 90.59680473965321, 64.19296310551898));
    // println!("\nhsluv_to_rgb: {:?}", out);

    assert_eq!(hsluv_to_hex(
      209.10944220554316, 90.59680473965321, 64.19296310551898),
      "#33aabb"
    );

    assert_eq!(hsluv_to_hex(
      265.8743202181779, 100.00000000000087, 0.36553347952621895),
      "#000011"
    );

    assert_eq!(hsluv_to_hex(
      120.49801639863085, 87.90775515802211, 79.64261215477285),
      "#77dd44"
    );

    assert_eq!(hsluv_to_hex(
      349.0493316233723, 36.46630341431993, 68.40444179723978),
      "#cc99aa"
    );

    assert_eq!(hsluv_to_hex(
      77.56161374811363, 100.00000000000237, 75.06323349501214),
      "#ccbb00"
    );
  }

  #[test]
  fn hex_to_hsluv_test() {
    let _ = env_logger::init();

    assert_is_close_enough(hex_to_hsluv("#33aabb"),
      (209.10944220554316, 90.59680473965321, 64.19296310551898)
    );

    assert_is_close_enough(hex_to_hsluv("#33aabb"),
      (209.10944220554316, 90.59680473965321, 64.19296310551898)
    );

    assert_is_close_enough(hex_to_hsluv("#000011"),
      (265.8743202181779, 100.00000000000087, 0.36553347952621895)
    );

    assert_is_close_enough(hex_to_hsluv("#77dd44"),
      (120.49801639863085, 87.90775515802211, 79.64261215477285)
    );


    // let hex = "#cc99aa";
    // let rgb = hex_to_rgb(hex);
    // let lch = rgb_to_lch(rgb);
    // let hsluv = lch_to_hsluv(lch);
    // info!("#cc99aa to RGB: {:?}, \nexpected: {:?}", rgb, (0.8, 0.6, 0.6666666666666666));
    // info!("#cc99aa to LCH: {:?}, \nexpected: {:?}", lch, (68.40444179723978, 30.92001450189495, 349.0493316233723));
    // info!("#cc99aa to HSLUV: {:?}, \nexpected: {:?}", hsluv, (349.0493316233723, 36.46630341431993, 68.40444179723978));

    assert_is_close_enough(hex_to_hsluv("#cc99aa"),
      (349.0493316233723, 36.46630341431993, 68.40444179723978)
    );

    assert_is_close_enough(hex_to_hsluv("#ccbb00"),
      (77.56161374811363, 100.00000000000237, 75.06323349501214)
    );
  }

  #[test]
  fn hpluv_to_hex_test() {
    assert_eq!(hpluv_to_hex(
      209.10944220554316, 97.71423894082238, 64.19296310551898),
      "#33aabb"
    );

    assert_eq!(hpluv_to_hex(
      265.8743202181779, 513.4126968442804, 0.36553347952621895),
      "#000011"
    );

    assert_eq!(hpluv_to_hex(
      120.49801639863085, 196.89030323040726, 79.64261215477285),
      "#77dd44"
    );

    // let hsluv_to_lch_out = hsluv_to_lch( (349.0493316233723, 36.46630341431993, 68.40444179723978) );
    // println!("\n--> hsluv_to_lch OUT: {:?}\nexpected:             {:?}", hsluv_to_lch_out, (68.40444179723978, 30.92001450189495, 349.0493316233723));

    assert_eq!(hpluv_to_hex(
      349.0493316233723, 57.35809410920396, 68.40444179723978),
      "#cc99aa"
    );

    assert_eq!(hpluv_to_hex(
      77.56161374811363, 140.36658438875855, 75.06323349501214),
      "#ccbb00"
    );
  }
  
  #[test]
  fn hsluv_to_lch_test_zero_is_zero() {
    assert_eq!(hsluv_to_lch((0.0, 0.0, 0.0)), (0.0, 0.0, 0.0))
  }

  #[test]
  fn hsluv_to_lch_test() {
    // #000011
    assert_is_close_enough(hsluv_to_lch( 
      (265.8743202181779, 100.00000000000087, 0.36553347952621895) ), 
      (0.36553347952621895, 1.478953224866108, 265.8743202181779));

    // #000022
    assert_is_close_enough(hsluv_to_lch( 
      (265.8743202181779, 100.00000000000084, 1.0431351037401557) ), 
      (1.0431351037401557, 4.22053823263236, 265.8743202181779));

    // #33aabb
    assert_is_close_enough(hsluv_to_lch( 
      (209.10944220554316, 90.59680473965321, 64.19296310551898) ), 
      (64.19296310551898, 49.43174595065832, 209.10944220554316));
  }

  #[test]
  fn hpluv_to_lch_test() {
    // #000011
    assert_is_close_enough(hpluv_to_lch(
      (265.8743202181779, 513.4126968442804, 0.36553347952621895)),
      (0.36553347952621895, 1.478953224866108, 265.8743202181779)
    );

    // #000022
    assert_is_close_enough(hpluv_to_lch( 
      (265.8743202181779, 513.4126968442803, 1.0431351037401557) ), 
      (1.0431351037401557, 4.22053823263236, 265.8743202181779));

    // #33aabb
    assert_is_close_enough(hpluv_to_lch( 
      (209.10944220554316, 97.71423894082238, 64.19296310551898) ), 
      (64.19296310551898, 49.43174595065832, 209.10944220554316));
  }

  #[test]
  fn xyz_to_luv_test() {
    // #000011
    assert_is_close_enough(xyz_to_luv(
      (0.0010116654996371217, 0.0004046661998548544, 0.0053281049647556315)),
      (0.36553347952621895, -0.10640253083479542, -1.4751207214237791)
    );

    // #000022
    assert_is_close_enough(xyz_to_luv( 
      (0.002887023638114141, 0.0011548094552456725, 0.015204991160734828) ), 
      (1.0431351037401557, -0.3036444573679825, -4.20960128950726));

    // #33aabb
    assert_is_close_enough(xyz_to_luv( 
      (0.24707991872543905, 0.33039602584241434, 0.5209043845097113) ), 
      (64.19296310551898, -43.18812416055838, -24.047524596569566));
  }

  #[test]
  fn lch_to_hsluv_test() {
    // #000011
    assert_is_close_enough(lch_to_hsluv(
      (0.36553347952621895, 1.478953224866108, 265.8743202181779)),
      (265.8743202181779, 100.00000000000087, 0.36553347952621895)
    );

    // #000022
    assert_is_close_enough(lch_to_hsluv( 
      (1.0431351037401557, 4.22053823263236, 265.8743202181779) ), 
      (265.8743202181779, 100.00000000000084, 1.0431351037401557));

    // #33aabb
    assert_is_close_enough(lch_to_hsluv( 
      (64.19296310551898, 49.43174595065832, 209.10944220554316) ), 
      (209.10944220554316, 90.59680473965321, 64.19296310551898));
  }

  #[test]
  fn lch_to_hpluv_test() {
    // #000011
    assert_is_close_enough(lch_to_hpluv(
      (0.36553347952621895, 1.478953224866108, 265.8743202181779)),
      (265.8743202181779, 513.4126968442804, 0.36553347952621895)
    );

    // #000022
    assert_is_close_enough(lch_to_hpluv( 
      (1.0431351037401557, 4.22053823263236, 265.8743202181779) ), 
      (265.8743202181779, 513.4126968442803, 1.0431351037401557));

    // #33aabb
    assert_is_close_enough(lch_to_hpluv( 
      (1.0431351037401557, 4.22053823263236, 265.8743202181779) ), 
      (265.8743202181779, 513.4126968442803, 1.0431351037401557));
  }

  #[test]
  fn xyz_to_rgb_test() {
    // #000011
    assert_is_close_enough(xyz_to_rgb(
      (0.0010116654996371217, 0.0004046661998548544, 0.0053281049647556315)),
      (0.0, 0.0, 0.06666666666666667)
    );

    // #000022
    assert_is_close_enough(xyz_to_rgb( 
      (0.002887023638114141, 0.0011548094552456725, 0.015204991160734828) ), 
      (0.0, 0.0, 0.13333333333333333));

    // #33aabb
    assert_is_close_enough(xyz_to_rgb( 
      (0.24707991872543905, 0.33039602584241434, 0.5209043845097113) ), 
      (0.2, 0.6666666666666666, 0.7333333333333333));
  }

  #[test]
  fn hsluv_to_rgb_test() {
    // #000011
    assert_is_close_enough(hsluv_to_rgb(
      (265.8743202181779, 100.00000000000087, 0.36553347952621895)),
      (0.0, 0.0, 0.06666666666666667)
    );

    // #000022
    assert_is_close_enough(hsluv_to_rgb( 
      (265.8743202181779, 100.00000000000084, 1.0431351037401557) ), 
      (0.0, 0.0, 0.13333333333333333));

    // #33aabb
    assert_is_close_enough(hsluv_to_rgb( 
      (209.10944220554316, 90.59680473965321, 64.19296310551898) ), 
      (0.2, 0.6666666666666666, 0.7333333333333333));
  }

  #[test]
  fn luv_to_xyz_test() {
    // #33aabb
    assert_is_close_enough(luv_to_xyz(
      (64.19296310551898, -43.18812416055838, -24.047524596569566) ),
      (0.24707991872543905, 0.33039602584241434, 0.5209043845097113)
    )
  }

  #[test]
  fn lch_to_luv_test() {
    // #33aabb
    assert_is_close_enough(lch_to_luv(
      (64.19296310551898, 49.43174595065832, 209.10944220554316) ),
      (64.19296310551898, -43.18812416055838, -24.047524596569566)
    )
  }
}
