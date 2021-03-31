//! Functions to get correspondence between two sequences like diff,
//! based on Myers' algorithm.
#[cfg(test)]
mod tests;
use std::cmp::{max, min};
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
use std::isize::{MAX, MIN};

/// An alias for the result of diff type
pub type Diff = Vec<Option<usize>>;

struct Difference<'a, X, Y> {
    xv: &'a [X],
    yv: &'a [Y],

    // working memory for forward path
    vf: Vec<isize>,
    vfm: Vec<isize>,
    // working memory for backward path
    vb: Vec<isize>,
    vbm: Vec<isize>,

    // edit script for xv
    xe: Vec<Option<usize>>,
    // edit script for yv
    ye: Vec<Option<usize>>,
}

impl<'a, X, Y> Difference<'a, X, Y>
where
    X: PartialEq<Y>,
{
    fn new(xv: &'a [X], yv: &'a [Y]) -> Self {
        let xe = vec![None; xv.len()];
        let ye = vec![None; yv.len()];
        Self {
            xv,
            yv,
            xe,
            ye,
            vf: vec![MIN; xv.len() / 10],
            vfm: vec![MIN; xv.len() / 10],
            vb: vec![MIN; yv.len() / 10],
            vbm: vec![MIN; yv.len() / 10],
        }
    }

    fn diff(&mut self) -> usize {
        self.diff_part((0, self.xv.len()), (0, self.yv.len()))
    }

    fn setv(&mut self, k: isize, v: isize, forward: bool) {
        let default = if forward { MIN } else { MAX };
        let (vec, i) = match (forward, k) {
            (true, k) if k >= 0 => (&mut self.vf, k),
            (true, k) => (&mut self.vfm, -k - 1),
            (false, k) if k >= 0 => (&mut self.vb, k),
            (false, k) => (&mut self.vbm, -k - 1),
        };
        let i = i as usize;
        // extend
        for _ in 0..(i + 1).saturating_sub(vec.len()) {
            vec.push(default)
        }
        vec[i] = v;
    }
    fn getv(&mut self, k: isize, forward: bool) -> Option<isize> {
        let (vec, i) = match (forward, k) {
            (true, k) if k >= 0 => (&self.vf, k),
            (true, k) => (&self.vfm, -k - 1),
            (false, k) if k >= 0 => (&self.vb, k),
            (false, k) => (&self.vbm, -k - 1),
        };
        vec.get(i as usize).cloned()
    }

    fn diff_part(
        &mut self,
        (mut xl, mut xr): (usize, usize),
        (mut yl, mut yr): (usize, usize),
    ) -> usize {
        // shrink by equality
        while xl < xr && yl < yr && self.xv[xl] == self.yv[yl] {
            self.xe[xl] = Some(yl);
            self.ye[yl] = Some(xl);
            xl += 1;
            yl += 1;
        }
        // same as backward
        while xl < xr && yl < yr && self.xv[xr - 1] == self.yv[yr - 1] {
            xr -= 1;
            yr -= 1;
            self.xe[xr] = Some(yr);
            self.ye[yr] = Some(xr);
        }

        // process simple case
        if xl == xr {
            self.ye[yl..yr].iter_mut().for_each(|x| *x = None);
            yr - yl
        } else if yl == yr {
            self.xe[xl..xr].iter_mut().for_each(|x| *x = None);
            xr - xl

        // divide and conquer
        } else {
            let (d, (xm, ym)) = self.find_mid((xl, xr), (yl, yr));
            self.diff_part((xl, xm), (yl, ym));
            self.diff_part((xm, xr), (ym, yr));
            d
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn find_mid(
        &mut self,
        (xl, xr): (usize, usize),
        (yl, yr): (usize, usize),
    ) -> (usize, (usize, usize)) {
        let xl = xl as isize;
        let xr = xr as isize;
        let yl = yl as isize;
        let yr = yr as isize;

        let kmin = xl - yr;
        let kmax = xr - yl;
        let kmidf = xl - yl; // center diag in this fragment for forwad snake
        let kmidb = xr - yr;
        let delta = (xr - xl) - (yr - yl);
        let is_odd = (delta & 1) == 1;

        self.setv(kmidf, xl, true);
        self.setv(kmidb, xr, false);

        let mut kminf = kmidf;
        let mut kmaxf = kmidf;
        let mut kminb = kmidb;
        let mut kmaxb = kmidb;

        let gety = |x: isize, k: isize| x.saturating_sub(k);

        for d in 1i64.. {
            // We don't have to check the case `d == 0` because it is handled in `fn diff_part`

            // forward
            {
                // update range
                if kminf > kmin {
                    kminf -= 1;
                    self.setv(kminf - 1, MIN, true);
                } else {
                    kminf += 1;
                }
                if kmaxf < kmax {
                    kmaxf += 1;
                    self.setv(kmaxf + 1, MIN, true);
                } else {
                    kmaxf -= 1
                }

                for k in (kminf..=kmaxf).rev().step_by(2) {
                    let x = {
                        let lo = self.getv(k - 1, true);
                        let hi = self.getv(k + 1, true);
                        max(lo.map(|x| x + 1), hi).unwrap()
                    };
                    let y = gety(x, k);
                    if !(xl <= x && x <= xr && yl <= y && y <= yr) {
                        continue;
                    }

                    // go forward in diagonal path
                    let (u, v) = {
                        let mut u = x;
                        let mut v = y;
                        let len = self.xv[u as usize..xr as usize]
                            .iter()
                            .zip(self.yv[v as usize..yr as usize].iter())
                            .take_while(|(x, y)| x == y)
                            .count() as isize;
                        u += len;
                        v += len;
                        (u, v)
                    };

                    debug_assert!(xl <= u && u <= xr);
                    debug_assert!(yl <= v && v <= yr);

                    self.setv(k, u, true);
                    if is_odd
                        && kminb <= k
                        && k <= kmaxb
                        && self.getv(k, false).map(|x| x <= u).unwrap_or(false)
                    {
                        return (2 * d as usize - 1, (x as usize, y as usize));
                    }
                }
            }

            // backward
            {
                // update range
                if kminb > kmin {
                    kminb -= 1;
                    self.setv(kminb - 1, MAX, false);
                } else {
                    kminb += 1;
                }
                if kmaxb < kmax {
                    kmaxb += 1;
                    self.setv(kmaxb + 1, MAX, false);
                } else {
                    kmaxb -= 1
                }

                for k in (kminb..=kmaxb).rev().step_by(2) {
                    let x = {
                        let lo = self.getv(k - 1, false);
                        let hi = self.getv(k + 1, false);
                        match (lo, hi.map(|x| x - 1)) {
                            (Some(lo), Some(hi)) => min(lo, hi),
                            (Some(lo), _) => lo,
                            (_, Some(hi)) => hi,
                            _ => unreachable!(),
                        }
                    };
                    let y = gety(x, k);
                    if !(xl <= x && x <= xr && yl <= y && y <= yr) {
                        continue;
                    }

                    // go backward in diagonal path
                    let (u, v) = {
                        let mut u = x;
                        let mut v = y;
                        let len = self.xv[xl as usize..u as usize]
                            .iter()
                            .rev()
                            .zip(self.yv[yl as usize..v as usize].iter().rev())
                            .take_while(|(x, y)| x == y)
                            .count() as isize;
                        u -= len;
                        v -= len;
                        (u, v)
                    };
                    debug_assert!(xl <= u && u <= xr);
                    debug_assert!(yl <= v && v <= yr);

                    self.setv(k, u, false);
                    if !is_odd
                        && kminf <= k
                        && k <= kmaxf
                        && self.getv(k, true).map(|v| v >= u).unwrap_or(false)
                    {
                        return (2 * d as usize, (x as usize, y as usize));
                    }
                }
            }
        }

        unreachable!();
    }
}

/// Returns the correspondence between two sequences.
///
/// The return value is a pair of tuples. The first tuple contains the index
/// where the item from the first sequence appears in the 2nd sequence or `None`
/// if the item doesn't appear in the 2nd sequence. The 2nd tuple is the same
/// but listing the corresponding indexes for the 2nd sequence in the first
/// sequence.
///
/// # Examples
///
/// ```
/// use seqdiff;
/// let (a2b, b2a) = seqdiff::diff(&[1, 2, 3], &[1, 3]);
/// assert_eq!(a2b, vec![Some(0), None, Some(1)]);
/// assert_eq!(b2a, vec![Some(0), Some(2)]);
/// ```
pub fn diff<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> (Diff, Diff) {
    let mut st = Difference::new(a, b);
    st.diff();
    let Difference { xe, ye, .. } = st;
    (xe, ye)
}

/// Compute similarity of two sequences.
/// The similarity is a floating point number in [0., 100.], computed based on
/// Levenshtein distance.
/// This is useful, for example, fuzzy search.
///
/// # Examples
///
/// ```
/// use seqdiff::ratio;
/// let r = ratio(
///     &"Hello world!".chars().collect::<Vec<_>>(),
///     &"Holly grail!".chars().collect::<Vec<_>>(),
/// );
/// assert!((r - 58.333333333333337).abs() < 1e-5);
/// ```
#[allow(clippy::many_single_char_names)]
pub fn ratio<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> f64 {
    let l = a.len() + b.len();
    if l == 0 {
        return 100.;
    }
    let dist = Difference::new(a, b).diff();
    let ret = l - dist;
    (ret * 100) as f64 / l as f64
}
