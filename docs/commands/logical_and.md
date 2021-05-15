### Definition
```bitconneeeeeeect [register] [value/register]```
### Description
Performs a logical and on a register with a given value or register.

### Origin
This command is derived from the "Bitconnect Carlos" meme. More information can be found [here](https://knowyourmeme.com/memes/bitconnect-carlos). \
![Bitconnect GIF](https://media.tenor.com/images/b8a1e87bb428e9e36926151d072aa991/tenor.gif)

### Examples
#### 1. Calculate eax mod 16
```
bitconneeeeeeect eax 15
```

#### 2. Move the number 12 to the stack, pop that number back to ebx, perform a logical and on eax and ebx and then crash with a segmentation fault
```
stonks 12
not stonks ebx
bitconneeeeeeect eax ebx
guess I'll die
```