use radix_spline::{RadixSpline, Builder};
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

fn radix_spline() -> (RadixSpline, Vec<u32>) {
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
  (b.build(), vs)
}

#[quickcheck]
fn query_any(a: u32) -> bool {
  let (rs, vs) = radix_spline();
  let (start, end) = rs.search_bound(&a);
  vs[start..end].contains(&a) == vs.contains(&a)
}

#[quickcheck]
fn query_in(a: u32) -> bool {
  let (rs, vs) = radix_spline();
  let idx = (vs.len() as f32) * (1.+(a as f32).sin())/2.;
  let a = vs[(idx as usize).min(vs.len() - 1)];
  let (start, end) = rs.search_bound(&a);
  vs[start..end].contains(&a) == vs.contains(&a)
}
