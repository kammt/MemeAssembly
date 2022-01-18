CC=gcc
CFLAGS=

ifeq ($(shell uname -s),Linux)
CFLAGS += -gstabs -no-pie
endif

.PHONY: all clean

all: toupper_test.c
	memeasm  -g -O -o toupper.o toupper.memeasm
	$(CC) -o toupper $^ toupper.o $(CFLAGS)

clean:
	rm toupper
