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
fn randomcheck_with_old(a: Vec<char>, b: Vec<char>) {
    let v = path_to_diff(
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
fn randomcheck_myers_with_dp(a: Vec<char>, b: Vec<char>) {
    let v = path_to_diff(get_shortest_edit_path_grid(&a, &b));
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
    let (dist, path) = get_shortest_edit_path(&s, &t, char::eq, true);
    let (a2b, _) = path_to_diff(path.unwrap());
    let n = a2b.iter().filter(|x| x.is_some()).count() * 2;
    let m = s.len() + t.len() - dist;
    assert_eq!(n, m);
}

#[quickcheck]
fn quick_diff_num_with_old(s: Vec<char>, t: Vec<char>) {
    let a = get_shortest_edit_path(&s, &t, char::eq, false).0;
    let mut diff = Difference::new(&s, &t);
    assert_eq!(a, diff.find_mid((0, s.len()), (0, t.len())).0)
}

#[quickcheck]
fn quick_ratio(s: Vec<char>, t: Vec<char>) {
    ratio(&s, &t);
}

#[quickcheck]
fn quick_ratio_same(s: Vec<char>) {
    assert!((ratio(&s, &s) - 100f64).abs() < 1e-5);
}

#[quickcheck]
fn quick_ratio_with_slow(s: Vec<char>, t: Vec<char>) {
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
