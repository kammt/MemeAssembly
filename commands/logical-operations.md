## Logical And
### Definition
```bitconneeeeeeect [register] [value/register]```
### Description
Performs a logical and on a register with a given value or register.

## Examples
#### 1. Calculate eax mod 16
```
bitconneeeeeeect eax 15
```

#### 2. Move the number 12 to the stack, pop that number back to rbx, perform a logical and on rax and rbx and then crash with a segmentation fault
```
stonks 12
not stonks rbx
bitconneeeeeeect rax rbx
guess I'll die
```