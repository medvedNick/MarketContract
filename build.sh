#!/bin/bash
set -e

cd token
./build.sh
cd ../market
./build.sh
cd ..
