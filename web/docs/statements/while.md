# WHILE (Precondition) Loops

Due to the compatability of both _IGCSE_ and _AS/AL_, both syntax of the `WHILE` Loop has been supported.

_AS/AL_ `WHILE` loops are written as follows:
```
WHILE <condition>
    <statement(s)>
ENDWHILE
```
_IGCSE_ `WHILE` loops are written as follows:
```
WHILE <condition> DO
    <statement(s)>
ENDWHILE
```
The condition __must__ be an expression that evaluates to a `BOOLEAN`.
The condition is tested before the statements, and the statements will only be executed if the condition
evaluates to `TRUE`. After the statements have been executed the condition is tested again. The loop terminates when the condition evaluates to `FALSE`.
The statements will not be executed if, on the first test, the condition evaluates to `FALSE`.