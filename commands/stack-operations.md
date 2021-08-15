## Pushing onto the stack
### Definition
```stonks [value/register]```
### Description
Pushes the given value or register to the stack. Only 64 bit registers are supported by this operation. The stack can be used to temporarily save values that do not fit into registers.

## Removing from the stack
### Definition
```not stonks [register]```
### Description
Moves the last value on the stack into the specified register. The amount of data removed from the stack depends on the size of the destination register.


!> Values pushed to the stack have to be removed before the end of your function or else a Segmentation Fault will occur!

## Examples
#### 1. Exchange the values of rax and rbx using the stack
```
stonks rax
stonks rbx
not stonks rax
not stonks rbx
```

#### 2. Repeatedly print the alphabet by saving and restoring rax
```
I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun main
    rax is brilliant, but I like 65
    stonks rax

    upgrade
    what can I say except al
    what can I say except \s
    upvote rax
    corporate needs you to find the difference between rax and 91
    fuck go back

    they're the same picture
    what can I say except \n

    not stonks rax
    stonks rax

    fuck go back

    I see this as an absolute win!
```