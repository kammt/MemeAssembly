## Moving values into registers
### Definition
```[register] is brilliant, but I like [value/register]```
### Description
Moves the specified value or register into the destination register.

## Zero a register
### Definition
`sneak 100 [register]`
### Description
Sets the value of a register to 0

## Examples
#### 1. Exchange the values of eax and ebx without using the stack
```
ecx is brilliant, but I like ebx
ebx is brilliant, but I like eax
eax is brilliant, but I like ecx
```

#### 2. Set ebx, ecx and edx to the value of eax
```
ebx is brilliant, but I like eax
ecx is brilliant, but I like eax
edx is brilliant, but I like eax
```

#### 3. Move the value at address eax into ebx
```
ebx is brilliant, but I like eax do you know de wey
```

#### 4. Intentionally cause a segFault by modifying the Stack Pointer
```
rsp is brilliant, but I like 69
```