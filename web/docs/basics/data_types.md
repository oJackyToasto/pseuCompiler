# Pseudocode Data Types

The following keywords are used to designate some basic data types:
- `INTEGER`: a whole number
- `REAL`: a number capable of containing a fractional part
- `CHAR`: a single character
- `STRING`: a sequence of zero or more characters
- `BOOLEAN`: the logical values TRUE and FALSE
- `DATE`: a valid calendar date
- `ARRAY`: an array of type

Literals of the above data types are written as follows:
- `INTEGER`: Written as normal in the denary system, e.g. 5, –3
- `REAL`: Always written with at least one digit on either side of the decimal point, zeros being added if necessary, e.g. 4.7, 0.3, –4.0, 0.0
- `CHAR`: A single character delimited by single quotes e.g. ꞌxꞌ, ꞌCꞌ, ꞌ@ꞌ
- `STRING`: Delimited by double quotes. A string may contain no characters (i.e. the empty string). e.g. "This is a string", ""
- `BOOLEAN`: `TRUE`, `FALSE`
- `DATE`: This will normally be written in the format dd/mm/yyyy. However, it is good practice to state explicitly that this value is of data type DATE and to explain the format (as the convention for representing dates varies across the world).

Identifiers (the names given to variables, constants, procedures and functions) are in mixed case. They can only contain letters (A–Z, a–z), digits (0–9) and the underscore character ( _ ). They must start with a letter and not a digit. Accented letters should not be used.

It is good practice to use identifier names that describe the variable, procedure or function they refer to. 

Single letters may be used where these are conventional (such as i and j when dealing with array indices, or X and Y when dealing with coordinates) as these are made clear by the convention.

Keywords identified elsewhere in this guide should never be used as variable names.

Identifiers should be considered case insensitive, for example, Countdown and CountDown should not be used as separate variable names.