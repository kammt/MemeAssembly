CC=gcc
CFLAGS+=-gstabs

.PHONY: all clean

all: stalinsort_test.c
	memeasm -g -O -o stalinsort.o stalin-sort.memeasm
	$(CC) -o stalinsort $^ stalinsort.o $(CFLAGS)

clean:
	rm stalinsort
