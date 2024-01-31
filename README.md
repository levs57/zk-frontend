### snark frontend

This repo contains utility traits for frontend of multi-commitment proof systems (such as protostar, or multi-round groth16), with customizable witness types, and hopefully relatively backend agnostic.


### info

Importantly, it is a job of Circuit trait implementor to ensure that signals end up in at least one commitment group.
Additional commitment groups are implemented, but I did not want alloc API to ask for commitment group, so it is duty of a Circuit implementor to ensure there is a current default group.