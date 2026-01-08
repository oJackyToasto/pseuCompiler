# Declarations

Declarations are made as follows:
```
DECLARE <identifier> : <data_type>
```

For conveinience, this interpreter has supported assigning a value while declaring, made as follows:
```
DECLARE <identifier> <- <value> : <data_type>
```

For conveinience, this interpreter has also supported multiple declarations in one line, made as follows:
```
DECLARE <identifier>, <identifier>, ... : <data_type>
```

The above is _NOT_ an CAIE standard, so use to your own risk.

---

Constants are normally declared at the beginning of a piece of pseudocode (unless it is desirable to restrict the scope of the constant).

Constants are declared by stating the identifier and the literal value in the following format:
```
CONSTANT <identifier> = <value>
```