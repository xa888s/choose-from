//! Choose a K-selection of values from N choices, where N and K are set at compile time (or not).
//!
//! # Why is this useful?
//!
//! One use case (the one that made me write this), would be to ensure a closure provided by a library user
//! only returns a selection of values provided
//! to it.
//!
//! For example:
//! ```
//! #[derive(Clone, Copy)]
//! enum Suit {
//!     Clubs,
//!     Diamonds,
//!     Hearts,
//!     Spades,
//! }
//!
//! #[derive(Clone, Copy)]
//! struct Suits<const N: usize>([Suit; N]);
//!
//! impl<const N: usize> Suits<N> {
//!     // constructor
//!     pub fn with_suits(suits: [Suit; N]) -> Suits<N> {
//!         Suits(suits)
//!     }
//!     
//!     // where chooser is some external function that chooses from the provided suits
//!     pub fn choose_suit<C>(&self, chooser: C)
//!     where
//!         C: FnOnce([Suit; N]) -> Suit
//!     {
//!         // have user choose some suit
//!         let suit = chooser(self.0);
//!
//!         // do stuff with suit
//!         // ...
//!     }
//! }
//! ```
//! In the above case, we have a container that holds suits, and we
//! want the user to choose one suit from our inner suit array. As the
//! function is currently written however, the user could return any arbitrary
//! suit, even if it was not contained within our array.
//! ```
//! # #[derive(Clone, Copy)]
//! # enum Suit {
//! #     Clubs,
//! #     Diamonds,
//! #     Hearts,
//! #     Spades,
//! # }
//! #
//! # #[derive(Clone, Copy)]
//! # struct Suits<const N: usize>([Suit; N]);
//! #
//! # impl<const N: usize> Suits<N> {
//! #     // constructor
//! #     pub fn with_suits(suits: [Suit; N]) -> Suits<N> {
//! #         Suits(suits)
//! #     }
//! #     pub fn choose_suit<C>(&self, chooser: C)
//! #     where
//! #         C: FnOnce([Suit; N]) -> Suit
//! #     {
//! #         // have user choose some suit
//! #         let suit = chooser(self.0);
//! #
//! #         // do stuff with suit
//! #         // ...
//! #     }
//! # }
//! let suits = Suits::with_suits([Suit::Clubs, Suit::Diamonds]);
//!     
//! // this means choose_suit will get a spades, even though our array does not
//! // include spades
//! suits.choose_suit(|_| Suit::Spades);
//! ```
//! This is where we can use the functions in this library to force the user to take one of our
//! provided choices
//! ```
//! # #[derive(Clone, Copy)]
//! # enum Suit {
//! #     Clubs,
//! #     Diamonds,
//! #     Hearts,
//! #     Spades,
//! # }
//! #
//! # #[derive(Clone, Copy)]
//! # struct Suits<const N: usize>([Suit; N]);
//! #
//! use choose_from::{choose_from_fixed, Choice};
//!
//! impl<const N: usize> Suits<N> {
//! #   pub fn with_suits(suits: [Suit; N]) -> Suits<N> {
//! #         Suits(suits)
//! #   }
//!     // ...
//!     // where chooser is some external function that chooses from the provided suits
//!     pub fn choose_suit<C>(&self, chooser: C)
//!     where
//!         C: FnOnce([Choice<'_, Suit>; N]) -> [Choice<'_, Suit>; 1]
//!     {
//!         // have user choose some suit (this suit is guaranteed to be from our choices)
//!         let [suit]: [Suit; 1] = choose_from_fixed(self.0).with(chooser);
//!
//!         // do stuff with suit
//!         // ...
//!     }
//!     // ...
//! }
//! ```
//! ## Alternative?
//!
//! If you thought about it for a bit, you may realize that you can just use an enum
//! over "choosable" values, and then providing a mapping from that enum to our original
//! values:
//! ```
//! # #[derive(Clone, Copy)]
//! # enum Suit {
//! #     Clubs,
//! #     Diamonds,
//! #     Hearts,
//! #     Spades,
//! # }
//! #
//! # #[derive(Clone, Copy)]
//! # struct Suits<const N: usize>([Suit; N]);
//! #
//! pub enum ChoosableSuit {
//!     Clubs,
//!     Diamonds,
//! }
//!
//! impl ChoosableSuit {
//!     pub fn to_suit(self) -> Suit {
//!         match self {
//!             ChoosableSuit::Clubs => Suit::Clubs,
//!             ChoosableSuit::Diamonds => Suit::Diamonds,
//!         }
//!     }
//! }
//!
//! impl<const N: usize> Suits<N> {
//! #   pub fn with_suits(suits: [Suit; N]) -> Suits<N> {
//! #         Suits(suits)
//! #   }
//!     // ...
//!     // where chooser is some external function that chooses from the provided suits
//!     pub fn choose_suit<C>(&self, chooser: C)
//!     where
//!         C: FnOnce([ChoosableSuit; 2]) -> ChoosableSuit
//!     {
//!         // have user choose some suit (let's imagine these ChoosableSuits are from our choices)
//!         let suit: Suit = chooser([ChoosableSuit::Clubs, ChoosableSuit::Diamonds]).to_suit();
//!
//!         // do stuff with suit
//!         // ...
//!     }
//!     // ...
//! }
//! ```
//! This works! But this only works for returning a single value (plus it is kind of annoying to write a bunch of
//! boilerplate enums everytime you want to choose between some values). When we try to return multiple (non-duplicate) values
//! (as an array, tuple, Vec, etc.), we run into the same problem as earlier, where we can't stop a user from providing
//! two or more duplicate choices (this is an example of choices *with* replacement, when we want choices *without* replacement).
//!
//! ## Concrete use case
//!
//! Let's imagine `chooser` to be some GUI selector. This allows us to abstract away the
//! logic of actually getting a choice from an application user to the user of our library.
//! Which means that multiple implementations of `chooser` can use our library (web app, CLI, desktop, etc.).
//!
//! # How do they work?
//!
//! Values are assured to be from the selection through two ways.
//! First the only constructor for [Choice] is private
//! ```compile_fail
//! use choose_from::choose_from_fixed;
//!
//! // we cannot access the private constructor. And it requires a reference
//! // to a Guard that we cannot construct
//! let one = Choice::with_guard(1, unreachable!());
//! ```
//! So we know choices cannot be created out of thin air, but what about the
//! owned [Choice]s provided to us through [`with`](crate::Selector::with) (or similar methods)?
//! If we moved them out of the closure (since we have ownership), and then used them as choices
//! for a new [choose_from] with the same type, then we could return values that aren't from the
//! available choices! If we try to do that:
//! ```compile_fail
//! use choose_from::choose_from_fixed;
//!
//! let mut smuggler = Vec::new();
//! choose_from(vec![1, 2, 3, 4]).any_with(|choices| {
//!     // try to move last three values out of the closure
//!     smuggle.extend(choices.drain(1..));
//!     choices
//! });
//!
//! // use the smuggled value later to do nefarious stuff
//! // if this was possible weird_values wouldn't be from our
//! // provided choices
//! let weird_values = choose_from(vec![]).any_with(|_| smuggler);
//! ```
//! This fails to compile. Remember the Guard we mentioned earlier? All choices have a
//! lifetime specifier. They don't actually hold any value, but they act as if they hold
//! a reference to a Guard. This stops a [Choice] from living longer than the call to
//! [with](crate::SelectorArr) (and similar methods), since the reference for each Guard
//! only lives as long as the body of the method (since [Choice] "holds" a reference to the guard,
//! it cannot live longer than it). Both of these steps combine to ensure that the `chooser`
//! function *MUST* select value(s) from the provided ones.
//!
//! If you are interested in learning more try reading the code, it is quite simple.

mod arr;
mod choice;
mod vec;

pub use arr::SelectorArr;
pub use choice::Choice;
use choice::Guard;
pub use vec::Selector;

/// Wraps our choices and allows us to choose from them
pub fn choose_from<T>(choices: Vec<T>) -> Selector<T> {
    Selector::with_choices(choices)
}

/// Wraps our fixed number of choices and allows us to choose from them
/// ```
/// use choose_from::choose_from_fixed;
/// let chosen = choose_from_fixed(["Hi", "how", "are ya?"]).with(|[first, second, third]| {
///     // the provided choices allow inspection of the values
///     let first_val: &&str = first.value();
///     assert_eq!(*first_val, "Hi");
///     
///     // this is our selection
///     [first, third]
/// });
///
/// assert_eq!(chosen, ["Hi", "are ya?"]);
/// ```
pub fn choose_from_fixed<const N: usize, T>(choices: [T; N]) -> SelectorArr<N, T> {
    SelectorArr::with_choices(choices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_two_i32s_from_fixed_four() {
        let choices = [1, 2, 3, 4];
        let chosen = choose_from_fixed(choices).with(|[one, two, _, _]| [one, two]);

        assert_eq!(chosen, [1, 2]);
    }

    // TODO: write more tests
}
