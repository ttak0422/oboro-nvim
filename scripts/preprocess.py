"""
preprocessor fmr lua.

example:
---------------------

```lua:target_file.lua
-- BEGIN_NORMAL
execute_foo()
-- END_NORMAL

-- BEGIN_OPTIMIZED
execute_bar()
-- END_OPTIMIZED
```

```sh
$ python3 preprocess.py OPTIMIZED target_file.lua
$ cat target_file.lua


execute_bar()

```
"""

import sys
import re

normal_pattern = re.compile(r"--\sBEGIN_NORMAL.*\s((.|\s)*?)--\sEND_NORMAL")
optimize_pattern = re.compile(
    r"--\sBEGIN_OPTIMIZED.*\s((.|\s)*?)--\sEND_OPTIMIZED")
comment_pattern = re.compile(r"^\s*--.*$", re.MULTILINE)
emptyline_pattern = re.compile(r"^\s*($|\n)", re.MULTILINE)


def apply_normal(target: str) -> None:
    with open(target) as f:
        file_content = f.read()

    # apply normal code.
    file_content = normal_pattern.sub(r"\1", file_content)
    # remove optimized code.
    file_content = optimize_pattern.sub("", file_content)

    with open(target, "w") as f:
        f.write(file_content)


def apply_optimized(target: str) -> None:
    with open(target) as f:
        file_content = f.read()

    # remove normal code.
    file_content = normal_pattern.sub("", file_content)
    # apply optimized code.
    file_content = optimize_pattern.sub(r"\1", file_content)
    # remove comment line.
    file_content = comment_pattern.sub("", file_content)
    # remove space and empty lines.
    file_content = emptyline_pattern.sub("", file_content)
    with open(target, "w") as f:
        f.write(file_content)


def main(args: list[str] = sys.argv) -> int:
    """entry point.

    usage:
    ---------------------
    python3 preprocess.py NORMAL foo.lua bar.lua

    params:
    ---------------------
    1st: flag (NORMAL | OPTIMIZED)
    Nth: target lua files
    """

    flag = args[1]
    target_files = args[2:]

    apply_fn = apply_optimized if flag == "OPTIMIZED" else apply_normal

    for target_file in target_files:
        apply_fn(target_file)

    return 0


if __name__ == "__main__":
    sys.exit(main())
