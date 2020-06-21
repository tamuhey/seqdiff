from .seqdiff import diff, __version__
from typing import Any, Callable, Optional, Sequence, TypeVar, List


T = TypeVar("T")
S = TypeVar("S")

RED = "\033[31m"
GREEN = "\033[32m"
ENDC = "\033[0m"


def _color(text: Any, color: str) -> str:
    return f"{color}{text}{ENDC}"


def _color_list(a: Sequence[Any], a2b: List[Optional[int]], color: str) -> List[str]:
    return [_color(ai, color) if j == None else str(ai) for ai, j in zip(a, a2b)]


def _format_list(a: List[str]) -> str:
    return "[" + ", ".join(a) + "]"


def print_diff(
    a: Sequence[S], b: Sequence[T], *, key: Optional[Callable[[S, T], bool]] = None
):
    a2b, b2a = diff(a, b, key=key)
    texta = _color_list(a, a2b, RED)
    textb = _color_list(b, b2a, GREEN)
    print(_format_list(texta))
    print(_format_list(textb))


__all__ = ["diff", "__version__", "print_diff"]
