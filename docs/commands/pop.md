### Definition
```not stonks [register]```
### Description
Moves the last value on the stack into the specified register. The amount of data removed from the stack depends on the size of the destination register.
> Note: Values pushed to the stack have to be removed before the end of your code or else a Segmentation Fault will occur!

### Origin
This command is derived from the "Stonks" meme. More information can be found [here](https://knowyourmeme.com/memes/stonks). \
![Bitconnect GIF](https://media.tenor.com/images/fa0bd5b188840241312b1f35ade00b9f/tenor.gif)

### Examples
#### 1. Exchange the values of eax and ebx using the stack
```
stonks eax
stonks ebx
not stonks eax
not stonks ebx
```

#### 2. Repeatedly print the alphabet by saving and restoring eax
```
eax is brilliant, but I like 65
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