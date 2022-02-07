#!/bin/bash
set -e

cd token
./test.sh
cd ../market
./test.sh
cd ..
