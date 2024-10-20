//! Choose a K-selection of values from N choices, where N and K are set at compile time.
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
//!     pub fn choose_suit(&self, chooser: impl FnOnce([Suit; N]) -> Suit) {
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
//! #     pub fn choose_suit(&self, chooser: impl FnOnce([Suit; N]) -> Suit) {
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
//! This is where we can use a Choose struct to force the user to take one of our
//! provided choices
//! ```
//! # use choose_from::{Choice, Choose};
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
//! impl<const N: usize> Suits<N> {
//! #   pub fn with_suits(suits: [Suit; N]) -> Suits<N> {
//! #         Suits(suits)
//! #   }
//!     // ...
//!     // where chooser is some external function that chooses from the provided suits
//!     pub fn choose_suit(&self, chooser: impl FnOnce([Choice<'_, Suit>; N]) -> [Choice<'_, Suit>; 1]) {
//!         // type inference :D (type would be: Choose<1, Suit, N>)
//!         let choices = Choose::with_choices(self.0);
//!         // have user choose some suit (this suit is guaranteed to be from our choices)
//!         let [suit]: [Suit; 1] = choices.choose_with(chooser);
//!
//!         // do stuff with suit
//!         // ...
//!     }
//!     // ...
//! }
//! ```
//! Let's imagine `chooser` to be some GUI selector. This allows us to abstract away the
//! logic of actually getting a choice from an application user to the user of our library.
//! Which means that multiple implementations of `chooser` can use our library (web app, CLI, desktop, etc.).

#[derive(Debug)]
struct Guard;

/// A specific choice, yielded by [`Choose::with_choices`]
#[derive(Debug)]
pub struct Choice<'guard, T> {
    value: T,
    _guard: &'guard Guard,
}

impl<'guard, T> Choice<'guard, T> {
    fn with_guard(value: T, guard: &'guard Guard) -> Choice<'guard, T> {
        Choice {
            value,
            _guard: guard,
        }
    }

    /// Allows you to inspect the value of the choice
    pub fn value(&self) -> &T {
        &self.value
    }

    fn into_inner(self) -> T {
        self.value
    }
}

/// Container that guarantees a selection of its values, where N is
/// the possible choices, and K is the size of the selection
// Read as: Choose K Ts from N (Ts)
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Choose<const K: usize, T, const N: usize> {
    choices: [T; N],
}

impl<const K: usize, T, const N: usize> Choose<K, T, N> {
    /// Creates a Choose struct with provided choices
    /// ```
    /// # use choose_from::Choose;
    /// // here we are creating a Choose to select 2 elements from 3
    /// let choices: Choose<2, _, 3> = Choose::with_choices([1, 2, 3]);
    ///
    /// // choose 2 from 3
    /// // ...
    /// ```
    pub fn with_choices(choices: [T; N]) -> Choose<K, T, N> {
        const {
            assert!(
                N >= K,
                "Selection must be smaller than or equal to the number of choices"
            );
        }
        Choose { choices }
    }

    /// The main method of this type. The closure `chooser` is used to take choice(s) from our provided
    /// choice(s) by returning a K-selection of it. The values of these choices are then
    /// returned by the function.
    /// ```
    /// # use choose_from::Choose;
    /// // types are inferred :D
    /// let choices = Choose::with_choices(["Hi", "how", "are ya?"]);
    ///
    /// let chosen = choices.choose_with(|[first, second, third]| {
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
    // we pass our possible choices to the function wrapped in Choice, which only allows
    // inspection of the value, and we must return an array of size K back full
    // of our choices. The values returned are GUARANTEED to only come from our original
    // choices thanks to the Choice struct and guard
    pub fn choose_with(
        self,
        chooser: impl FnOnce([Choice<'_, T>; N]) -> [Choice<'_, T>; K],
    ) -> [T; K] {
        // here we use a guard to prevent the caller from "smuggling" a value out of the closure.
        // This ensures that Choice values built from our given choices are only
        // available within the closure (they can't escape), since Choice has no
        // publicly accessible constructor.
        //
        // For example, this should fail to compile:
        //
        // let choices = Choose::with_choices([1, 2, 3, 4]);
        // let mut extras = Vec::new();
        // let chosen = choices.choose_with(|[one, two, three, _]| {
        //     extras.push(three);
        //     [one, two]
        // });
        //
        // because extras is being used to "smuggle" the third choice out, which would be possible
        // if we didn't have a guard since we are passing the closure an array of owned Choices
        //
        let _guard = Guard;
        let choices = self.choices.map(|t| Choice::with_guard(t, &_guard));

        chooser(choices).map(Choice::into_inner)
        // _guard is dropped when function returns, which means that no one
        // has any Choice values anymore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_two_i32s_from_four() {
        let choices = Choose::with_choices([1, 2, 3, 4]);
        let chosen = choices.choose_with(|[one, two, _, _]| [one, two]);

        assert_eq!(chosen, [1, 2]);
    }
}
