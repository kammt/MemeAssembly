## Memory access / pointers to memory
### Definition
```[register] do you know de wey```
### Description
Adding the suffix interprets the register not as a value but a pointer to memory.

## Crashing the program
### Definition
`guess I'll die`
### Description
Crashes the program with a Segmentation Fault.

## Jump randomly
### Definition
`confused stonks`
### Description
Jumps to a random line of code that is decided at compilation time.

?> All instances of `confused stonks` inside a MemeASM file will jump to the same line

## Balancing your code
### Definition
`perfectly balanced as all things should be`
### Description
Restore balance to your code by randomly deleting half the commands at compile-time. When using this command multiple times, half the lines are deleted for every instance of this command.

## Assertion
### Definition
`it's over 9000 [register]`
### Description
Checks if the value of the specified register is larger than 9000. If not, a General Protection Fault is generated.

## Remove Frame Pointer(TM)
### Definition
`refuses to elaborate and leaves`
### Description
Removes MemeASM-style frame pointers from the stack. Note that this is translated to `mov rbp, rsp; pop rsp`.

## Create a code barrier
### Definition
`you shall not pass!`
### Description
This command triggers an endless loop when reached.

## Destabilise your program
### Definition
`Houston, we have a problem`
### Description
Oh yes you do have a problem. This command sets the stack pointer to zero. Good luck with fixing that.

## Random bullshit go!
### Definition
`it's dangerous to go alone, take [register]`
### Description
Fills the specified register with a random value generated at runtime. Note that 8-Bit registers cannot be used as parameters.

## Do nothing
### Definition
`oh no! anyway`
### Description
This command gets translated to a `nop`. It literally does nothing.

## Violating the ABI
### Definition
`wait, that's illegal`
### Description
Intentionally violates the System V ABI by modifying callee-saved registers without restoring their original value.

## Ignore commands
### Definition
`[command] or draw 25`
### Description
The compiler will completely disregard the command before and instead add 25 to the register rax.