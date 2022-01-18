CC=gcc
CFLAGS=

ifeq ($(shell uname -s),Linux)
CFLAGS += -gstabs -no-pie
endif

.PHONY: all clean

all: stalinsort_test.c
	memeasm  -O -o stalinsort.o stalin-sort.memeasm -g
	$(CC) -o stalinsort $^ stalinsort.o $(CFLAGS)

clean:
	rm stalinsort
