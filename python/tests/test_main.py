import pytest
import math
from seqdiff import diff, __version__
from hypothesis import strategies as st, given, example


def test_version():
    assert isinstance(__version__, str)


nan = float("nan")


def cmp_nan_eq(x, y) -> bool:
    if math.isnan(x) and math.isnan(y):
        return True
    return x == y


@pytest.mark.parametrize(
    "a,b,key,expected",
    [
        ([float("nan")], [float("nan")], None, ([None], [None])),
        ([nan], [nan], None, ([None], [None])),
        ([nan], [nan], cmp_nan_eq, ([0], [0])),
    ],
)
def test_diff(a, b, key, expected):
    assert diff(a, b, key=key) == expected


@given(
    st.one_of(
        st.tuples(t, t)
        for t in [
            st.lists(s) for s in [st.integers(), st.text(), st.booleans(), st.floats()]
        ]
    )
)
@example(([0.0], [0.0]))
@example(ab=([nan], [nan]))
def test_random(ab):
    a, b = ab
    (a2b, b2a) = diff(a, b)
    for i, j in enumerate(a2b):
        assert j is None or a[i] == b[j], (a, b, a2b, b2a)
    for i, j in enumerate(b2a):
        assert j is None or a[j] == b[i]
    commons = list(set(a) & set(b))
    c = commons[0] if len(commons) > 0 else None
    if commons and (c == c):
        assert any(x is not None for x in a2b) and any(x is not None for x in b2a)


@given(st.lists(st.text()))
def test_equality(a):
    a2b, b2a = diff(a, a)
    assert a2b == b2a

