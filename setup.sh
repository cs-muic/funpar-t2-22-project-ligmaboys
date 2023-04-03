#!/bin/bash

# Install dependencies and setup pre-commit
cargo install --path .
pre-commit install
