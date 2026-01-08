# CASE statements

`CASE` statements allow one out of several branches of code to be executed, depending on the value of a variable.
`CASE` statements are written as follows:

```
CASE OF <identifier>
    <value 1> : <statement1>
                <statement2>
                ...
    <value 2> : <statement1>
                <statement2>
                ...
    ...
ENDCASE
```

An `OTHERWIS`E clause can be the last case:
```
CASE OF <identifier>
    <value 1> : <statement1>
                <statement2>
                ...
    <value 2> : <statement1>
                <statement2>
                ...
    OTHERWISE : <statement1>
                <statement2>
                ...
ENDCASE
```
Each value may be represented by a range, for example:
```
<value1> TO <value2> : <statement1>
<statement2>
...
```
Note that the `CASE` clauses are tested in sequence. When a case that applies is found, its statement is executed and the `CASE` statement is complete. Control is passed to the statement after the `ENDCASE`. Any remaining cases are not tested.
If present, an `OTHERWISE` clause must be the last case. Its statement will be executed if none of the preceding cases apply.