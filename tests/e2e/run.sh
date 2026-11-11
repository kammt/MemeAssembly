#!/bin/sh
# End-to-end test suite for the MemeAssembly compiler rewrite. See tests/e2e/run.py --help.
exec python3 -I "$(dirname "$0")/run.py" "$@"
