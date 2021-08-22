## Set a breakpoint
### Definition
```it's a trap```
### Description
Raises SIGTRAP. If a debugger is attached, this breaks into the debugger, allowing you to see all current register values and manually step through your code. If no debugger is attached, the process is terminated and a core dump is generated.

?> Make sure to use `-g` if you want to debug your program