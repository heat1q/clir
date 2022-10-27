#!/bin/bash

mkdir -p test_files
mkdir -p test_files/c
mkdir -p test_files/f


head -c 1024 /dev/urandom > test_files/a.tmp
head -c 1024 /dev/urandom > test_files/b.tmp
head -c 1024 /dev/urandom > test_files/c/d.tmp
head -c 1024 /dev/urandom > test_files/f/f.tmp
