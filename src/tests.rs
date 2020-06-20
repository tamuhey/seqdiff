use crate::*;
#[cfg(test)]
struct EditPathFromGrid {
    // This struct is only for testing
    // Inefficient but simple algorithm
    d: Vec<Vec<usize>>,
    cur: (usize, usize),
    exhausted: bool,
}

#[cfg(test)]
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
#[cfg(test)]
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
