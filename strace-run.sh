#!/bin/bash

timeout 0.5 strace -r -o strace-${EPOCHREALTIME}.file "./target/debug/syscall-sandbox"
