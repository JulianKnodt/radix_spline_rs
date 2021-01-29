// Some convenient constants which can be made generic over later

/// When to use linear search instead of binary search
const LINEAR_THRESH: usize = 32;
/// Key Type
type T = u32;
/// Number of bits to use for radix
const RADIX_BITS: T = 10;
/// Precision to use for linear comparison
const PREC: f32 = f32::EPSILON;

#[derive(Clone, Copy, Debug, Default)]
struct Coordinate {
  x: T,
  y: f32,
}

/// Represents the oreintation between two intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
  Linear,
  Clockwise,
  Counterclockwise,
}


fn orient(dx1: f32, dy1: f32, dx2: f32, dy2: f32) -> Orientation {
  let e = (dy1 * dx2) - (dy2 * dx1);
  if e > PREC {
    Orientation::Clockwise
  } else if e < -PREC {
    Orientation::Counterclockwise
  } else {
    Orientation::Linear
  }
}

fn shift_bits(diff: T, radix_bits: T) -> T {
  return (std::mem::size_of::<T>() as T)
    .saturating_sub(radix_bits)
    .saturating_sub(diff.leading_zeros() as T);
}

#[derive(Default, Debug)]
pub struct Builder {
  min: T,
  max: T,

  shift_bits: T,

  max_error: f32,
  prev_prefix: usize,

  radix_table: Vec<usize>,
  spline_points: Vec<Coordinate>,

  // have to record points since not all will be in spline points
  num_points: usize,

  // last y value
  prev_y: f32,
  // last x added
  prev_x: T,

  // number of distinct values there are(is this one redundant?)
  distinct: usize,

  lower_limit: Coordinate,
  upper_limit: Coordinate,
  prev_point: Coordinate,
}

impl Builder {
  pub fn new(min: T, max: T) -> Self {
    if min > max {
      return Self::new(max, min)
    }
    let shift_bits = shift_bits(max - min, RADIX_BITS);
    let max_prefix = (max - min) >> shift_bits;
    Builder {
      min,
      max,
      shift_bits,
      radix_table: vec![0; 2 + (max_prefix as usize)],
      max_error: 32.,
      prev_x: min,
      ..Default::default()
    }
  }
  pub fn with_error(&mut self, err: f32) -> &mut Self {
    assert_eq!(self.num_points, 0, "Must assign error before adding items");
    self.max_error = err;
    self
  }
  pub fn build(mut self) -> RadixSpline {
    if self.num_points == 0 {
      return RadixSpline::default();
    }
    debug_assert!(self.prev_x <= self.max);
    if self.spline_points.last().unwrap().x != self.prev_x {
      self.add_key_to_spline(Coordinate {
        x: self.prev_x,
        y: self.prev_y,
      });
    }
    let l = self.radix_table.len();
    self.radix_table[(self.prev_prefix + 1).min(l - 1)..].fill(self.spline_points.len());

    let Builder {
      min,
      max,
      max_error,
      radix_table,
      spline_points,
      num_points,
      shift_bits,
      ..
    } = self;
    RadixSpline {
      min,
      max,
      shift_bits,
      radix_table,
      spline_points,
      num_points,
      max_error,
    }
  }
  pub fn push(&mut self, x: T) -> &mut Self {
    let y = if self.num_points == 0 {
      0.
    } else {
      self.prev_y + 1.
    };

    self.insert(x, y);

    self.num_points += 1;
    self.prev_x = x;
    self.prev_y = y;
    self
  }

  fn insert(&mut self, x: T, y: f32) -> &mut Self {
    debug_assert!(self.min <= x && x <= self.max);

    if self.num_points == 0 {
      self.distinct = 1;
      return self
        .add_key_to_spline(Coordinate { x, y })
        .set_prev_cdf(x, y);
    }

    if x == self.prev_x {
      return self;
    }

    self.distinct += 1;
    let max_err = self.max_error;
    let upper_y = y + max_err;
    let lower_y = (y - max_err).max(0.);

    if self.distinct == 2 {
      return self
        .set_upper_limit(x, upper_y)
        .set_lower_limit(x, lower_y)
        .set_prev_cdf(x, y);
    }

    let last = *self.spline_points.last().unwrap();

    debug_assert!(self.upper_limit.x >= last.x);
    debug_assert!(self.lower_limit.x >= last.x);
    debug_assert!(x >= last.x);
    let upper_limit_x_diff = (self.upper_limit.x as f32) - (last.x as f32);
    let lower_limit_x_diff = (self.lower_limit.x as f32) - (last.x as f32);
    let x_diff = (x - last.x) as f32;

    debug_assert!(self.upper_limit.y >= last.y);
    debug_assert!(y >= last.y);
    let upper_limit_y_diff = self.upper_limit.y - last.y;
    let lower_limit_y_diff = self.lower_limit.y - last.y;
    let y_diff = y - last.y;

    debug_assert_ne!(self.prev_point.x, last.x);

    if orient(upper_limit_x_diff, upper_limit_y_diff, x_diff, y_diff) != Orientation::Clockwise
      || orient(lower_limit_x_diff, lower_limit_y_diff, x_diff, y_diff)
        != Orientation::Counterclockwise
    {
      self
        .add_key_to_spline(self.prev_point)
        .set_upper_limit(x, upper_y)
        .set_lower_limit(x, lower_y);
    } else {
      // decrease upper limit if applicable
      let upper_y_diff = upper_y - last.y;
      if orient(upper_limit_x_diff, upper_limit_y_diff, x_diff, upper_y_diff)
        == Orientation::Clockwise
      {
        self.set_upper_limit(x, upper_y);
      }

      // increase lower limit if applicable
      let lower_y_diff = lower_y - last.y;
      if orient(lower_limit_x_diff, lower_limit_y_diff, x_diff, lower_y_diff)
        == Orientation::Counterclockwise
      {
        self.set_lower_limit(x, lower_y);
      }
    }
    self.set_prev_cdf(x, y)
  }

  fn set_prev_cdf(&mut self, x: T, y: f32) -> &mut Self {
    self.prev_point = Coordinate { x, y };
    self
  }
  fn set_upper_limit(&mut self, x: T, y: f32) -> &mut Self {
    self.upper_limit = Coordinate { x, y };
    self
  }
  fn set_lower_limit(&mut self, x: T, y: f32) -> &mut Self {
    self.lower_limit = Coordinate { x, y };
    self
  }

  fn add_key_to_spline(&mut self, coord: Coordinate) -> &mut Self {
    self.spline_points.push(coord);

    let curr_prefix = ((coord.x - self.min) >> self.shift_bits) as usize;

    if curr_prefix != self.prev_prefix {
      self.radix_table[self.prev_prefix + 1..=curr_prefix].fill(self.spline_points.len() - 1);
      self.prev_prefix = curr_prefix;
    }

    self
  }
}

#[derive(Debug, Default)]
pub struct RadixSpline {
  min: T,
  max: T,
  shift_bits: T,

  max_error: f32,
  num_points: usize,

  radix_table: Vec<usize>,
  spline_points: Vec<Coordinate>,
}

impl RadixSpline {
  /// returns range in data[start..end] where key might be.
  pub fn search_bound(&self, key: &T) -> (usize, usize) {
    let est = self.get_estimated_position(key);
    let start = (est - self.max_error).max(0.);
    let end = (est + self.max_error + 2.).min(self.num_points as f32);
    (start as usize, end as usize)
  }

  pub fn get_estimated_position(&self, key: &T) -> f32 {
    if key <= &self.min {
      return 0.;
    } else if key >= &self.max {
      return (self.num_points - 1) as f32;
    }
    let idx = self.spline_segment(key);
    if idx == 0 {
      return 0.;
    }
    let l = &self.spline_points[idx - 1];
    let r = &self.spline_points[idx];
    let slope = (r.y - l.y) / ((r.x - l.x) as f32);
    debug_assert!(slope > 0.);
    debug_assert!(key >= &l.x);

    slope * ((key - l.x) as f32) + l.y
  }

  // gets the index of the end of the spline which contains key: T.
  fn spline_segment(&self, key: &T) -> usize {
    let prefix: usize = ((key - self.min) >> self.shift_bits) as usize;
    debug_assert!(prefix + 1 < self.radix_table.len());
    let begin = self.radix_table[prefix];
    let end = self.radix_table[prefix + 1];
    debug_assert!(end >= begin);
    if end == begin {
      return begin;
    } else if end - begin < LINEAR_THRESH {
      return begin
        + self.spline_points[begin..end]
          .iter()
          .position(|v| &v.x >= key)
          .unwrap();
    }

    let lb = self.spline_points[begin..end].binary_search_by(|c| c.x.partial_cmp(key).unwrap());
    match lb {
      Ok(i) | Err(i) => i,
    }
  }
}
