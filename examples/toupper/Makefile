CC=gcc
CFLAGS+=-gstabs -no-pie

.PHONY: all clean

all: toupper_test.c
	memeasm  -O -o toupper.o toupper.memeasm
	$(CC) -o toupper $^ toupper.o $(CFLAGS)

clean:
	rm toupper
