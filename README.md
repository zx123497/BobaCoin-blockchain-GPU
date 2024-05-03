# Lab 4: Choose-your-Own Distributed System

As indicated in the lab writeup, you have an opportunity to design and implement your own distributed system, service, or application.  There is a lot of flexibility, so we aren't providing you with starter code.  This nearly empty repo should be your starting point, and you can build off any of your previous lab outcomes.  As usual, please use this repo for your project work, so the course staff can help you along the way


## Description of project topic, goals, and tasks

### Project Topic
Proof of Work (PoW) Blockchain Consensus

### Goals
In this lab, we will build a peer-to-peer blockchain system based on the Proof of Work consensus. The system allows us to start multiple worker nodes for mining blocks and client nodes for sending transactions to the blockchain. The blockchain system will attempt to mine blocks containing the requested transactions. The system ensures that all worker nodes eventually have the most up-to-date and longest blockchain.

### Tasks
1. Implement client node, a client node should be able to:
   - Create RSA key pairs.
   - Generate valid transactions.
   - A valid transaction should contain:
     - Id: UUID
     - Sender: Public Key of sender
     - Receiver: Public Key of receiver
     - Hash: SHA256 encrypted string
     - Signature: Sign the hash with private key
     - timestamp: timestamp

2. Impelment worker node, a worker node should be able to:
      - Connect to nodes in the network, and update the peer list when other nodes join the network.
      - Accept valid transactions from client nodes and update the transaction pool. Share transactions with other peer nodes.
      - Mine new blocks containing the transactions from the pool, based on Proof of Work. A valid block should contain:
        - Id
        - Difficulty
        - Hash
        - Previous Hash
        - Transactions
        - Timestamp
      - Sharing new, valid blocks with peer nodes should adhere to the Longest Chain Rule. That is, the nodes should only accept longer chains. If multiple blocks are mined simultaneously, the consensus protocol should be followed, ensuring that all nodes will eventually share a single version of the blockchain.
      - Once valid blocks have been accepted, the corresponding transaction is no longer in the pool


## Dependencies to run this code

In order to build `tonic` >= 0.8.0, you need the `protoc` Protocol Buffers compiler, along with Protocol Buffers resource files.  You will also need OpenSSL for RSA signatures.

#### Ubuntu

```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
sudo apt install openssl
```

#### Alpine Linux

```sh
sudo apk add protoc protobuf-dev
sudo apk add openssl
```

#### macOS

Assuming [Homebrew](https://brew.sh/) is already installed. (If not, see instructions for installing Homebrew on [the Homebrew website](https://brew.sh/).)

```zsh
brew install protobuf
brew install openssl@3
```

## Usage
To build the project
```
cargo b
```
or
```
make build
```
To start a worker node without a peer node (The first node in a blockchain network)
```zsh
cargo r <port>
```

Start a worker node with a specified peer
```zsh
cargo r <port> -p <peer-port>
```

To start a client, interact with specified worker node, should start worker nodes before running clients
```zsh
cargo r <port> -c
```

> #### Commands for Clients
> 
>`new` -- Create a new transaction and submit to the network.
>
>`exit`-- Exit the program.
>


## Description of tests and how to run them

To run tests:
```zsh
cargo test
```
or
```
make final
```
### 1. Test join network (10%)
   The test will start three worker nodes, expecting three of them will know each other.
### 2. Test submit transaction (10%)
   The test will start three worker nodes and send a transaction to one of them. It is expected that all three will eventually have the transaction in their own transaction pools.

### 3. Test mine block (15%)
It will start three worker nodes and send a transaction to one of the nodes. After one of the nodes finishes mining a new block, we expect all three of them to have a copy of the updated blockchain.

### 4. Test invalid block (15%)
It will start three worker nodes and send a transaction to one of the nodes. After one of the nodes has mined a new block, it will modify the block and attempt to append it to the blockchain on every node. We expect that the nodes will reject the modified block.

### 5. Test longer chain (20%)
It will start two worker nodes first and send a transaction to one of the nodes. After the two workers start mining, it will start the third worker. This worker will receive the transaction from the first two worker nodes and begin mining. The test will then send two more transactions to the network. Since the third worker starts mining slightly later than the first two workers, we expect that the third worker will switch to the longer chain and work on the next block.

### 6. Test fork (30%)
The test starts with two nodes that are not connected to each other. it will send different transactions to each of the nodes. The two nodes will create two different blockchains, representing a fork. Then, it will start two more worker nodes that know each other and update each of the new worker nodes with the two blockchains. Finally, it will send a new transaction to the network, expecting that the two nodes will eventually have the same blockchain (resolve the fork).

## 
   


   




