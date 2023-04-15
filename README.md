# libp2p-workshop

Hi there,

Welcome to the libp2p workshop.

We will build a peer-to-peer decentralized chat app using libp2p. Our
application will allow anyone with internet access across the globe to
communicate without depending on any central infrastructure. The workshop will
give hands-on experience on how to build peer-to-peer vs. client-to-server.

## Before the workshop

1. Install git.

   https://git-scm.com/book/en/v2/Getting-Started-Installing-Git

1. Install Rust Programming Language.

   https://www.rust-lang.org/tools/install

1. Clone this repository.

   """sh
   $ git clone https://github.com/mxinden/libp2p-workshop.git
   """

1. Make sure you can compile the _hello world_ program in this repository on the
   "main" branch.

   """sh
   $ cargo run
   Finished dev [unoptimized + debuginfo] target(s) in 0.04s
   Running "target/debug/libp2p-workshop-node"
   Hello, world!
   """

Done? Great. You are all set for the workshop.

## Workshop

### Iteration 1

For iteration 1, we want to achieve the following things:

1. Add "libp2p" as a dependency to our project.
2. Activate the following features: `["async-std", "yamux", "noise", "tcp", "dns", "ping"]`.
3. Create a `libp2p::identity::Keypair`.
4. Create a DNS transport that is baked by TCP, upgrade it with noise and yamux.
5. Create a `libp2p::swarm::Swarm` with the `ping::Behaviour`.
6. Poll the swarm via `swarm.next_event()` and print some interesting events:
- Established connections
- Closed connections
- ping round-trip time

### Iteration 2

You may notice that once the connection is established, it is immediately closed again.
This is because libp2p by default closes idle connections and our connection doesn't do anything interesting.
For workshop purposes, we can fix that by adding the `keep_alive` behaviour to our swarm.

This way, we are going to learn how to compose multiple behaviours together to form an application.

Create a new `Behaviour` struct and compose it with the `ping::Behaviour` and the `keep_alive::Behaviour`.
Then, replace the `ping::Behaviour` in the swarm with your new `Behaviour` struct.
Make sure to derive `libp2p::swarm::NetworkBehaviour` on it.

Additionally, activate the `identify` feature and compose the `identify` behaviour into the event.

### Iteration 3

In iteration 3, we are going to abstract away the networking event loop in our application.

Take the loop and extract it into an `async fn`.
Additionally, add a bounded `futures` mpsc channel and use `futures::future::select` to poll the swarm and the channel.

This allows us to continuously poll the swarm for new events while also being able to send messages to it.

We are going to model the messages as an `enum`, for now we only need one message: `Dial`.
It should take a `Multiaddr` and instruct the `Swarm` to dial it.

## Additional Resources

Below are a couple of resources for those interested in reading more about
the stack used in this workshop.
**No knowledge is required in order to participate in the workshop!**

### Libp2p

Libp2p is a modular network stack that enables the development of peer-to-peer network applications.

- Introduction to Libp2p: <https://docs.libp2p.io/introduction/>
- Tutorial for getting started with rust-libp2p: <https://github.com/libp2p/rust-libp2p/blob/master/src/tutorials/ping.rs>
- Libp2p Specs: <https://github.com/libp2p/specs>

### Rust Programming Language

In this workshop we are using the Rust implementation of the libp2p networking stack.

- Rust Getting started: <https://www.rust-lang.org/learn/get-started>
- The Rust Book: <https://doc.rust-lang.org/stable/book/>
