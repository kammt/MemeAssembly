### Definition
```stonks [value/register]```
### Description
Pushes the given value or register to the stack. Only 16 and 32 bit registers are supported by this operation. The stack can be used to temporarily save values that do not fit into registers.
> Note: Values pushed to the stack have to be removed before the end of your code or else a Segmentation Fault will occur!

### Origin
This command is derived from the "Stonks" meme. More information can be found [here](https://knowyourmeme.com/memes/stonks).

### Examples
#### 1. Exchange the values of eax and ebx using the stack
```
stonks eax
stonks ebx
not stonks eax
not stonks ebx
```

#### 3. Repeatedly print the alphabet by saving and restoring eax
```
eax is great, but I want 65
stonks eax

upgrade
what can I say except al
what can I say except \s
upvote eax
corporate needs you to find the difference between eax and 91
fuck go back

they're the same picture
what can I say except \n

not stonks eax
stonks eax

fuck go back
```