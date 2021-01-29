use radix_spline::Builder;

fn main() {
  let mut vs = (0..10000)
    .map(|v| ((v as f32 * 377.98).fract().sin() + 1.) * 4500.)
    .map(|v| v as u32)
    .collect::<Vec<_>>();
  vs.push(8128);
  vs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

  let mut b = Builder::new(vs[0], *vs.last().unwrap());
  for &v in &vs[..] {
    b.push(v);
  }
  let rs = b.build();
  let (start, end) = rs.search_bound(&8128);
  assert!(&vs[start..end].contains(&8128), "{:?}", &vs[start..end])
}
