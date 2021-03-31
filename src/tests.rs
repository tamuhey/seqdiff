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
fn get_shortest_edit_path_slow<A: PartialEq<B>, B>(a: &[A], b: &[B]) -> EditPathFromGrid {
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

fn dist_from_diffs(a: &Diff, b: &Diff) -> usize {
    let a = a.iter().filter(|x| x.is_none()).count();
    let b = b.iter().filter(|x| x.is_none()).count();
    a + b
}

fn compare_with_slow<T>(a: &[T], b: &[T])
where
    T: PartialEq,
{
    let dist = Difference::new(&a, &b).diff();
    let (da, db) = path_to_diff(get_shortest_edit_path_slow(&a, &b));
    let dist_slow = dist_from_diffs(&da, &db);
    assert_eq!(dist, dist_slow);
}

#[rstest(a,b,expected,
    case(vec![0,1], vec![1,0], 2),
    case(vec![0], vec![1,0,0], 2),
)]
fn hm_dist(a: Vec<u8>, b: Vec<u8>, expected: usize) {
    let y = Difference::new(&a, &b).diff();
    assert_eq!(y, expected);
}

#[rstest(a,b,
    case(vec![0,1], vec![1,0]),
    case(vec![0], vec![1,0,0]),
)]
fn hm_diff_with_slow(a: Vec<u8>, b: Vec<u8>) {
    compare_with_slow(&a, &b);
}

#[quickcheck]
fn qc_diff_with_slow(a: Vec<u8>, b: Vec<u8>) {
    compare_with_slow(&a, &b);
}

#[quickcheck]
fn qc_distance_consistency(s: Vec<char>, t: Vec<char>) {
    let mut st = Difference::new(&s, &t);
    let dist = st.diff();
    let y = dist_from_diffs(&st.xe, &st.ye);
    assert_eq!(dist, y);
}

#[quickcheck]
fn qc_check_equality(s: Vec<usize>, t: Vec<usize>) {
    let (a, b) = diff(&s, &t);
    assert_eq!(s.len(), a.len());
    let check = |s: &[_], t: &[_], a: &[_]| {
        for (&si, &j) in s.iter().zip(a.iter()) {
            if let Some(j) = j {
                assert_eq!(si, t[j]);
            }
        }
    };
    check(&s, &t, &a);
    check(&t, &s, &b);
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
fn qc_ratio_fuzz(s: Vec<char>, t: Vec<char>) {
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
fn hm_ratio(s: &str, t: &str, expected: f64) {
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
        case(vec![0, 3], vec![1, 1, 1, 1, 0, 3], (4, (0, 2) )),
        case(vec![0], vec![1, 1, 1], (4, (1, 1))),
        case(vec![0], vec![1, 1], (3, (1, 1))),
        case(vec![0], vec![0, 1, 0], (2, (0, 1))),
        case(vec![0], vec![0, 0, 0], (2, (0, 1))),
        case(vec![0], vec![], (1, (1, 0))),
        case(vec![], vec![0], (1, (0, 1))),
        case(vec![], vec![], (0, (0, 0))),
        case(vec![0, 1, 2], vec![0, 1, 1, 2], (1, (2, 3))),
        case(vec![0, 1, 1, 2], vec![0, 1, 2], (1, (3, 2))),
        case(vec![0, 1, 2, 3], vec![0, 1, 2], (1, (4, 3))),
        case(vec![0, 1, 2], vec![0, 2, 2], (2, (2, 1))),
        case(vec![0, 2, 2], vec![0, 1, 2], (2, (2, 1))),
        case(vec![0, 1, 2], vec![0, 1, 2], (0, (3, 3))),
)]
fn hm_find_mid(xv: Vec<usize>, yv: Vec<usize>, expected: (usize, (usize, usize))) {
    let n = xv.len();
    let m = yv.len();
    let mut diff = Difference::new(&xv, &yv);
    let ret = diff.find_mid((0, n), (0, m));

    assert_eq!(ret, expected);
}
