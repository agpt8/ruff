"""Common operations on Posix pathnames.

Instead of importing this module directly, import os and refer to
this module as os.path.  The "os.path" name is an alias for this
module on Posix systems; on other systems (e.g. Windows),
os.path provides the same operations in a manner specific to that
platform, and is an alias to another module (e.g. ntpath).

Some of this can actually be useful on non-Posix systems too, e.g.
for manipulation of the pathname component of URLs.
"""

import sys
from _typeshed import AnyOrLiteralStr, BytesPath, FileDescriptorOrPath, StrOrBytesPath, StrPath
from collections.abc import Iterable
from genericpath import (
    ALLOW_MISSING as ALLOW_MISSING,
    _AllowMissingType,
    commonprefix as commonprefix,
    exists as exists,
    getatime as getatime,
    getctime as getctime,
    getmtime as getmtime,
    getsize as getsize,
    isdir as isdir,
    isfile as isfile,
    samefile as samefile,
    sameopenfile as sameopenfile,
    samestat as samestat,
)

if sys.version_info >= (3, 13):
    from genericpath import isdevdrive as isdevdrive
from os import PathLike
from typing import AnyStr, overload
from typing_extensions import LiteralString

__all__ = [
    "normcase",
    "isabs",
    "join",
    "splitdrive",
    "split",
    "splitext",
    "basename",
    "dirname",
    "commonprefix",
    "getsize",
    "getmtime",
    "getatime",
    "getctime",
    "islink",
    "exists",
    "lexists",
    "isdir",
    "isfile",
    "ismount",
    "expanduser",
    "expandvars",
    "normpath",
    "abspath",
    "samefile",
    "sameopenfile",
    "samestat",
    "curdir",
    "pardir",
    "sep",
    "pathsep",
    "defpath",
    "altsep",
    "extsep",
    "devnull",
    "realpath",
    "supports_unicode_filenames",
    "relpath",
    "commonpath",
]
__all__ += ["ALLOW_MISSING"]
if sys.version_info >= (3, 12):
    __all__ += ["isjunction", "splitroot"]
if sys.version_info >= (3, 13):
    __all__ += ["isdevdrive"]

supports_unicode_filenames: bool
# aliases (also in os)
curdir: LiteralString
pardir: LiteralString
sep: LiteralString
altsep: LiteralString | None
extsep: LiteralString
pathsep: LiteralString
defpath: LiteralString
devnull: LiteralString

# Overloads are necessary to work around python/mypy#17952 & python/mypy#11880
@overload
def abspath(path: PathLike[AnyStr]) -> AnyStr:
    """Return an absolute path."""

@overload
def abspath(path: AnyStr) -> AnyStr: ...
@overload
def basename(p: PathLike[AnyStr]) -> AnyStr:
    """Returns the final component of a pathname"""

@overload
def basename(p: AnyOrLiteralStr) -> AnyOrLiteralStr: ...
@overload
def dirname(p: PathLike[AnyStr]) -> AnyStr:
    """Returns the directory component of a pathname"""

@overload
def dirname(p: AnyOrLiteralStr) -> AnyOrLiteralStr: ...
@overload
def expanduser(path: PathLike[AnyStr]) -> AnyStr:
    """Expand ~ and ~user constructions.  If user or $HOME is unknown,
    do nothing.
    """

@overload
def expanduser(path: AnyStr) -> AnyStr: ...
@overload
def expandvars(path: PathLike[AnyStr]) -> AnyStr:
    """Expand shell variables of form $var and ${var}.  Unknown variables
    are left unchanged.
    """

@overload
def expandvars(path: AnyStr) -> AnyStr: ...
@overload
def normcase(s: PathLike[AnyStr]) -> AnyStr:
    """Normalize case of pathname.  Has no effect under Posix"""

@overload
def normcase(s: AnyOrLiteralStr) -> AnyOrLiteralStr: ...
@overload
def normpath(path: PathLike[AnyStr]) -> AnyStr:
    """Normalize path, eliminating double slashes, etc."""

@overload
def normpath(path: AnyOrLiteralStr) -> AnyOrLiteralStr: ...
@overload
def commonpath(paths: Iterable[LiteralString]) -> LiteralString:
    """Given a sequence of path names, returns the longest common sub-path."""

@overload
def commonpath(paths: Iterable[StrPath]) -> str: ...
@overload
def commonpath(paths: Iterable[BytesPath]) -> bytes: ...

# First parameter is not actually pos-only,
# but must be defined as pos-only in the stub or cross-platform code doesn't type-check,
# as the parameter name is different in ntpath.join()
@overload
def join(a: LiteralString, /, *paths: LiteralString) -> LiteralString:
    """Join two or more pathname components, inserting '/' as needed.
    If any component is an absolute path, all previous path components
    will be discarded.  An empty last part will result in a path that
    ends with a separator.
    """

@overload
def join(a: StrPath, /, *paths: StrPath) -> str: ...
@overload
def join(a: BytesPath, /, *paths: BytesPath) -> bytes: ...
@overload
def realpath(filename: PathLike[AnyStr], *, strict: bool | _AllowMissingType = False) -> AnyStr:
    """Return the canonical path of the specified filename, eliminating any
    symbolic links encountered in the path.
    """

@overload
def realpath(filename: AnyStr, *, strict: bool | _AllowMissingType = False) -> AnyStr: ...
@overload
def relpath(path: LiteralString, start: LiteralString | None = None) -> LiteralString:
    """Return a relative version of a path"""

@overload
def relpath(path: BytesPath, start: BytesPath | None = None) -> bytes: ...
@overload
def relpath(path: StrPath, start: StrPath | None = None) -> str: ...
@overload
def split(p: PathLike[AnyStr]) -> tuple[AnyStr, AnyStr]:
    """Split a pathname.  Returns tuple "(head, tail)" where "tail" is
    everything after the final slash.  Either part may be empty.
    """

@overload
def split(p: AnyOrLiteralStr) -> tuple[AnyOrLiteralStr, AnyOrLiteralStr]: ...
@overload
def splitdrive(p: PathLike[AnyStr]) -> tuple[AnyStr, AnyStr]:
    """Split a pathname into drive and path. On Posix, drive is always
    empty.
    """

@overload
def splitdrive(p: AnyOrLiteralStr) -> tuple[AnyOrLiteralStr, AnyOrLiteralStr]: ...
@overload
def splitext(p: PathLike[AnyStr]) -> tuple[AnyStr, AnyStr]:
    """Split the extension from a pathname.

    Extension is everything from the last dot to the end, ignoring
    leading dots.  Returns "(root, ext)"; ext may be empty.
    """

@overload
def splitext(p: AnyOrLiteralStr) -> tuple[AnyOrLiteralStr, AnyOrLiteralStr]: ...
def isabs(s: StrOrBytesPath) -> bool:
    """Test whether a path is absolute"""

def islink(path: FileDescriptorOrPath) -> bool:
    """Test whether a path is a symbolic link"""

def ismount(path: FileDescriptorOrPath) -> bool:
    """Test whether a path is a mount point"""

def lexists(path: FileDescriptorOrPath) -> bool:
    """Test whether a path exists.  Returns True for broken symbolic links"""

if sys.version_info >= (3, 12):
    def isjunction(path: StrOrBytesPath) -> bool:
        """Test whether a path is a junction
        Junctions are not supported on the current platform
        """

    @overload
    def splitroot(p: AnyOrLiteralStr) -> tuple[AnyOrLiteralStr, AnyOrLiteralStr, AnyOrLiteralStr]:
        """Split a pathname into drive, root and tail.

        The tail contains anything after the root.
        """

    @overload
    def splitroot(p: PathLike[AnyStr]) -> tuple[AnyStr, AnyStr, AnyStr]: ...
