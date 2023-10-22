#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod transmitter {

    use ink::storage::Mapping;
    use ink::prelude::{string::String, vec::Vec};
    use ink::env::hash::Sha2x256;

    pub type Username = Vec<u8>;
    pub type Content = Vec<u8>;

    #[derive(PartialEq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum MessageType {
        Text,
        Email { subject: String },
        // EmailAttachment { subject: String, mtype: Box<MessageType>}, //Looks like Box creates some problems - indeed
        // Request { id: u32 },
        // Response { id: u32, /*mtype: Box<MessageType>*/},
        Json,
        // Stream,
        Custom(String),
    }

    #[derive(PartialEq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Message {
        from: Username,
        mtype: MessageType,
        content: Content,
        hash: [u8;32],
    }

    #[ink(storage)]
    pub struct Transmitter {
        usernames: Mapping<Username,AccountId>,
        users: Mapping<AccountId,Vec<Username>>,
        messages: Mapping<Username,Vec<Message>>,
    }

    #[derive(Debug,PartialEq,scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Error {
        NameTaken,
        InvalidName,
        NameNonexistent(Username),
        WrongAccount(Username),
        NoMessages,
        MessageNonexistent,
        NoNames,
    }

    impl Transmitter {

        /// Constructor.
        #[ink(constructor)]
        pub fn new() -> Transmitter {
            Transmitter {
                usernames: Mapping::new(),
                users: Mapping::new(),
                messages: Mapping::new(),
            }
        }

        /// Attempts to register a new name connected to your account id.
        #[ink(message,payable)]
        pub fn register_name(&mut self, name: Vec<u8>) -> Result<(),Error> {

            if name.len() == 0 {
                
                return Err(Error::InvalidName);

            }

            if self.usernames.contains(&name) {

                return Err(Error::NameTaken);

            } else {

                self.usernames.insert(&name,&self.env().caller());

                if let Some(mut user_names) = self.users.get(&self.env().caller()) {

                    user_names.push(name);

                    self.users.insert(&self.env().caller(), &user_names);

                } else {

                    let mut user_names = Vec::<Username>::new();

                    user_names.push(name);

                    self.users.insert(&self.env().caller(), &user_names);

                }

                return Ok(());

            }

        }

        /// Lists the names registered to your account.
        #[ink(message)]
        pub fn get_names(&self) -> Result<Vec<Username>,Error> {

            if let Some(user_names) = self.users.get(&self.env().caller()) {

                return Ok(user_names);

            } else {

                return Err(Error::NoNames);

            }
        }

        /// Attempts to send a message to another user using one of your names.
        /// The name from which you wish the message to be sent must be specified.
        #[ink(message,payable)]
        pub fn send_message(&mut self, from: Username, to: Username, mtype: MessageType, content: Content) -> Result<(),Error> {

            if let Some(account_id) = self.usernames.get(&from) {

                if account_id != self.env().caller() {

                    return Err(Error::WrongAccount(from));

                }

                if let None = self.usernames.get(&to) {

                    return Err(Error::NameNonexistent(to));

                }

                if let Some(mut messages) = self.messages.get(&to) {

                    let mut to_be_hashed = Vec::<u8>::new();
                    to_be_hashed.extend(self.env().block_number().to_be_bytes());
                    to_be_hashed.extend(content.clone().iter());

                    let hash = self.env().hash_bytes::<Sha2x256>(&to_be_hashed);

                    messages.push( Message { from, mtype, content, hash });

                    self.messages.insert(&to, &messages);

                    return Ok(());

                } else {

                    let mut messages = Vec::<Message>::new();

                    let mut to_be_hashed = Vec::<u8>::new();
                    to_be_hashed.extend(self.env().block_number().to_be_bytes());
                    to_be_hashed.extend(content.clone().iter());

                    let hash = self.env().hash_bytes::<Sha2x256>(&content);

                    messages.push( Message { from, mtype, content, hash } );

                    self.messages.insert(&to, &messages);

                    return Ok(());

                }

            } else {

                return Err(Error::NameNonexistent(from));

            }

        }

        /// Attempts to make all the messages that were sent to a specific name of yours available.
        #[ink(message,payable)]
        pub fn get_all_messages(&self, belonging_to: Username) -> Result<Vec<Message>,Error> {
            
            if let Some(account_id) = self.usernames.get(&belonging_to) {

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

        // #[ink(message)]
        // pub fn get_message(&self, belonging_to: Name, hash: [u8;32]) -> Result<Message,Error> {

        //     if let Some(account_id) = self.names.get(&belonging_to) {

        //         if account_id != self.env().caller() {
                    
        //             return Err(Error::WrongAccount(belonging_to));

        //         }

        //         if let Some(messages) = self.messages.get(&belonging_to) {

        //             let mut msg_pos: Option<usize> = None;

        //             for (pos,message) in messages.iter().enumerate() {
                        
        //                 if message.hash == hash {

        //                     msg_pos = Some(pos);

        //                 }

        //             }

        //             if let Some(pos) = msg_pos {

        //                 return Ok(messages[pos]);

        //             } else {

        //                 return Err(Error::MessageNonexistent);

        //             }

        //         } else {

        //             return Err(Error::NoMessages);

        //         }

        //     } else {

        //         return Err(Error::NameNonexistent(belonging_to));

        //     }
        // }

        /// Attempts to find and delete the specified message. The account name and message hash must be specified.
        #[ink(message)]
        pub fn delete_message(&mut self, belonging_to: Username, hash: [u8;32]) -> Result<(),Error> {

            if let Some(account_id) = self.usernames.get(&belonging_to) {

                if account_id != self.env().caller() {

                    return Err(Error::WrongAccount(belonging_to));

                }

                if let Some(mut messages) = self.messages.get(&belonging_to) {

                    let mut msg_pos = None;

                    for (pos,message) in messages.iter().enumerate() {

                        if message.hash == hash {

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

            if let Err(e) = transmitter.register_name("Alice".into()) {
                panic!("Encountered error {:?} whilst registering Alice's name.",e)
            };

            if let Err(e) = transmitter.register_name("Bob".into()) {
                panic!("Encountered error {:?} whilst registering Bob's name.",e)
            };

            if let Err(e) = transmitter.send_message(
                "Alice".into(),
                "Bob".into(),
                MessageType::Text,
                "Hello, Bob!".into()
            ) {
                panic!("Encountered error {:?} whilst sending message to Bob.",e)
            };

            if let Err(e) = transmitter.send_message(
                "Alice".into(),
                "Bob".into(),
                MessageType::Text,
                "Have a nice day!".into()
            ) {
                panic!("Encountered error {:?} whilst sending message to Bob.",e)
            };

            let mut message_hash = [0u8;32];

            match transmitter.get_all_messages("Bob".into()) {
                Ok(messages) => {

                    if messages.len() != 2 {

                        panic!("Expected to get 2 messages, instead got {}",messages.len());

                    }

                    message_hash = messages[0].hash;


                },
                Err(e) => {

                    panic!("Encountered error {:?} whilst getting Bob's messages.",e)

                }
            };
            
            if let Err(e) = transmitter.delete_message(
                "Bob".into(),
                message_hash
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
        use ink_e2e::{build_message, subxt::book::setup::client};

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {

            let constructor = TransmitterRef::new();

            let contract_account_id = client
                .instantiate("transmitter", &ink_e2e::eve(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            macro_rules! new_name {
                ($name:literal) => {
                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.register_name($name.to_string()))
                };
            }

            macro_rules! send_message {
                ($from:literal -> $to:literal : $content:literal) => {
                        build_message::<TransmitterRef>(contract_account_id.clone())
                            .call(|transmitter| transmitter.send_message(
                                $from.to_string(),
                                $to.to_string(),
                                $content.into()))
                };
            }

            macro_rules! call_dry_run {
                (alice : $fn_name:tt, pay $amnt:tt) => {
                    client.call_dry_run(
                        &ink_e2e::alice(),
                        &$fn_name,
                        $amnt,
                        None)
                        .await
                };

                (bob : $fn_name:tt, pay $amnt:tt) => {
                    client.call_dry_run(
                        &ink_e2e::bob(),
                        &$fn_name,
                        $amnt,
                        None)
                        .await
                };

            }

            macro_rules! get_names {
                () => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.get_names())

                };
            }

            let new_name_alice = new_name!("Alice");

            let new_name_alice_result = call_dry_run!(alice: new_name_alice, pay 0);

            if let Err(e) = new_name_alice_result.return_value() { panic!("{:?}",e) };


            let new_name_bob = new_name!("Bob");

            let new_name_bob_result = call_dry_run!(bob: new_name_bob, pay 0);

            if let Err(e) = new_name_bob_result.return_value() { panic!("{:?}",e) };


            let get_user_names = get_names!();

            let get_user_names_result = call_dry_run!(alice: get_user_names, pay 0);

            if let Err(e) = get_user_names_result.return_value() { panic!("{:?}",e) };


            let send_message_alice = send_message!("Alice" -> "Bob" : "Hello, Bob!" );

            let send_message_alice_result = call_dry_run!(alice: send_message_alice, pay 0);

            if let Err(e) = send_message_alice_result.return_value() { panic!("{:?}",e) };


            let send_message_bob = send_message!("Bob" -> "Alice": "Hello, Alice! How are you?");

            let send_message_bob_result = call_dry_run!(bob: send_message_bob, pay 0);

            if let Err(e) = send_message_bob_result.return_value() { panic!("{:?}",e) };
            

            Ok(())
        }

    }
}
