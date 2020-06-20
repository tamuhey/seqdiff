//! Diff between two sequences
#[cfg(test)]
mod tests;
use std::collections::HashMap;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Node {
    P((usize, usize)),
    Root,
}

struct EditPath {
    nodes_map: HashMap<Node, Node>,
    cur: Node,
}

impl<'a> Iterator for EditPath {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            Node::Root => None,
            Node::P(cur) => {
                self.cur = *self.nodes_map.get(&Node::P(cur)).unwrap();
                Some(cur)
            }
        }
    }
}

/// Returns an iterator over the shotest path of the edit graph based on Myers' diff algorithm.
///
/// See [An O(ND) Difference Algorithm and Its Variations](http://www.xmailserver.org/diff2.pdf)
#[allow(clippy::many_single_char_names)]
fn get_shortest_edit_path_myers<A, B, F>(a: &Vec<A>, b: &Vec<B>, cmp: F) -> EditPath
where
    F: Fn(&A, &B) -> bool,
{
    let n = a.len();
    let m = b.len();
    let bound = n + m;
    let get_y = |x, k| x + bound - k;
    let mut v = vec![0; 2 * bound + 1];
    let mut nodes_map = HashMap::new();
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
            nodes_map.insert(Node::P((x, y)), parent);
            while x < n && y < m && cmp(&a[x], &b[y]) {
                nodes_map.insert(Node::P((x + 1, y + 1)), Node::P((x, y)));
                x += 1;
                y += 1;
            }
            v[k] = x;
            if x >= n && y >= m {
                break 'outer;
            }
        }
    }

    EditPath {
        nodes_map,
        cur: Node::P((n, m)),
    }
}

pub type Diff = Vec<Option<usize>>;

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

pub fn diff_by<A, B, F>(a: &Vec<A>, b: &Vec<B>, cmp: F) -> (Diff, Diff)
where
    F: Fn(&A, &B) -> bool,
{
    path_to_diff(get_shortest_edit_path_myers(a, b, cmp))
}

pub fn diff<A: PartialEq<B>, B>(a: &Vec<A>, b: &Vec<B>) -> (Diff, Diff) {
    diff_by(a, b, <A as PartialEq<B>>::eq)
}
