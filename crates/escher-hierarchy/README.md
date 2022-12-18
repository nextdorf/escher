# escher-hierarchy
Hierarchy crate for escher. While writing the UI I realized having a hierarchical data structure with elements that mutate each other is not as simply as one might expect. The main problem is the borrowchecking of safe rustlang. Wrting the UI system itself without a dedicated hierarchy system is probably possible with RefCells and smart pointers, but since I'll heavely rely on hierarchies for my plugin system later on, it is probably a good learning experience to write this crate now.

## Hierarchy system
The `Hierarchy` trait defines a collection for structs implementing the `Entity` trait. In order to work with rust's borrow checking every entity can mutate itself. The `hierarchy` is run which then runs all entries.

At the start the hierachy's runner is called with the initial Input `input_1` and a set of identities `ids`. It calls every entity's runner with an identity in `ids` on the input and an internal state of the hierarchy. Every called entity potentially returns a result. The vector over all results is then used to update the hierarchy's internal state, make additional updates to the map of entities and decide whether the run is done or if another round is started with some new `input_2`. One important restriction (which can't be enforced at compile time) is that the internal state and the entities do not share any data. The default implementation uses an unsafe code block which will possibly invoke data races if that restriction is not enforced manually.
```ascii
              +-------------------------------------+  +------------
              |      Collection with K entities     |  |
[Start]       +-----------------------+             |  +------------
   |          | (E_1) (E_2) ... (E_M) | (E_M+1) ... |  | (E'_1) ...
   |          +-----------------------+-------------+  +------------
   |             ^                 \                      ^
   |Input_1     /Input_N            \Result_N ----->---- /Input_N+1
   | (1..)     / (1..M)              \                  /  (1..M')
   v          /                       v                /
 (H_1)-...->(H_N)                    (H_N)--------->(H_N+1)
   |          |                        |               |
   +-(Run...)-+-(------------Run_N-----|-------------)-+-(Run_N+1...
                                       |
                                       +-------------->[Done]
```
