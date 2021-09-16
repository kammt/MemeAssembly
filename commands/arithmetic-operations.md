## Adding two values
### Definition
```[register] units are ready, with [value/register] units well on the way```
### Description
Adds a value to the specified destination register

## Subtracting two values
### Definition
```parry [register/value] you filthy casual [register]```
### Description
Subtracts a value from the specified destination register

## Signed Multiplication
### Definition
`[register] is getting out of hand, now there are [value/register] of them`

### Description
Performs a signed multiplication while only changing the state of the destination register

## Signed Division
### Definition
`look at what [value/register] needs to mimic a fraction of [register]`

### Description
Performs a signed division without changing the state of any registers except for the destination register

## Divide by two
### Definition
```they had us in the first half, not gonna lie [register]```
### Description
Halves the value of the specified register. This is done by a binary right shift.

## Multiply by two
### Definition
```upgrades, people. Upgrades [register]```
### Description
Doubles the value of the specified register. This is done by a binary left shift.

## Increase/Decrease by 1
### Definition
`upvote [register]` / `downvote [register]`
### Description
Adds/Subtracts 1 from the specified destination register

## Examples
#### 1. Shift rax to the right by 6 bits
```
they had us in the first half, not gonna lie rax
they had us in the first half, not gonna lie rax
they had us in the first half, not gonna lie rax
they had us in the first half, not gonna lie rax
they had us in the first half, not gonna lie rax
they had us in the first half, not gonna lie rax
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

    I see this as an absolute win
```