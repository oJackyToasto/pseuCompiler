# REPEAT-UNTIL (Postcondition) Loops
`REPEAT-UNTIL` loops are written as follows:
```
REPEAT
 <statement(s)>
UNTIL <condition>
```
The condition __must__ be an expression that evaluates to a `BOOLEAN`.
The statements in the loop will be executed at least once. The condition is tested after the statements are executed and if it evaluates to `TRUE` the loop terminates, otherwise the statements are executed again.