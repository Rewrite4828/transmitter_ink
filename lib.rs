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

    #[derive(Debug,PartialEq,scale::Decode, scale::Encode)]
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

        /// Constructor.
        #[ink(constructor)]
        pub fn new() -> Transmitter {
            Transmitter {
                names: Mapping::new(),
                messages: Mapping::new(),
            }
        }

        /// Attempts to register a new name connected to your account id.
        #[ink(message)]
        pub fn register_name(&mut self, name: String) -> Result<(),Error> {

            if self.names.contains(&name) {

                return Err(Error::NameTaken);

            } else {

                self.names.insert(&name,&self.env().caller());

                return Ok(());

            }

        }

        /// Attempts to send a message to another user using one of your names.
        /// The name from which you wish the message to be sent must be specified.
        #[ink(message)]
        pub fn send_message(&mut self, from: Name, to: Name, content: Content) -> Result<(),Error> {

            if let Some(account_id) = self.names.get(&from) {

                if account_id != self.env().caller() {

                    return Err(Error::WrongAccount(from));

                }

                if let None = self.names.get(&to) {

                    return Err(Error::NameNonexistent(to));

                }

                if let Some(mut messages) = self.messages.get(&to) {

                    messages.push( Message { from, content });

                    self.messages.insert(&to, &messages);

                    return Ok(());

                } else {

                    let mut messages = Vec::<Message>::new();

                    messages.push( Message { from, content } );

                    self.messages.insert(&to, &messages);

                    return Ok(());

                }

            } else {

                return Err(Error::NameNonexistent(from));

            }

        }

        /// Attempts to make all the messages that were sent to a specific name of yours available.
        #[ink(message)]
        pub fn get_messages(&self, belonging_to: Name) -> Result<Vec<Message>,Error> {
            
            if let Some(account_id) = self.names.get(&belonging_to) {

                if account_id != self.env().caller() {

                    return Err(Error::WrongAccount(belonging_to));

                }

                if let Some(messages) = self.messages.get(&belonging_to) {

                    if messages.len() == 0 {

                        return Err(Error::NoMessages);

                    }

                    return Ok(messages);

                } else {

                    return Err(Error::NoMessages);

                }

            } else {

                return Err(Error::NameNonexistent(belonging_to));

            }

        }

        /// Attempts to find and delete the specified message. The account name must be specified.
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

            let mut transmitter = Transmitter::new();

            if let Err(e) = transmitter.register_name("Alice".to_string()) {
                panic!("Encountered error {:?} whilst registering Alice's name.",e)
            };

            if let Err(e) = transmitter.register_name("Bob".to_string()) {
                panic!("Encountered error {:?} whilst registering Bob's name.",e)
            };

            if let Err(e) = transmitter.send_message(
                "Alice".to_string(),
                "Bob".to_string(),
                "Hello, Bob!".to_string()
            ) {
                panic!("Encountered error {:?} whilst sending message to Bob.",e)
            };

            if let Err(e) = transmitter.send_message(
                "Alice".to_string(),
                "Bob".to_string(),
                "Have a nice day!".to_string()
            ) {
                panic!("Encountered error {:?} whilst sending message to Bob.",e)
            };

            match transmitter.get_messages("Bob".to_string()) {
                Ok(messages) => {

                    if messages.len() != 2 {

                        panic!("Expected to get 2 messages, instead got {}",messages.len());

                    }

                },
                Err(e) => {

                    panic!("Encountered error {:?} whilst getting Bob's messages.",e)

                }
            };
            
            if let Err(e) = transmitter.delete_message(
                "Bob".to_string(),
                "Alice".to_string(),
                "Hello, Bob!".to_string()
            ) { 
                panic!("Encountered error {:?} whilst deleting message.",e)
            };

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
