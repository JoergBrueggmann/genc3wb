# Example: a meta compiler DSL produced by a node

This example is a *meta compiler-compiler network* of two *nodes* in which
the *output* of the first is the *meta compiler DSL* of the second: the
*node* `writer` generates 'reader.gc3', and the *node* `reader` is
configured by it.

| File | Holds |
| --- | --- |
| 'letters.txt' | the *input* of `writer`: the letters `reader` is to accept, as alternatives of its syntax |
| 'writer.gc3' | the *meta compiler DSL* of `writer`: a generator that writes the *meta compiler DSL* of `reader` |
| 'text.txt' | the *input* of `reader`: a text of those letters |
| 'network.gc3n' | the network: `writer` writes 'reader.gc3', which `reader` reads as its *meta compiler DSL* |
| 'expected/reader.gc3', 'expected/copy.txt' | what the run has to produce |

## Running it

    genc3d --network network.gc3n --build

The run writes 'reader.gc3' and 'copy.txt', and they are the ones under
'expected'.

## The activation order

The *factory* starts both *nodes*; `reader` is started with
`--meta-dsl-produced`, reads no 'reader.gc3' and is a *waiting node*. The
*factory* activates the *nodes* one after the other in the *activation
order*, `writer` first: once active, `writer` transmits 'reader.gc3' to
`reader`, which applies it as its configuration and becomes a *valid
node*; then the *factory* activates `reader`, and the network is an
*active network*.

A change of 'letters.txt' is taken incrementally: `writer` derives it
again and transmits the difference of 'reader.gc3' to `reader`, which
reconfigures itself and derives 'text.txt' again.
