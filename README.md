# Lab 4: Choose-your-Own Distributed System

As indicated in the lab writeup, you have an opportunity to design and implement your own distributed system, service, or application.  There is a lot of flexibility, so we aren't providing you with starter code.  This nearly empty repo should be your starting point, and you can build off any of your previous lab outcomes.  As usual, please use this repo for your project work, so the course staff can help you along the way


## Description of project topic, goals, and tasks

...

## Dependencies to run this code

In order to build `tonic` >= 0.8.0, you need the `protoc` Protocol Buffers compiler, along with Protocol Buffers resource files.

#### Ubuntu

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
```

#### Alpine Linux

```sh
sudo apk add protoc protobuf-dev
```

#### macOS

Assuming [Homebrew](https://brew.sh/) is already installed. (If not, see instructions for installing Homebrew on [the Homebrew website](https://brew.sh/).)

```zsh
brew install protobuf
```

## Description of tests and how to run them

1. Test for...

```
make test
```
