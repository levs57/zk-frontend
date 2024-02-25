### snark frontend

This repo contains utility traits for frontend of multi-commitment proof systems (such as protostar, or multi-round groth16), with customizable witness types, and hopefully relatively backend agnostic.


### info

Importantly, it is a job of Circuit trait implementor to ensure that signals end up in at least one commitment group.
Additional commitment groups are implemented, but I did not want alloc API to ask for commitment group, so it is duty of a Circuit implementor to ensure there is a current default group.

### Design Choices

Library can be roughly partitioned into 4 (hopefuly not very entagled) parts:
- Frontend (user-side)
- Frontend (dev-side)
- Middleend
- Backend

#### Frontend (user-side)
Current state: Not final but usable.

Consists of ready to use gadgets and primitives that are relatively comfortable to use.
Gadgets can be easily enabled/disabled by implementing traits on Circuit, (we probably will provide macro for default impls). 
Mainstream gadget is a trait with blanket implementation for it's functionality that can effortlesly be implemented.
New gadgets are composed from other gadgets by requesting them as supertraits, typical flow should be simmilar to src/gadgets/traits/poseidon_permutation.rs

#### Frontend (dev-side)
Current state: A very unstable krutch-enabled hell.

API for variable/signal/const creation, advices, constraints etc.
Users are advised not to use theese APIs but they are sometimes crucial in implementing some behaviours e.g. src/gadgets/traits/poseidon_permutation.rs uses direct advice API to set initial_capacity.

#### Middlend
Current state: non existent

Should transform Circuit into execution graph for Backend. Should also apply finalizers and optimisations to said graph.

#### Backend
Current state: Nearly finished

Should compute execution graph and produce witness.

API (vaguely):
```
Advice {
    inputs(&self) -> Vec<UUID>;
    outputs(&self) -> Vec<UUID>;
    call(self, &mut storage);
}

BackendInput {
    inputs: Vec<UUID>,
    outputs: Vec<UUID>,
    edges: Vec<dyn Advice>,
}
```
Backend should execute graph interleaving with external inputs computation and return a Witness object that allows indexing with circuit addresses.