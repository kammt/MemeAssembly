.PHONY: all

all:
	set -e
	for file in *.memeasm ; do memeasm -o $$(basename $$file .memeasm) $$file ; done
	for dir in */ ; do cd $$dir && make && cd .. ; done

