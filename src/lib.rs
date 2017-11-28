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

const REF_Y : f64   = 1.0;
const REF_U : f64   = 0.19783000664283;
const REF_V : f64   = 0.46831999493879;
// CIE LUV constants
const KAPPA : f64   = 903.2962962;
const EPSILON : f64 = 0.0088564516;

pub fn hsluv_to_hex(hsl: (f64, f64, f64)) -> String {
  rgb_to_hex(
    hsluv_to_rgb(hsl)
  )
}

pub fn hpluv_to_hex(hsl: (f64, f64, f64)) -> String {
  rgb_to_hex(
    hpluv_to_rgb(hsl)
  )
}

pub fn hex_to_hsluv(hex: &str) -> (f64, f64, f64) {
  rgb_to_hsluv(
    hex_to_rgb(hex)
  )
}

pub fn hex_to_hpluv(hex: &str) -> (f64, f64, f64) {
  rgb_to_hpluv(
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

pub fn rgb_to_hpluv(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
  lch_to_hpluv(rgb_to_lch(rgb))
}

pub fn rgb_to_lch(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
  luv_to_lch(
    xyz_to_luv(
      rgb_to_xyz(rgb)
    )
  )
}

pub fn rgb_to_xyz(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
  let rgbl = vec![to_linear(rgb.0), to_linear(rgb.1), to_linear(rgb.2)];
  let mapping : Vec<f64> = M_INV.iter().map(|i| dot_product(&i.to_vec(), &rgbl)).collect();
  (mapping[0], mapping[1], mapping[2])
}

pub fn luv_to_lch(luv: (f64, f64, f64)) -> (f64, f64, f64) {
  let (l, u, v) = luv;
  let c = (u*u + v*v).sqrt();
  if c < 0.00000001 {
    (l, c, 0.0)
  } else {
    let hrad = f64::atan2(v, u);
    let mut h = radians_to_degrees(hrad);
    if h < 0.0 {
      h += 360.0;
    }
    (l, c, h)
  }
}

pub fn rgb_to_hex(rgb: (f64, f64, f64)) -> String {
  let (r,g,b) = rgb_prepare(rgb);
  format!("#{:02x}{:02x}{:02x}", r,g,b)
}

pub fn hex_to_rgb(raw_hex: &str) -> (f64, f64, f64) {
  let hex = raw_hex.trim_left_matches('#');
  if hex.len() != 6 {
    println!("Not a hex string!");
    return (0.0,0.0,0.0)
  }
  let mut chunks = hex.as_bytes().chunks(2);
  let red = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
  let green = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
  let blue = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
  ( (red.unwrap_or(0) as f64) / 255.0, (green.unwrap_or(0) as f64) / 255.0, (blue.unwrap_or(0) as f64) / 255.0 )
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

fn dot_product(a: &[f64], b: &[f64] ) -> f64 {
  a.iter().zip(b.iter()).map(|(i, j)| i * j).sum()
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

  let mut retval : Vec<(f64, f64)> = Vec::new();
  
  for ms in &M {
    let (m1, m2, m3) = (ms[0], ms[1], ms[2]);
    for t in 0..2 {
      let top1 = (284517.0 * m1 - 94839.0 * m3) * sub2;
      let top2 = (838422.0 * m3 + 769860.0 * m2 + 731718.0 * m1) * l * sub2 - 769860.0 * f64::from(t) * l;
      let bottom = (632260.0 * m3 - 126452.0 * m2) * sub2 + 126452.0 * f64::from(t);

      retval.push((top1/bottom, top2/bottom));
    }
  }
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