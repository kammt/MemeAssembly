### Definition
```[register] is brilliant, but I like [value/register]```
### Description
Moves the specified value or register into the destination register.

### Origin
This command is derived from the "This is brilliant, but I like this" meme. More information can be found [here](https://knowyourmeme.com/memes/this-is-brilliant-but-i-like-this). 

### Examples
#### 1. Exchange the values of eax and ebx without using the stack
```
ecx is brilliant, but I like ebx
ebx is brilliant, but I like eax
eax is brilliant, but I like ecx
```

#### 2. Set every register to the value of eax
```
ebx is brilliant, but I like eax
ecx is brilliant, but I like eax
edx is brilliant, but I like eax
```

#### 3. Intentionally make your program crash by modifying the stack pointer
```
esp is brilliant, but I like 69
```