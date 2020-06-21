from typing import Callable, List, Optional, Sequence, Any, Tuple, TypeVar

T = TypeVar("T")
S = TypeVar("S")

def diff(
    a: Sequence[S], b: Sequence[T], *, key: Optional[Callable[[S, T], bool]] = None
) -> Tuple[List[Optional[int]], List[Optional[int]]]: ...

__version__: str

