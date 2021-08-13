## Single-use jump markers
### Definition
```
upgrade
[...]
fuck go back
```
```
banana
[...]
where banana
```
### Description
Defines a jump marker and jumps back to that marker at a later point.
 
!> These jump markers can only be used once in a MemeASM file!

## Monke jumps
### Definition
```
monke [monke marker name]
[...]
return to monke [monke marker name]
```
### Description 
Defines a Monke jump marker and jumps back to that marker at a later point. Monke jump markers must only contain the letters ‘a’ and ‘u’, of which both must appear at least once. \
For example, "uuuaaauauauauauaaaua" is valid, while "uuuuuu" or "a" are not.

## Comparison jumps
### Definition
```
who would win? [value/register] or [value/register]
[...]
[value/register] wins
[...]
[value/register] wins
```
### Description
Compares the two given values or registers and jumps to the jump marker of the value or register that is greater than the other. If they are equal, no jump is performed.

## Check for equivalence
### Definition
```
corporate needs you to find the difference between [value/register] and [value/register]
[...]
they're the same picture
```
### Description
Compares the two given parameters and jumps to `they're the same picture` if they are equal

!> This comparison can only be used once in a MemeASM file!
