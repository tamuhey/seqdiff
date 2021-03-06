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
    // working memory for backward path
    vb: Vec<isize>,
    offset_d: isize,

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
        let dmax = xv.len() + yv.len() + 1;
        let offset_d = yv.len() as isize;
        let vf = vec![MIN; dmax];
        let vb = vec![MAX; dmax];
        let xe = vec![None; xv.len()];
        let ye = vec![None; yv.len()];
        Self {
            xv,
            yv,
            vf,
            vb,
            offset_d,
            xe,
            ye,
        }
    }

    fn diff(&mut self) -> usize {
        self.diff_part((0, self.xv.len()), (0, self.yv.len()))
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

        // convert k to index of working memory (vf, vb)
        let ktoi = {
            let offset = self.offset_d;
            move |k: isize| -> usize { (k + offset) as usize }
        };

        self.vf[ktoi(kmidf)] = xl;
        self.vb[ktoi(kmidb)] = xr;

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
                    if let Some(x) = self.vf.get_mut(ktoi(kminf - 1)) {
                        *x = MIN;
                    }
                } else {
                    kminf += 1;
                }
                if kmaxf < kmax {
                    kmaxf += 1;
                    if let Some(x) = self.vf.get_mut(ktoi(kmaxf + 1)) {
                        *x = MIN;
                    }
                } else {
                    kmaxf -= 1
                }

                for k in (kminf..=kmaxf).rev().step_by(2) {
                    let ik = ktoi(k);
                    let x = {
                        let lo = self.vf.get(ktoi(k - 1)).cloned();
                        let hi = self.vf.get(ktoi(k + 1)).cloned();
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

                    self.vf[ik] = u;
                    if is_odd && kminb <= k && k <= kmaxb && self.vb[ik] <= u {
                        return (2 * d as usize - 1, (x as usize, y as usize));
                    }
                }
            }

            // backward
            {
                // update range
                if kminb > kmin {
                    kminb -= 1;
                    if let Some(x) = self.vb.get_mut(ktoi(kminb - 1)) {
                        *x = MAX;
                    }
                } else {
                    kminb += 1;
                }
                if kmaxb < kmax {
                    kmaxb += 1;
                    if let Some(x) = self.vb.get_mut(ktoi(kmaxb + 1)) {
                        *x = MAX;
                    }
                } else {
                    kmaxb -= 1
                }

                for k in (kminb..=kmaxb).rev().step_by(2) {
                    let x = {
                        let lo = self.vb.get(ktoi(k - 1)).cloned();
                        let hi = self.vb.get(ktoi(k + 1)).cloned();
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

                    let ik = ktoi(k);
                    self.vb[ik] = u;
                    if !is_odd && kminf <= k && k <= kmaxf && self.vf[ik] >= u {
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
