# PEP 8 — Style Guide for Python Code (Adapted Specification)

This document provides a complete specification of the PEP 8 coding standards, configured for strict column limits (79 characters for code, 72 characters for docstrings and comments).

---

## Code Layout

### Indentation

Use 4 spaces per indentation level.

Continuation lines should align wrapped elements either vertically using Python's implicit line joining inside parentheses, brackets, and braces, or using a hanging indent.

* Preferred (Continuation line aligned with opening delimiter):
```python
foo = long_function_name(
    var_one, var_two,
    var_three, var_four
)
```

* Preferred (Hanging indent; further indentation required to distinguish from code block):
```python
def long_function_name(
        var_one, var_two, var_three,
        var_four):
    print(var_one)
```

* Optional (Closing bracket on its own line):
```python
my_list = [
    1, 2, 3,
    4, 5, 6,
]
```

### Tabs or Spaces?

Spaces are the preferred indentation method. Tabs should be used solely to remain consistent with code that is already indented with tabs. Python disallows mixing tabs and spaces for indentation.

### Maximum Line Length

* **Code Lines:** Limit all code lines to a maximum of **79 characters**.
* **Comments & Docstrings:** Limit all block comments, inline comments, and docstrings to a maximum of **72 characters**.

#### Rationale for Line Lengths

Limiting line lengths ensures developers can view multiple files side-by-side without horizontal scrolling and fits within standard terminal dimensions. Long lines in docstrings or comments should be wrapped at 72 characters to accommodate editors that display documentation with left margins.

#### Preferred Line Wrapping

The preferred way of wrapping long lines is by taking advantage of Python's implicit line joining inside parentheses, brackets, and braces. Long lines can also be broken across multiple lines by wrapping expressions in parentheses.

```python
# Preferred line continuation inside parentheses:
with open('/path/to/some/file/with/a/very/large/name/on/disk') as file_1, \
     open('/path/to/another/file/with/a/very/large/name') as file_2:
    file_2.write(file_1.read())
```

### Line Breaks Around Binary Operators

When breaking lines containing binary operators, break **before** the operator. This matching mathematical convention improves readability by keeping operators aligned in the same visual column.

* Preferred (Break before operator):
```python
income = (gross_wages
          + tax_refund
          - interest_expenses)
```

### Blank Lines

* Surround top-level function and class definitions with two blank lines.
* Method definitions inside a class are surrounded by a single blank line.
* Extra blank lines may be used sparingly to separate groups of related functions.
* Use blank lines in functions, sparingly, to indicate logical sections.

### Source File Encoding

Code in the core Python distribution should always use UTF-8. Files using UTF-8 should not have an encoding declaration.

---

## Imports

Imports should usually be placed on separate lines:

* Preferred:
```python
import os
import sys
```

* Acceptable (Importing multiple names from a module):
```python
from subprocess import Popen, PIPE
```

### Import Grouping

Imports should always be put at the top of the file, just after any module comments and docstrings, and before module globals and constants. Group imports in the following order, separated by a blank line between groups:

* Standard library imports.
* Related third-party imports.
* Local application/library specific imports.

### Absolute vs Relative Imports

Absolute imports are recommended, as they are usually more readable and tend to be better behaved:

```python
import mypkg.sibling
from mypkg import sibling
from mypkg.sibling import example
```

Explicit relative imports are an acceptable alternative when dealing with complex package layouts where absolute imports would be unnecessarily verbose.

---

## String Quotes

In Python, single-quoted strings and double-quoted strings are the same. Pick a convention and stick to it. When a string contains single or double quote characters, use the other to avoid escaping backslashes in the string.

---

## Whitespace in Expressions and Statements

### Pet Peeves

Avoid extraneous whitespace in the following situations:

* Immediately inside parentheses, brackets, or braces:
  * Incorrect: `spam( ham[ 1 ], { eggs: 2 } )`
  * Correct: `spam(ham[1], {eggs: 2})`

* Immediately before a comma, semicolon, or colon:
  * Incorrect: `if x == 4 : print(x) ; x , y = y , x`
  * Correct: `if x == 4: print(x); x, y = y, x`

* Immediately before the open parenthes/bracket that starts a function call, indexing, or slicing:
  * Incorrect: `spam (1)` or `dct ['key']`
  * Correct: `spam(1)` or `dct['key']`

* More than one space around an assignment (or other) operator to align it with another:
  * Incorrect:
    ```python
    x        = 1
    long_var = 2
    ```
  * Correct:
    ```python
    x = 1
    long_var = 2
    ```

### Other Recommendations

* Always surround these binary operators with a single space on either side: assignment (`=`), augmented assignment (`+=`, `-=`, etc.), comparisons (`==`, `<`, `>`, `!=`, `<=`, `>=`), and Booleans (`in`, `not in`, `is`, `is not`, `and`, `or`, `not`).
* Don't use spaces around the `=` sign when used to indicate a keyword argument, or when used to indicate a default value for an unannotated function parameter:
  * Correct: `def complex(real, imag=0.0): return magic(r=real, i=imag)`

---

## Comments

Comments that contradict the code are worse than no comments. Always make a priority of keeping comments up to date when code changes!

* Comments should be complete sentences.
* Block comments should generally consist of one or more paragraphs built out of complete sentences.
* Keep comments wrapped at or under **72 characters**.

### Block Comments

Block comments generally apply to some (or all) code that follows them, and are indented to the same level as that code. Each line of a block comment starts with a `#` and a single space (unless it is indented text inside the comment).

### Inline Comments

Use inline comments sparingly. An inline comment is a comment on the same line as a statement. Inline comments should be separated by at least two spaces from the statement. They should start with a `#` and a single space.

```python
x = x + 1  # Compensate for border padding
```

### Documentation Strings (Docstrings)

* Write docstrings for all public modules, functions, classes, and methods.
* Docstrings are not necessary for non-public methods, but you should have a comment that describes what the method does. This comment should appear after the `def` line.
* Multiline docstrings consist of a summary line just like a one-line docstring, followed by a blank line, followed by a more elaborate description.
* Wrap all lines in docstrings at **72 characters**.

```python
"""Single line summary of object purpose.

Detailed paragraph describing parameters, behavior, side effects, and
return values. Always formatted to stay under the 72-character limit.
"""
```

---

## Naming Conventions

### Overriding Principle

Names that are visible to the user as public parts of the API should follow conventions that reflect usage rather than implementation.

### Descriptive Naming Styles

* `b` (single lowercase letter)
* `B` (single uppercase letter)
* `lowercase`
* `lower_case_with_underscores` (Snake Case — standard for functions, variables, modules)
* `UPPERCASE`
* `UPPER_CASE_WITH_UNDERSCORES` (Constants)
* `CapitalizedWords` (PascalCase / CamelCase — standard for classes and types)
* `_single_leading_underscore`: weak "internal use" indicator (e.g., `from M import *` does not import objects whose name starts with an underscore).
* `single_trailing_underscore_`: used by convention to avoid conflicts with Python keywords (e.g., `class_`).
* `__double_leading_underscore`: when naming a class attribute, invokes name mangling.

### Specific Naming Recommendations

* **Modules:** Modules should have short, all-lowercase names. Underscores can be used in the module name if it improves readability.
* **Classes:** Class names should normally use the `CapWords` convention.
* **Type Variables:** Names of type variables should normally be capwords preferring short names: `T`, `AnyStr`, `Num`.
* **Functions & Variables:** Function names should be lowercase, with words separated by underscores as necessary to improve readability.
* **Constants:** Constants are usually defined on a module level and written in all capital letters with underscores separating words (e.g., `MAX_OVERFLOW`).
* **Self & cls:** Always use `self` for the first argument to instance methods, and `cls` for the first argument to class methods.

---

## Programming Recommendations

* Code should be written in a way that does not disadvantage other implementations of Python (PyPy, Jython, IronPython, etc.).
* Comparisons to singletons like `None` should always be done with `is` or `is not`, never the equality operators.
  * Correct: `if foo is None:`
  * Incorrect: `if foo == None:`
* Use `is not` operator rather than `not ... is`.
  * Correct: `if foo is not bar:`
  * Incorrect: `if not foo is bar:`
* When implementing ordering operations with rich comparisons, it is best to implement all six operations (`__eq__`, `__ne__`, `__lt__`, `__le__`, `__gt__`, `__ge__`) or use the `functools.total_ordering` decorator.
* Always use a `def` statement instead of an assignment statement that binds a lambda expression directly to an identifier.
  * Correct: `def f(x): return 2 * x`
  * Incorrect: `f = lambda x: 2 * x`
* Derive exceptions from `Exception` rather than `BaseException`.
* Use exception chaining explicitly (`raise X from Y`) when re-raising exceptions to preserve tracebacks.
* Use `''.startswith()` and `''.endswith()` instead of string slicing to check for prefixes or suffixes.
* Object type comparisons should always use `isinstance()` instead of comparing types directly.

---

## Pre-Release Verification Checklist

Prior to publishing code or merging PRs, ensure the following pass locally:

* `flake8` or `ruff` — Check for code style errors and enforce 79-character line limit.
* `black --line-length 79` — Format code to enforce layout rules automatically.
* Docstring audit — Verify all docstring blocks adhere to the 72-character margin limit.
