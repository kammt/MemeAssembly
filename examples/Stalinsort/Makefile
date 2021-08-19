CC=gcc
CFLAGS+=-gstabs -no-pie

.PHONY: all clean

all: stalinsort_test.c
	memeasm  -O -o stalinsort.o stalin-sort.memeasm -g
	$(CC) -o stalinsort $^ stalinsort.o $(CFLAGS)

clean:
	rm stalinsort
