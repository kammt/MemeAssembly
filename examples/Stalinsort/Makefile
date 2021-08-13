CC=gcc
CFLAGS+=-std=gnu11 -g

.PHONY: all clean

all: stalinsort_test.c
	memeasm -c -o stalinsort.S stalin-sort.memeasm
	$(CC) -o stalinsort $^ stalinsort.S $(CFLAGS)

clean:
	rm stalinsort
