#include <stdio.h>

extern void toupper_str(char* str);

int main() {
    char str[] = "The qu1ck BR0wN FoX juMpZ ov3r ThE l4zY dog. Hel1, Y3aH!";
    printf("Before toupper_str: %s\n", str);
    toupper_str(str);
    printf("After toupper_str: %s\n", str);
}    
