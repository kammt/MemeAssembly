## Printing to the console
### Definition
```what can I say except [ASCII-character/8-bit register]```
### Description
Prints the given character to stdout. The given parameter can either be a letter itself, an ASCII-value in decimal form or an 8-bit register (e.g. al) containing an ASCII-value.

## Reading from stdin
### Definition
`let me in. LET ME IIIIIIIIN [8-bit register]`

### Description
Reads a single character from stdin into the specified register. If the user pressed enter, the register will contain \n, which is 10 in ASCII.


## Perform a syscall
### Definition
`we need air support`

### Description
Performs a syscall

## Examples
#### 1. Hello World!
```
I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main
    what can I say except H
    what can I say except e
    what can I say except l
    what can I say except l
    what can I say except o
    what can I say except \s
    what can I say except W
    what can I say except o
    what can I say except r
    what can I say except l
    what can I say except d
    what can I say except !
    what can I say except \n

    I see this as an absolute win
```
#### 2. Print the Unicode-Character "😎"
```
I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main
    what can I say except 240
    what can I say except 159
    what can I say except 152
    what can I say except 142
    
    I see this as an absolute win
```

#### 3. Print the alphabet with spaces in between and a new line at the end
```
I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main
    eax is brilliant, but I like 65

    upgrade
    what can I say except al
    what can I say except \s
    upvote eax
    corporate needs you to find the difference between eax and 91
    fuck go back

    they're the same picture
    what can I say except \n

    I see this as an absolute win
```