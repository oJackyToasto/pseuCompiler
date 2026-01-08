# FOR (Count-controlled) Loops

`FOR` (Count-controlled) loops are written as follows:
```
FOR <identifier> ← <value1> TO <value2>
 <statement(s)>
NEXT <identifier>
```
The identifier __must__ be a variable of data type `INTEGER`, and the values should be expressions that evaluate to integers.
The variable is assigned each of the integer values from `value1` to `value2` inclusive, running the statements inside the `FOR` loop after each assignment. If `value1 = value2` the statements will be executed once, and if `value1 > value2` the statements will not be executed.
It is good practice to repeat the identifier after `NEXT`, particularly with nested `FOR` loops.

An increment can be specified as follows:
```
FOR <identifier> ← <value1> TO <value2> STEP <increment>
 <statement(s)>
NEXT <identifier>
```

The increment must be an expression that evaluates to an ``INTEGER``. In this case the identifier will be assigned the values from `value1` in successive increments of increment until it reaches `value2`. If it goes past `value2`, the loop terminates. The increment can be negative.