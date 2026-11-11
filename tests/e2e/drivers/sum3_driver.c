/* Links against the -O object of programs/obj/sum3.memeasm (sum3(a,b,c) = a+b+c, hello() prints a line). */
#include <stdio.h>

extern long sum3(long a, long b, long c);
extern void hello(void);

int main(void) {
    fflush(stdout);
    hello();
    printf("%ld\n", sum3(1, 2, 3));
    printf("%ld\n", sum3(-100, 42, 1000000000000L));
    printf("%ld\n", sum3(0x7fffffffffffffffL, 1, 0));
    fflush(stdout);
    return (int)(sum3(10, 20, 12) & 0xff);
}
