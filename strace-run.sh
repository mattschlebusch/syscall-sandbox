#!/bin/bash

timeout 0.5 strace -r -o strace-"$(date +%s)".file "./target/debug/syscall-sandbox"
