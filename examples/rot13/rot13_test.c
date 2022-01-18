#include <stdio.h>

extern void rot13(char *str);

int main() {
	char str[] = "Nu, V frr lbh'er n zna bs phygher nf jryy.";
	printf("Before rot13: %s\n", str);
	rot13(str);
	printf("After rot13: %s\n", str);
}
