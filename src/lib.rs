//! Functions to get correspondence between two sequences like diff,
//! based on Myers' algorithm.
#[cfg(test)]
mod tests;
use std::cmp::{max, min};
use std::collections::HashMap;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

struct Difference<'a, X, Y> {
    xv: &'a [X],
    yv: &'a [Y],

    // working memory for forward path
    vf: Vec<usize>,
    // working memory for backward path
    vb: Vec<usize>,
    offset_d: i64,
}

impl<'a, X, Y> Difference<'a, X, Y>
where
    X: PartialEq<Y>,
{
    fn new(xv: &'a [X], yv: &'a [Y]) -> Self {
        let dmax = xv.len() + yv.len() + 1;
        let offset_d = yv.len() as i64;
        let vf = vec![0usize; dmax];
        let vb = vec![!0usize; dmax];
        Self {
            xv,
            yv,
            vf,
            vb,
            offset_d,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn find_mid(
        &mut self,
        (xl, xr): (usize, usize),
        (yl, yr): (usize, usize),
    ) -> (usize, (usize, usize), (usize, usize)) {
        let kmin = xl as i64 - yr as i64;
        let kmax = xr as i64 - yl as i64;
        let kmidf = xl as i64 - yl as i64; // center diag in this fragment for forwad snake
        let kmidb = xr as i64 - yr as i64;
        let delta = ((xr - xl) as i64) - ((yr - yl) as i64);
        let is_odd = (delta & 1) == 1;
        let ktoi = {
            let offset = self.offset_d;
            move |k: i64| -> usize { (k + offset) as usize }
        };

        self.vf[ktoi(kmidf)] = xl;
        self.vb[ktoi(kmidb)] = xr;

        let mut kminf = kmidf;
        let mut kmaxf = kmidf;
        let mut kminb = kmidb;
        let mut kmaxb = kmidb;

        let gety = |x: usize, k: i64| (x as i64 - k) as usize;
        for d in 0i64.. {
            // forward
            {
                for k in (kminf..=kmaxf).step_by(2) {
                    let ikhi = ktoi(k + 1);
                    let iklo = ktoi(k - 1);
                    let ik = ktoi(k);
                    let x = if d == 0 {
                        xl
                    } else if k == kminf || k != kmaxf && self.vf[iklo] < self.vf[ikhi] {
                        self.vf[ikhi]
                    } else {
                        self.vf[iklo] + 1
                    };
                    let y = gety(x, k);
                    let mut u = x;
                    let mut v = y;
                    while u < xr && v < yr && self.xv[u] == self.yv[v] {
                        u += 1;
                        v += 1;
                    }
                    self.vf[ik] = u;
                    if is_odd && kminb <= k && k <= kmaxb && self.vb[ik] <= u {
                        return (2 * d as usize - 1, (x, y), (u, v));
                    }
                }
            }
            println!("vf {:?}", self.vf);

            // backward
            {
                for k in (kminb..=kmaxb).step_by(2) {
                    let ikhi = ktoi(k + 1);
                    let iklo = ktoi(k - 1);
                    let ik = ktoi(k);
                    let x = if d == 0 {
                        xr
                    } else if k == kminb || k != kmaxb && self.vb[iklo] > self.vb[ikhi] {
                        self.vb[ikhi] - 1
                    } else {
                        self.vb[iklo]
                    };
                    let y = gety(x, k);
                    let mut u = x;
                    let mut v = y;
                    while xl < u && yl < v && self.xv[u - 1] == self.yv[v - 1] {
                        u -= 1;
                        v -= 1;
                    }
                    self.vb[ik] = u;
                    if !is_odd && kminf <= k && k <= kmaxf && self.vf[ktoi(k)] >= u {
                        return (2 * d as usize, (u, v), (x, y));
                    }
                }
            }
            println!("vb {:?}", self.vb);

            // update range
            if kminf > kmin {
                kminf -= 1;
                if let Some(x) = self.vf.get_mut(ktoi(kminf - 1)) {
                    *x = 0
                }
            } else {
                kminf += 1;
            }
            if kmaxf < kmax {
                kmaxf += 1;
                if let Some(x) = self.vf.get_mut(ktoi(kmaxf + 1)) {
                    *x = 0;
                }
            } else {
                kmaxf -= 1
            }

            if kminb > kmin {
                kminb -= 1;
                if let Some(x) = self.vb.get_mut(ktoi(kminb - 1)) {
                    *x = !0usize;
                }
            } else {
                kminb += 1;
            }
            if kmaxb < kmax {
                kmaxb += 1;
                if let Some(x) = self.vb.get_mut(ktoi(kmaxb + 1)) {
                    *x = !0usize;
                }
            } else {
                kmaxb -= 1
            }
        }

        unreachable!();
    }
}

#[test]
fn find_mid() {
    use std::array::IntoIter;
    let testcases = IntoIter::new([
        // (vec![0], vec![1, 1, 1], (4, (0, 2), (0, 2))),
        // (vec![0], vec![1, 1], (3, (0, 2), (0, 2))),
        // (vec![0], vec![0, 1, 0], (2, (1, 2), (1, 2))),
        // (vec![0], vec![0, 0, 0], (2, (0, 1), (1, 2))),
        // (vec![0], vec![], (1, (0, 0), (0, 0))),
        (vec![], vec![0], (1, (0, 0), (0, 0))),
        // (vec![], vec![], (0, (0, 0), (0, 0))),
        (vec![0, 1, 2], vec![0, 1, 1, 2], (1, (2, 3), (3, 4))),
        // (vec![0, 1, 1, 2], vec![0, 1, 2], (1, (3, 2), (4, 3))),
        // (vec![0, 1, 2, 3], vec![0, 1, 2], (1, (4, 3), (4, 3))),
        // (vec![0, 1, 2], vec![0, 2, 2], (2, (1, 2), (1, 2))),
        // (vec![0, 2, 2], vec![0, 1, 2], (2, (1, 2), (1, 2))),
        // (vec![0, 1, 2], vec![0, 1, 2], (0, (0, 0), (3, 3))),
    ]);
    for (xv, yv, expected) in testcases {
        let n = xv.len();
        let m = yv.len();
        let mut diff = Difference::new(&xv, &yv);
        let ret = diff.find_mid((0, n), (0, m));
        assert_eq!(ret, expected, "\nxv: {:?}\nyv: {:?}", xv, yv);
    }
}

#[cfg(test)]
use self::old::get_shortest_edit_path;

fn path_to_diff(mut path: impl Iterator<Item = (usize, usize)>) -> (Diff, Diff) {
    let (mut i, mut j) = path.next().unwrap();
    let mut a2b = vec![None; i];
    let mut b2a = vec![None; j];
    for (pi, pj) in path {
        if (i - pi) + (j - pj) == 2 {
            a2b[pi] = Some(pj);
            b2a[pj] = Some(pi);
        }
        i = pi;
        j = pj;
    }
    (a2b, b2a)
}

/// An alias for the result of diff type
pub type Diff = Vec<Option<usize>>;

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
    diff_by(a, b, <A as PartialEq<B>>::eq)
}

/// Returns the correspondence between two sequences with a comparison function.
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
/// let nan_eq = |a: &f64, b: &f64| {
///     if a.is_nan() && b.is_nan() {
///         true
///     } else {
///         a == b
///     }
/// };
/// let (a2b, b2a) = seqdiff::diff_by(&[1., 2., f64::NAN], &[1., f64::NAN], nan_eq);
/// assert_eq!(a2b, vec![Some(0), None, Some(1)]);
/// assert_eq!(b2a, vec![Some(0), Some(2)]);
/// ```
pub fn diff_by<A, B, F>(a: &[A], b: &[B], is_eq: F) -> (Diff, Diff)
where
    F: Fn(&A, &B) -> bool,
{
    path_to_diff(get_shortest_edit_path(a, b, is_eq, true).1.unwrap())
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
    let ret = l - get_shortest_edit_path(a, b, <A as PartialEq<B>>::eq, false).0;
    (ret * 100) as f64 / l as f64
}

#[cfg(test)]
mod old {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
    enum Node {
        P((usize, usize)),
        Root,
    }

    #[allow(clippy::many_single_char_names)]
    pub fn get_shortest_edit_path<A, B, F>(
        a: &[A],
        b: &[B],
        is_eq: F,
        get_path: bool,
    ) -> (usize, Option<impl Iterator<Item = (usize, usize)>>)
    where
        F: Fn(&A, &B) -> bool,
    {
        let n = a.len();
        let m = b.len();
        let bound = n + m;
        let get_y = |x, k| x + bound - k;
        let mut v = vec![0; 2 * bound + 1];
        let mut nodes_map = if get_path { Some(HashMap::new()) } else { None };
        let mut distance = !0;
        'outer: for d in 0..=bound {
            for k in ((bound - d)..=bound + d).step_by(2) {
                let (mut x, parent) = if d == 0 {
                    (0, Node::Root)
                } else if k == (bound - d) || k != (bound + d) && v[k - 1] < v[k + 1] {
                    let px = v[k + 1];
                    (px, Node::P((px, get_y(px, k + 1))))
                } else {
                    let px = v[k - 1];
                    (px + 1, Node::P((px, get_y(px, k - 1))))
                };
                let mut y = get_y(x, k);
                if get_path {
                    nodes_map.as_mut().unwrap().insert(Node::P((x, y)), parent);
                }
                while x < n && y < m && is_eq(&a[x], &b[y]) {
                    x += 1;
                    y += 1;
                }
                v[k] = x;
                if x >= n && y >= m {
                    distance = d;
                    break 'outer;
                }
            }
        }
        debug_assert_ne!(distance, !0);
        if get_path {
            let mut cur = Node::P((n, m));
            let nodes_map = nodes_map.unwrap();
            let path = std::iter::from_fn(move || match cur {
                Node::Root => None,
                Node::P(ncur) => {
                    cur = if let Some(cur) = nodes_map.get(&Node::P(ncur)) {
                        *cur
                    } else {
                        Node::P((ncur.0 - 1, ncur.1 - 1))
                    };
                    Some(ncur)
                }
            });
            (distance, Some(path))
        } else {
            (distance, None)
        }
    }
}
