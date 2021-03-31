use crate::*;
use rstest::rstest;
struct EditPathFromGrid {
    // This struct is only for testing
    // Inefficient but simple algorithm
    d: Vec<Vec<usize>>,
    cur: (usize, usize),
    exhausted: bool,
}

impl Iterator for EditPathFromGrid {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        if self.cur == (0, 0) {
            self.exhausted = true;
            return Some((0, 0));
        }
        let (i, j) = self.cur;
        let ncur = if i > 0 && j > 0 {
            let ncur = *[(i, j - 1), (i - 1, j)]
                .iter()
                .min_by_key(|x| self.d[x.0][x.1])
                .unwrap();
            let ul = self.d[i - 1][j - 1];
            if self.d[ncur.0][ncur.1] > ul && self.d[i][j] == ul {
                (i - 1, j - 1)
            } else {
                ncur
            }
        } else if i > 0 {
            (i - 1, j)
        } else {
            (i, j - 1)
        };
        self.cur = ncur;
        Some((i, j))
    }
}

#[allow(clippy::many_single_char_names)]
fn get_shortest_edit_path_grid<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> EditPathFromGrid {
    let n = a.len();
    let m = b.len();
    let mut d = vec![vec![std::usize::MAX; m + 1]; n + 1];
    d[0] = (0..=m).collect();
    for (i, e) in d.iter_mut().enumerate().take(n + 1) {
        e[0] = i;
    }
    for (i, ca) in a.iter().enumerate() {
        for (j, cb) in b.iter().enumerate() {
            if ca == cb {
                d[i + 1][j + 1] = d[i][j];
            } else {
                let mut u = (i + 1, j);
                let l = (i, j + 1);
                if d[u.0][u.1] > d[l.0][l.1] {
                    u = l;
                }
                d[i + 1][j + 1] = d[u.0][u.1] + 1;
            }
        }
    }

    EditPathFromGrid {
        d,
        cur: (n, m),
        exhausted: false,
    }
}

#[quickcheck]
fn qc_with_old(a: Vec<char>, b: Vec<char>) {
    let v = old::path_to_diff(
        old::get_shortest_edit_path(&a, &b, char::eq, true)
            .1
            .unwrap(),
    );
    let w = diff(&a, &b);
    assert_eq!(v, w);
    let (a2b, b2a) = w;
    assert_eq!(a.len(), a2b.len());
    assert_eq!(b.len(), b2a.len());
}

#[quickcheck]
fn qc_myers_with_dp(a: Vec<char>, b: Vec<char>) {
    let v = old::path_to_diff(get_shortest_edit_path_grid(&a, &b));
    let w = diff(&a, &b);
    assert_eq!(v, w);
    let (a2b, b2a) = w;
    assert_eq!(a.len(), a2b.len());
    assert_eq!(b.len(), b2a.len());
}

#[test]
fn test_diff() {
    let cases = [
        (
            (vec![std::f64::NAN], vec![std::f64::NAN]),
            (vec![None], vec![None]),
        ),
        (
            (vec![1., 2., 3.], vec![1., 3.]),
            (vec![Some(0), None, Some(1)], vec![Some(0), Some(2)]),
        ),
    ];
    for ((a, b), expected) in cases.iter() {
        let ret = diff(a, b);
        assert_eq!(ret, *expected);
    }
}

pub fn slow_ratio<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> f64 {
    let l = a.len() + b.len();
    if l == 0 {
        return 100.;
    }
    let (a2b, _) = diff(a, b);
    let m = a2b.iter().filter(|x| x.is_some()).count() * 2;
    ((100 * m) as f64) / (l as f64)
}

#[quickcheck]
fn distance_consistency(s: Vec<char>, t: Vec<char>) {
    let (dist, path) = old::get_shortest_edit_path(&s, &t, char::eq, true);
    let (a2b, _) = old::path_to_diff(path.unwrap());
    let n = a2b.iter().filter(|x| x.is_some()).count() * 2;
    let m = s.len() + t.len() - dist;
    assert_eq!(n, m);
}

#[quickcheck]
fn qc_diff_num_with_old(s: Vec<char>, t: Vec<char>) {
    let a = old::get_shortest_edit_path(&s, &t, char::eq, false).0;
    let mut diff = Difference::new(&s, &t);
    assert_eq!(a, diff.find_mid((0, s.len()), (0, t.len())).0)
}

#[quickcheck]
fn qc_ratio(s: Vec<char>, t: Vec<char>) {
    ratio(&s, &t);
}

#[quickcheck]
fn qc_ratio_same(s: Vec<char>) {
    assert!((ratio(&s, &s) - 100f64).abs() < 1e-5);
}

#[quickcheck]
fn qc_ratio_with_slow(s: Vec<char>, t: Vec<char>) {
    let slow = slow_ratio(&s, &t);
    let fast = ratio(&s, &t);
    assert!((slow - fast).abs() < 1e-5);
}

#[rstest(
    s,
    t,
    expected,
    case("abc", "abc", 100.),
    case("abc", "abd", 66.66666667),
    case("abc", "abddddd", 40.)
)]
fn test_ratio(s: &str, t: &str, expected: f64) {
    let ret = ratio(
        &s.chars().collect::<Vec<_>>(),
        &t.chars().collect::<Vec<_>>(),
    );
    assert!(
        (ret - expected).abs() < 1e-5,
        "expected: {}\nresult: {}",
        expected,
        ret
    );
}

#[rstest(xv, yv, expected,
        case(vec![0, 3], vec![1, 1, 1, 1, 0, 3], (4, (0, 2), (0, 2))),
        case(vec![0], vec![1, 1, 1], (4, (0, 2), (0, 2))),
        case(vec![0], vec![1, 1], (3, (0, 2), (0, 2))),
        case(vec![0], vec![0, 1, 0], (2, (0, 1), (0, 1))),
        case(vec![0], vec![0, 0, 0], (2, (0, 1), (0, 1))),
        case(vec![0], vec![], (1, (1, 0), (1, 0))),
        case(vec![], vec![0], (1, (0, 1), (0, 1))),
        case(vec![], vec![], (0, (0, 0), (0, 0))),
        case(vec![0, 1, 2], vec![0, 1, 1, 2], (1, (2, 3), (3, 4))),
        case(vec![0, 1, 1, 2], vec![0, 1, 2], (1, (3, 2), (4, 3))),
        case(vec![0, 1, 2, 3], vec![0, 1, 2], (1, (4, 3), (4, 3))),
        case(vec![0, 1, 2], vec![0, 2, 2], (2, (1, 2), (1, 2))),
        case(vec![0, 2, 2], vec![0, 1, 2], (2, (1, 2), (1, 2))),
        case(vec![0, 1, 2], vec![0, 1, 2], (0, (0, 0), (3, 3))),

         )]
fn find_mid(xv: Vec<usize>, yv: Vec<usize>, expected: (usize, (usize, usize), (usize, usize))) {
    let n = xv.len();
    let m = yv.len();
    let mut diff = Difference::new(&xv, &yv);
    let ret = diff.find_mid((0, n), (0, m));
    assert_eq!(ret, expected);
}

// old implementations
mod old {
    use super::*;
    use std::collections::HashMap;

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

    pub fn path_to_diff(mut path: impl Iterator<Item = (usize, usize)>) -> (Diff, Diff) {
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

    pub fn diff<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> (Diff, Diff) {
        diff_by(a, b, <A as PartialEq<B>>::eq)
    }

    pub fn diff_by<A, B, F>(a: &[A], b: &[B], is_eq: F) -> (Diff, Diff)
    where
        F: Fn(&A, &B) -> bool,
    {
        path_to_diff(get_shortest_edit_path(a, b, is_eq, true).1.unwrap())
    }

    pub fn ratio<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> f64 {
        let l = a.len() + b.len();
        if l == 0 {
            return 100.;
        }
        let ret = l - get_shortest_edit_path(a, b, <A as PartialEq<B>>::eq, false).0;
        (ret * 100) as f64 / l as f64
    }
}
