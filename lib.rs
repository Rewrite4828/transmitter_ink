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
        NameNonexistent(Name),
        WrongAccount(Name),
        NoMessages,
        MessageNonexistent,
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

        #[ink(message)]
        pub fn send_message(&mut self, from: Name, to: Name, content: Content) -> Result<(),Error> {

            if let Some(account_id) = self.names.get(&from) {

                if account_id != self.env().caller() {

                    return Err(Error::WrongAccount(from));

                }

                if let Some(mut messages) = self.messages.get(&to) {

                    messages.push( Message { from, content });

                    return Ok(());

                } else {

                    return Err(Error::NameNonexistent(to));

                }

            } else {

                return Err(Error::NameNonexistent(from));

            }

        }

        #[ink(message)]
        pub fn delete_message(&mut self, belonging_to: Name, from: Name, content: Content) -> Result<(),Error> {

            let message_to_del = Message { from, content };

            if let Some(account_id) = self.names.get(&belonging_to) {

                if account_id != self.env().caller() {

                    return Err(Error::WrongAccount(belonging_to));

                }

                if let Some(mut messages) = self.messages.get(&belonging_to) {

                    let mut msg_pos = None;

                    for (pos,message) in messages.iter().enumerate() {

                        if *message == message_to_del {
                            msg_pos = Some(pos);
                        } 

                    }

                    if let Some(pos) = msg_pos {

                        messages.remove(pos);

                        return Ok(());

                    } else {

                        return Err(Error::MessageNonexistent);

                    }

                } else {
                    
                    return Err(Error::NoMessages);

                }

            } else {

                return Err(Error::NameNonexistent(belonging_to));

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
