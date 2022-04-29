# This should work just fine
main: main.memeasm function.memeasm
	memeasm -o $@ $^

.PHONY: fail_undefined fail_no_main clean

# This should fail as the function referenced in main.memeasm is defined in a file that is not given as input file
fail_undefined: main.memeasm
	memeasm -o $@ $^

# This should fail as we don't have a main function
fail_no_main: function.memeasm
	memeasm -o $@ $^

clean:
	rm -f main fail_undefined fail_no_main
