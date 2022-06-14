.PHONY: all

all:
	for file in *.memeasm ; do memeasm -o ex $$file ; done
	rm ex
	for dir in */ ; do cd $$dir && make && cd .. ; done

