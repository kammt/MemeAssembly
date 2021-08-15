#!/bin/bash
nfpm package -p deb -t "../"
nfpm package -p rpm -t "../"