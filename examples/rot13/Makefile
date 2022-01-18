CC=gcc
CFLAGS+=-gstabs -no-pie

.PHONY: all clean

all: rot13_test.c
	memeasm  -g -O -o rot13.o rot13.memeasm
	$(CC) -o rot13 $^ rot13.o $(CFLAGS)

clean:
	rm rot13
