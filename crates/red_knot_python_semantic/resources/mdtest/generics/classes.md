# Generic classes

## PEP 695 syntax

TODO: Add a `red_knot_extension` function that asserts whether a function or class is generic.

This is a generic class defined using PEP 695 syntax:

```py
class C[T]: ...
```

A class that inherits from a generic class, and fills its type parameters with typevars, is generic:

```py
class D[U](C[U]): ...
```

A class that inherits from a generic class, but fills its type parameters with concrete types, is
_not_ generic:

```py
class E(C[int]): ...
```

A class that inherits from a generic class, and doesn't fill its type parameters at all, implicitly
uses the default value for the typevar. In this case, that default type is `Unknown`, so `F`
inherits from `C[Unknown]` and is not itself generic.

```py
class F(C): ...
```

## Legacy syntax

This is a generic class defined using the legacy syntax:

```py
from typing import Generic, TypeVar

T = TypeVar("T")

# TODO: no error
# error: [invalid-base]
class C(Generic[T]): ...
```

A class that inherits from a generic class, and fills its type parameters with typevars, is generic.

```py
class D(C[T]): ...
```

(Examples `E` and `F` from above do not have analogues in the legacy syntax.)

## Specializing generic classes explicitly

The type parameter can be specified explicitly:

```py
class C[T]:
    x: T

reveal_type(C[int]())  # revealed: C[int]
```

The specialization must match the generic types:

```py
# error: [too-many-positional-arguments] "Too many positional arguments to class `C`: expected 1, got 2"
reveal_type(C[int, int]())  # revealed: Unknown
```

If the type variable has an upper bound, the specialized type must satisfy that bound:

```py
class Bounded[T: int]: ...
class BoundedByUnion[T: int | str]: ...
class IntSubclass(int): ...

reveal_type(Bounded[int]())  # revealed: Bounded[int]
reveal_type(Bounded[IntSubclass]())  # revealed: Bounded[IntSubclass]

# error: [invalid-argument-type] "Object of type `str` cannot be assigned to parameter 1 (`T`) of class `Bounded`; expected type `int`"
reveal_type(Bounded[str]())  # revealed: Unknown

# error:  [invalid-argument-type] "Object of type `int | str` cannot be assigned to parameter 1 (`T`) of class `Bounded`; expected type `int`"
reveal_type(Bounded[int | str]())  # revealed: Unknown

reveal_type(BoundedByUnion[int]())  # revealed: BoundedByUnion[int]
reveal_type(BoundedByUnion[IntSubclass]())  # revealed: BoundedByUnion[IntSubclass]
reveal_type(BoundedByUnion[str]())  # revealed: BoundedByUnion[str]
reveal_type(BoundedByUnion[int | str]())  # revealed: BoundedByUnion[int | str]
```

If the type variable is constrained, the specialized type must satisfy those constraints:

```py
class Constrained[T: (int, str)]: ...

reveal_type(Constrained[int]())  # revealed: Constrained[int]

# TODO: error: [invalid-argument-type]
# TODO: revealed: Constrained[Unknown]
reveal_type(Constrained[IntSubclass]())  # revealed: Constrained[IntSubclass]

reveal_type(Constrained[str]())  # revealed: Constrained[str]

# TODO: error: [invalid-argument-type]
# TODO: revealed: Unknown
reveal_type(Constrained[int | str]())  # revealed: Constrained[int | str]

# error: [invalid-argument-type] "Object of type `object` cannot be assigned to parameter 1 (`T`) of class `Constrained`; expected type `int | str`"
reveal_type(Constrained[object]())  # revealed: Unknown
```

## Inferring generic class parameters

We can infer the type parameter from a type context:

```py
class C[T]:
    x: T

c: C[int] = C()
# TODO: revealed: C[int]
reveal_type(c)  # revealed: C[Unknown]
```

The typevars of a fully specialized generic class should no longer be visible:

```py
# TODO: revealed: int
reveal_type(c.x)  # revealed: Unknown
```

If the type parameter is not specified explicitly, and there are no constraints that let us infer a
specific type, we infer the typevar's default type:

```py
class D[T = int]: ...

reveal_type(D())  # revealed: D[int]
```

If a typevar does not provide a default, we use `Unknown`:

```py
reveal_type(C())  # revealed: C[Unknown]
```

If the type of a constructor parameter is a class typevar, we can use that to infer the type
parameter:

```py
class E[T]:
    def __init__(self, x: T) -> None: ...

# TODO: revealed: E[int] or E[Literal[1]]
reveal_type(E(1))  # revealed: E[Unknown]
```

The types inferred from a type context and from a constructor parameter must be consistent with each
other:

```py
# TODO: error: [invalid-argument-type]
wrong_innards: E[int] = E("five")
```

## Generic subclass

When a generic subclass fills its superclass's type parameter with one of its own, the actual types
propagate through:

```py
class Base[T]:
    x: T | None = None

class Sub[U](Base[U]): ...

reveal_type(Base[int].x)  # revealed: int | None
reveal_type(Sub[int].x)  # revealed: int | None
```

## Generic methods

Generic classes can contain methods that are themselves generic. The generic methods can refer to
the typevars of the enclosing generic class, and introduce new (distinct) typevars that are only in
scope for the method.

```py
class C[T]:
    def method[U](self, u: U) -> U:
        return u
    # error: [unresolved-reference]
    def cannot_use_outside_of_method(self, u: U): ...

    # TODO: error
    def cannot_shadow_class_typevar[T](self, t: T): ...

c: C[int] = C[int]()
# TODO: no error
# TODO: revealed: str or Literal["string"]
# error: [invalid-argument-type]
reveal_type(c.method("string"))  # revealed: U
```

## Cyclic class definition

A class can use itself as the type parameter of one of its superclasses. (This is also known as the
[curiously recurring template pattern][crtp] or [F-bounded quantification][f-bound].)

Here, `Sub` is not a generic class, since it fills its superclass's type parameter (with itself).

`stub.pyi`:

```pyi
class Base[T]: ...
class Sub(Base[Sub]): ...

reveal_type(Sub)  # revealed: Literal[Sub]
```

A similar case can work in a non-stub file, if forward references are stringified:

`string_annotation.py`:

```py
class Base[T]: ...
class Sub(Base["Sub"]): ...

reveal_type(Sub)  # revealed: Literal[Sub]
```

In a non-stub file, without stringified forward references, this raises a `NameError`:

`bare_annotation.py`:

```py
class Base[T]: ...

# error: [unresolved-reference]
class Sub(Base[Sub]): ...
```

## Another cyclic case

```pyi
# TODO no error (generics)
# error: [invalid-base]
class Derived[T](list[Derived[T]]): ...
```

[crtp]: https://en.wikipedia.org/wiki/Curiously_recurring_template_pattern
[f-bound]: https://en.wikipedia.org/wiki/Bounded_quantification#F-bounded_quantification
