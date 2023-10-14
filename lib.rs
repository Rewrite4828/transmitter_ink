#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod transmitter {

    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};

    pub type Name = String;
    pub type Content = String;

    #[derive(PartialEq,scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Message {
        from: Name,
        content: Content,
    }

    #[ink(storage)]
    pub struct Transmitter {
        names: Mapping<Name,AccountId>,
        messages: Mapping<Name,Vec<Message>>,
    }

    #[derive(PartialEq,scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Error {
        NameTaken,
    }

    impl Transmitter {

        #[ink(constructor)]
        pub fn new() -> Transmitter {
            Transmitter {
                names: Mapping::new(),
                messages: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn register_name(&mut self, name: String) -> Result<(),Error> {

            if self.names.contains(&name) {

                return Err(Error::NameTaken);

            } else {

                self.names.insert(&name,&self.env().caller());

                return Ok(());

            }

        }

    }


    #[cfg(test)]
    mod tests {

        use super::*;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    }
}
