# Lottery [![Built with ink!](https://raw.githubusercontent.com/paritytech/ink/master/.images/badge_flat.svg)](https://github.com/paritytech/ink)
A smart contract based on the ink! lang by parity to implement a simple lottery within a contracts pallet.
The pallet side is simply assumed and in general no tests have yet been written to confirm the correctness. 

The randomness used to select the winner is brought into the scope of the contract via the chain extension that provides a source for randomness.

The whole logic is made by me but lots of the code is either taken or heavily inspired by the ink! documentation or their implementation examples.

