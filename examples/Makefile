.PHONY: all

all:
	for file in *.memeasm ; do memeasm -o $$(basename $$file .memeasm) $$file || exit 1 ; done
	for dir in */ ; do cd $$dir && make || exit 1 && cd .. ; done

