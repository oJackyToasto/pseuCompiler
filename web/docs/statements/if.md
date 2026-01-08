# IF statements

IF statements are used to select based on a boolean value. 

IF statements may or may not have an `ELSE` clause.
IF statements without an else clause are written as follows:

```
IF <condition> THEN
    <statement(s)>
ENDIF
```

IF statements with an else clause are written as follows:

```
IF <condition> THEN
    <statement(s)>
ELSE
    <statement(s)>
ENDIF
```

NESTED IF statements are written as follows:

```
IF <condition> THEN
    <statement(s)>
ELSE
    IF <condition> THEN
        <statement(s)>
    ENDIF
ENDIF
```