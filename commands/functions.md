## Defining a function
### Definition
`I like to have fun, fun, fun, fun, fun, fun, fun, fun, fun, fun [function name]`
### Description
Defines a function. Function names must follow the naming convention of labels defined by the GNU Assembler. This means that the following rules apply:
- Only the characters 0-9, a-z, A-Z, _, $ and . are allowed
- The first character is not allowed to be a number

?> All statements must be inside a function. 

!> If an executable is created, a function labeled 'main' must exist

## Call a function
### Definition
`[function name]: whomst has summoned the almighty one`
### Description
Calls a function. This can also be an external function, as long as you are not creating an executable. In that case, the called function must also be defined in your MemeASM code.

## Exit a function
### Definition
`right back at ya, buckaroo`
### Description
Exits a function. The return value can be defined by the programmer.

## Exit success
### Definition
`I see this as an absolute win`
### Description
Exits a function with return value 0

## Exit failure
### Definition
`no, I don't think I will`
### Description
Exits a function with return value 1