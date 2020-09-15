//! Functions to get correspondence between two sequences like diff,
//! based on Myers' algorithm.
#![deny(warnings)]
#[cfg(test)]
mod tests;
use std::collections::BTreeMap;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
enum Node {
    P((usize, usize)),
    Root,
}

/// Returns an iterator over the shotest path of the edit graph based on Myers'
/// diff algorithm.
///
/// See [An O(ND) Difference Algorithm and Its Variations](http://www.xmailserver.org/diff2.pdf)
#[allow(clippy::many_single_char_names)]
fn get_shortest_edit_path<A, B, F>(
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
    let mut nodes_map = BTreeMap::new();
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
                nodes_map.insert(Node::P((x, y)), parent);
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
    if get_path {
        let mut cur = Node::P((n, m));
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
