#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod transmitter {

    use ink::storage::{Mapping, Lazy, traits::ManualKey};
    use ink::prelude::{string::String, vec::Vec};
    use ink::env::hash::Sha2x256;

    pub type Username = String;
    pub type Content = Vec<u8>;

    #[derive(PartialEq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum MessageType {
        Text,
        Email { subject: String },
        ReplyTo { hash: [u8;32] },
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
        timestamp: Timestamp,
    }

    #[derive(PartialEq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Sale {
        username: Username,
        to: AccountId,
        price: Balance,
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
        InsufficientBalance,
        NotContractOwner,
        UpgradeFailed,
        PaymentFailed {
            received: Balance,
            required: Balance,
            missing: Balance,
        },
        WithdrawFailed,
        NoBalance,
        NoAccount,
        CloseAccountFailed,
        UsernameAlreadyInSale,
        UsernameNotInSale,
        NoSalesForYou,
    }

    #[derive(Clone,Debug,PartialEq,scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct UserInfo {
        usernames: Option<Vec<Username>>,
        balance: Balance,
    }

    #[derive(PartialEq,scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct UsernameInfo {
        account_id: AccountId,
        messages: Option<Vec<Message>>,
        fee_payment_time: Timestamp,
    }

    #[derive(Debug,PartialEq,scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct OwnerInfo {
        account_id: AccountId,
        balance: Balance,
    }

    #[ink(storage)]
    pub struct Transmitter {
        users: Mapping<AccountId,UserInfo, ManualKey<1>>,
        usernames: Mapping<Username,UsernameInfo, ManualKey<2>>,
        // messages: Mapping<Username,Vec<Message>>,
        // balances: Mapping<AccountId,Balance>,
        sale_offers: Lazy<Option<Vec<Sale>>, ManualKey<3>>,
        owner: OwnerInfo,
        registration_fee: Balance,
        // fee_payment_dates: Mapping<Username,Timestamp>,
    }

    impl Transmitter {

        /// Constructor.
        #[ink(constructor)]
        pub fn new() -> Transmitter {
            Transmitter {
                usernames: Mapping::new(),
                users: Mapping::new(),
                // messages: Mapping::new(),
                // balances: Mapping::new(),
                sale_offers: Lazy::new(),
                owner: OwnerInfo { account_id: Self::env().caller(), balance: 0 },
                registration_fee: 1,
                // fee_payment_dates: Mapping::new(),
            }
        }

        /// Tells you the fee for registering a username.
        #[ink(message)]
        pub fn check_fee(&self) -> Balance {
            self.registration_fee
        }

        /// Attempts to register a new name connected to your account id.
        /// The correct registration fee must be paid (use 'get_registration_fee').
        /// If the payment does not equal the fee, the remainder is stored in your account's balance.
        #[ink(message,payable)]
        pub fn register_username(&mut self, name: String) -> Result<(),Error> {

            let transferred = self.env().transferred_value();
            let timestamp = self.env().block_timestamp();

            if let Some(_) = self.usernames.get(&name) {

                return Err(Error::NameTaken);

            }

            let mut user_balance: Balance = 0;

            if transferred > self.registration_fee {

                self.owner.balance += self.registration_fee;

                user_balance += transferred - self.registration_fee;

            } else if transferred < self.registration_fee {

                user_balance += transferred;

                let new_user_info = UserInfo { usernames: None, balance: user_balance };

                self.users.insert(&self.env().caller(), &new_user_info);

                return Err(Error::PaymentFailed {
                    received: transferred,
                    required: self.registration_fee,
                    missing:  self.registration_fee - transferred
                });

            }

            if let Some(user_info) = self.users.get(&self.env().caller()) {

                let mut usernames = user_info.usernames.unwrap_or(Vec::new());

                usernames.push(name.clone());

                let balance = user_info.balance + user_balance;

                let new_user_info = UserInfo {
                    usernames: Some(usernames),
                    balance,
                };

                self.users.insert(&self.env().caller(), &new_user_info);


                let new_username_info = UsernameInfo {
                    account_id: self.env().caller(),
                    messages: None,
                    fee_payment_time: timestamp,
                };

                self.usernames.insert(&name, &new_username_info);

                return Ok(());

            } else {


                let mut usernames = Vec::<Username>::new();

                usernames.push(name.clone());

                let new_user_info = UserInfo { usernames: Some(usernames), balance: user_balance };

                self.users.insert(&self.env().caller(), &new_user_info);


                let new_username_info = UsernameInfo {
                    account_id: self.env().caller(),
                    messages: None,
                    fee_payment_time: timestamp,
                };

                self.usernames.insert(&name, &new_username_info);

                return Ok(());

            }

        }

        /// Lists the names registered to your account.
        #[ink(message)]
        pub fn get_usernames(&self) -> Result<Vec<Username>,Error> {

            if let Some(user_info) = self.users.get(&self.env().caller()) {

                if let Some(usernames) = user_info.usernames {

                    return Ok(usernames);

                } else {

                    return Err(Error::NoNames);

                }

            } else {

                return Err(Error::NoAccount);

            }
        }

        /// Attempts to state the balance associated to your account.
        #[ink(message)]
        pub fn get_balance(&self) -> Result<Balance,Error> {

            if let None = self.users.get(&self.env().caller()) {

                return Err(Error::NoAccount);

            }
        
            if let Some(user_info) = self.users.get(&self.env().caller()) {

                return Ok(user_info.balance);

            } else {

                return Ok(0);

            }
        }

        /// Attempts to send a message to another user using one of your names.
        /// The name from which you wish the message to be sent must be specified.
        #[ink(message)]
        pub fn send_message(&mut self, from: Username, to: Username, mtype: MessageType, content: Content) -> Result<(),Error> {

            let timestamp = self.env().block_timestamp();

            if let Some(username_info) = self.usernames.get(&from) {

                if username_info.account_id != self.env().caller() {

                    return Err(Error::WrongAccount(from));

                }

                if let Some(username_info) = self.usernames.get(&to) {

                    let mut messages = username_info.messages.unwrap_or(Vec::new());

                    let mut to_be_hashed = Vec::<u8>::new();
                    to_be_hashed.extend(self.env().block_number().to_be_bytes());
                    to_be_hashed.extend(content.clone().iter()); // Mayber hashing only the message content is enough?

                    let hash = self.env().hash_bytes::<Sha2x256>(&to_be_hashed);

                    messages.push( Message { from, mtype, content, hash, timestamp });

                    let new_username_info = UsernameInfo {
                        account_id: username_info.account_id,
                        messages: Some(messages),
                        fee_payment_time: username_info.fee_payment_time,
                    };

                    self.usernames.insert(&to, &new_username_info);

                    return Ok(());

                } else {

                    return Err(Error::NameNonexistent(to));

                }

                
            } else {

                return Err(Error::NameNonexistent(from));

            }

        }

        /// Attempts to make all the messages that were sent to a specific name of yours available.
        #[ink(message,payable)]
        pub fn get_all_messages(&self, belonging_to: Username) -> Result<Vec<Message>,Error> {
            
            if let Some(username_info) = self.usernames.get(&belonging_to) {

                if self.env().caller() != username_info.account_id {

                    return Err(Error::WrongAccount(belonging_to));

                }

                if let Some(messages) = username_info.messages {

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

        /// Attempts to find and delete the specified message. The account name and message hash must be specified.
        #[ink(message)]
        pub fn delete_message(&mut self, belonging_to: Username, hash: [u8;32]) -> Result<(),Error> {

            if let Some(username_info) = self.usernames.get(&belonging_to) {

                if username_info.account_id != self.env().caller() {

                    return Err(Error::WrongAccount(belonging_to));

                }

                if let Some(mut messages) = username_info.messages {

                    let mut msg_pos = None;

                    for (pos,message) in messages.iter().enumerate() {

                        if message.hash == hash {

                            msg_pos = Some(pos);

                        } 

                    }

                    if let Some(pos) = msg_pos {

                        messages.remove(pos);

                        let username_info = UsernameInfo {
                            account_id: self.env().caller(),
                            messages: if messages.len() == 0 { None } else { Some(messages) },
                            fee_payment_time: username_info.fee_payment_time,
                        };

                        self.usernames.insert(&belonging_to, &username_info);

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

        /// Removes all messages that are in sotrage. This operation is not undoable, so proceed with caution.
        #[ink(message)]
        pub fn delete_all_messages(&mut self, username: Username) -> Result<(),Error> {

            if let Some(mut username_info) = self.usernames.get(&username) {

                if username_info.account_id != self.env().caller() {

                    return Err(Error::WrongAccount(username));

                }

                username_info.messages = None;

                self.usernames.insert(&username, &username_info);

                return Ok(());

            } else {

                return Err(Error::NameNonexistent(username));

            }
        }

        /// Attempts to send the balance associated to your account back to you.
        #[ink(message)]
        pub fn withdraw_balance(&mut self) -> Result<(),Error> {

            if let Some(mut user_info) = self.users.get(&self.env().caller()) {

                if user_info.balance == 0 {

                    return Err(Error::NoBalance);

                }

                if let Err(_) = self.env().transfer(self.env().caller(), user_info.balance) {

                    return Err(Error::WithdrawFailed);

                } else {

                    user_info.balance = 0;

                    self.users.insert(&self.env().caller(), &user_info);

                    return Ok(());

                }

            } else {

                return Err(Error::NoBalance);

            }
        }

        /// Makes a sale offer to the specified user. A 5% fee is charged.
        #[ink(message)]
        pub fn sell_username_to(&mut self, username: Username, to: AccountId, price: Balance) -> Result<(),Error> {

            if let Some(username_info) = self.usernames.get(&username) {

                if username_info.account_id != self.env().caller() {

                    return Err(Error::WrongAccount(username));

                }

                if let Some(sale_offers) = self.sale_offers.get() {

                    if let Some(mut sale_offers) = sale_offers {

                        for sale in sale_offers.iter() {

                            if sale.username == username {
        
                                return Err(Error::UsernameAlreadyInSale);
        
                            }
        
                        }
    
                        sale_offers.push(Sale { username, to, price });
    
                        self.sale_offers.set(&Some(sale_offers));
    
                        return Ok(());

                    } else {

                        let mut sale_offers = Vec::<Sale>::new();

                        sale_offers.push(Sale { username, to, price });

                        self.sale_offers.set(&Some(sale_offers));

                        return Ok(());

                    }

                } else {

                    let mut sale_offers = Vec::<Sale>::new();

                    sale_offers.push(Sale { username, to, price });

                    self.sale_offers.set(&Some(sale_offers));

                    return Ok(());

                }


            } else {

                return Err(Error::NameNonexistent(username));

            }

        }

        /// Cancels the sale offer of the specified username.
        #[ink(message)]
        pub fn cancel_sale(&mut self, username: Username) -> Result<(),Error> {

            if let Some(username_info) = self.usernames.get(&username) {

                if username_info.account_id != self.env().caller() {

                    return Err(Error::WrongAccount(username));

                }

                if let Some(sale_offers) = self.sale_offers.get() {

                    if let Some(mut sale_offers) = sale_offers {

                        let mut sale_pos: Option<usize> = None;

                        for (pos, sale) in sale_offers.iter().enumerate() {
    
                            if sale.username == username {
    
                                sale_pos = Some(pos);
    
                                break;
    
                            }
    
                        }
    
                        if let Some(pos) = sale_pos {
    
                            sale_offers.remove(pos);

                            if sale_offers.len() == 0 {

                                self.sale_offers.set(&None);

                            } else {

                                self.sale_offers.set(&Some(sale_offers));

                            }
    
                            return Ok(());
    
                        } else {
    
                            return Err(Error::UsernameNotInSale);
    
                        }

                    } else {

                        return Err(Error::UsernameNotInSale);

                    }
                    

                } else {

                    return Err(Error::UsernameNotInSale);

                }

            } else {

                return Err(Error::NameNonexistent(username));

            }

        }

        /// Gets any sale propositions made to you.
        #[ink(message)]
        pub fn get_sale_propositions(&mut self) -> Result<Vec<Sale>, Error> {
            
            let sale_offers = self.sale_offers.get();

            if let Some(sale_offers) = sale_offers {

                if let Some(sale_offers) = sale_offers {

                    let mut sales_to_user = Vec::<Sale>::new();

                    for sale in sale_offers.iter() {
    
                        if sale.to == self.env().caller() {
    
                            sales_to_user.push(Sale { username: sale.username.clone(), to: sale.to, price: sale.price } );
    
                        }
    
                    }
    
                    if sales_to_user.len() == 0 {
    
                        return Err(Error::NoSalesForYou);
    
                    } else {
    
                        return Ok(sales_to_user);
    
                    }

                } else {

                    return Err(Error::NoSalesForYou);

                }

            } else {

                return Err(Error::NoSalesForYou);

            }

        }

        /// Executes a proposed sale.
        #[ink(message,payable)]
        pub fn buy_username(&mut self, username: Username) -> Result<(),Error> {
            todo!()
        }

        /// A sale proposition made to you is cancelled.
        #[ink(message)]
        pub fn refuse_to_buy(&mut self, username: Username) -> Result<(),Error> {
            todo!()
        }

        /// Attempts to close your account. Any remaining balance will be sent back to you.
        #[ink(message)]
        pub fn close_account(&mut self) -> Result<(),Error> {
            if let Some(user_info) = self.users.get(&self.env().caller()) {

                if user_info.balance > 0 {

                    if let Err(_) = self.env().transfer(self.env().caller(), user_info.balance) {

                        return Err(Error::CloseAccountFailed);

                    }

                }
            
                if let Some(usernames) = user_info.usernames {

                    for username in usernames.iter() {

                        self.usernames.remove(username);
    
                    }

                }

                self.users.remove(&self.env().caller());

                return Ok(());

            } else {

                return Err(Error::NoAccount);

            }
        }

        /// Transfers the contract ownership. Can only be called by the current owner.
        #[ink(message)]
        pub fn co_transfer_contract_ownership(&mut self, new_owner: AccountId) -> Result<(),Error> {

            if self.env().caller() == self.owner.account_id {

                self.owner.account_id = new_owner;

                return Ok(());

            } else {

                return Err(Error::NotContractOwner);

            }

        }

        /// Updated the contract code. Can only be called by the contract owner.
        #[ink(message)]
        pub fn co_set_code(&mut self, code_hash: ink::primitives::Hash) -> Result<(),Error> {

            if self.env().caller() == self.owner.account_id {

                match self.env().set_code_hash(&code_hash) {

                    Ok(()) => {

                        return Ok(());

                    },
                    Err(_) => {

                        return Err(Error::UpgradeFailed)

                    }

                }


            } else {

                return Err(Error::NotContractOwner);

            }

        }

        /// Sets a new value for the username registration fee. Can only be called by the contract owner.
        #[ink(message)]
        pub fn co_set_fee(&mut self, new_fee: Balance) -> Result<(),Error> {

            if self.env().caller() == self.owner.account_id {

                self.registration_fee = new_fee;

                return Ok(());

            } else {

                return Err(Error::NotContractOwner);

            }

        }

        /// Withdraw the balance stored. Can only be called by the contract owner.
        #[ink(message)]
        pub fn co_owner_withdraw_all_balance(&mut self) -> Result<(),Error> {

            if self.owner.balance > 0 {

                if let Err(_) = self.env().transfer(self.owner.account_id, self.owner.balance) {

                    return Err(Error::WithdrawFailed);

                } else {

                    self.owner.balance = 0;

                    return Ok(());

                }

            } else {

                return Err(Error::NoBalance);

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

            if let Err(e) = transmitter.co_set_fee(0) {

                panic!("Error {:?} while setting registration fee.",e);

            };

            if let Err(e) = transmitter.register_username("Alice".into()) {
                panic!("Encountered error {:?} while registering Alice's name.",e)
            };

            if let Err(e) = transmitter.register_username("Bob".into()) {
                panic!("Encountered error {:?} while registering Bob's name.",e)
            };

            if let Err(e) = transmitter.send_message(
                "Alice".into(),
                "Bob".into(),
                MessageType::Text,
                "Hello, Bob!".into()
            ) {
                panic!("Encountered error {:?} while sending message to Bob.",e)
            };

            if let Err(e) = transmitter.send_message(
                "Alice".into(),
                "Bob".into(),
                MessageType::Text,
                "Have a nice day!".into()
            ) {
                panic!("Encountered error {:?} while sending message to Bob.",e)
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

                    panic!("Encountered error {:?} while getting Bob's messages.",e)

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

        use ink::env::call::DelegateCall;
        /// A helper function used for calling contract messages.
        use ink_e2e::{build_message, subxt::book::setup::client, Keypair};

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
                        .call(|transmitter| transmitter.register_username($name.into()))
                };
            }

            macro_rules! send_message {
                ($from:literal -> $to:literal : $content:literal) => {
                        build_message::<TransmitterRef>(contract_account_id.clone())
                            .call(|transmitter| transmitter.send_message(
                                $from.into(),
                                $to.into(),
                                MessageType::Text,
                                $content.into())
                            )
                };
            }

            // macro_rules! call_dry_run {

            //     (alice : $fn_name:tt, pay $amnt:tt) => {
            //         client.call_dry_run(
            //             &ink_e2e::alice(),
            //             &$fn_name,
            //             $amnt,
            //             None)
            //             .await
            //     };

            //     (bob : $fn_name:tt, pay $amnt:tt) => {
            //         client.call_dry_run(
            //             &ink_e2e::bob(),
            //             &$fn_name,
            //             $amnt,
            //             None)
            //             .await
            //     };

            // }

            macro_rules! get_names {
                () => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.get_usernames())

                };
            }

            macro_rules! get_all_messages {
                ($username:literal) => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.get_all_messages($username.into()))

                }
            }

            macro_rules! delete_all_messages {
                ($username:literal) => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.delete_all_messages($username.into()))

                }
            }

            macro_rules! delete_message {

                ($username:literal, $hash:tt) => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.delete_message($username.into(),$hash))

                }
            }

            macro_rules! get_balance {

                () => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.get_balance())

                }
            }

            macro_rules! withdraw_balance {

                () => {

                    build_message::<TransmitterRef>(contract_account_id.clone())
                        .call(|transmitter| transmitter.withdraw_balance())

                }

            }

            macro_rules! call_run {
                (alice : $fn_name:tt, pay $amnt:tt) => {
                    client.call(
                        &ink_e2e::alice(),
                        $fn_name,
                        $amnt,
                        None)
                        .await
                };

                (bob : $fn_name:tt, pay $amnt:tt) => {
                    client.call(
                        &ink_e2e::bob(),
                        $fn_name,
                        $amnt,
                        None)
                        .await
                };

            }

            // ----------------------------------------------------------------------

            // Alice registers a new username.

            let new_name_alice = new_name!("Alice");

            let new_name_alice_result = call_run!(alice: new_name_alice, pay 1);

            if let Err(e) = new_name_alice_result.expect("Error w/ 'new_name_alice'.").return_value() { panic!("{:?}",e) };


            // Bob also registers a new username.

            let new_name_bob = new_name!("Bob");

            let new_name_bob_result = call_run!(bob: new_name_bob, pay 2);

            if let Err(e) = new_name_bob_result.expect("Error w/ 'new_name_bob'.").return_value() { panic!("{:?}",e) };


            // Alice wants to know which usernames belong to her.

            let get_user_names = get_names!();

            let get_user_names_result = call_run!(alice: get_user_names, pay 0);

            if let Err(e) = get_user_names_result.expect("Error w/ 'get_user_names'.").return_value() { panic!("{:?}",e) };


            // Alice sends a message to bob.

            let send_message_alice = send_message!("Alice" -> "Bob" : "Hello, Bob!" );

            let send_message_alice_result = call_run!(alice: send_message_alice, pay 0);

            if let Err(e) = send_message_alice_result.expect("Error w/ 'send_message_alice'").return_value() { panic!("{:?}",e) };


            // Bob downloads all of the messages he's received to far.

            let get_all_messages = get_all_messages!("Bob");

            let get_all_messages_result = call_run!(bob: get_all_messages, pay 0);

            match get_all_messages_result.expect("Error w/ 'get_all_messages (bob)'").return_value() {
                Ok(messages) => {

                    if messages.len() != 1 {

                        panic!("Error: incorrect number of messages received by bob.");

                    }

                },
                Err(e) => {

                    panic!(" Error {:?} while getting Bob's messages",e);

                }
            }


            // Bob replies to Alice's message.

            let send_message = send_message!("Bob" -> "Alice": "Hello, Alice! How are you?");

            let send_message_result = call_run!(bob: send_message, pay 0);

            if let Err(e) = send_message_result.expect("Error w/ 'send_message_bob.").return_value() { panic!("{:?}",e) };


            // Alice downloads all of the messages she's received.

            let get_all_messages = get_all_messages!("Alice");

            let get_all_messages_result = call_run!(alice: get_all_messages, pay 0);

            match get_all_messages_result.expect("Error w/ 'get_all_messages (alice)'").return_value() {
                Ok(messages) => {

                    if messages.len() != 1 {

                        panic!("Error: incorrect number of messages received by alice.");

                    }

                },
                Err(e) => {

                    panic!(" Error {:?} while getting Alice's messages",e);

                }
            }


            // Alice decides that Bob's message isn't critically important, and so she deletes it.

            let delete_all_messages = delete_all_messages!("Alice");

            let delete_all_messages_result = call_run!(alice: delete_all_messages, pay 0);

            if let Err(e) = delete_all_messages_result.expect("Error w/ 'delete_all_messages (alice)'").return_value() {

                panic!("{:?}",e);

            }


            // Bob thinks the same about Alice's message.

            let delete_all_messages = delete_all_messages!("Bob");

            let delete_all_messages_result = call_run!(bob: delete_all_messages, pay 0);

            if let Err(e) = delete_all_messages_result.expect("Error w/ 'delete_all_messages (bob)'").return_value() {

                panic!("{:?}",e);
                
            }


            // Bob forgot to tell alice something.

            let send_message = send_message!("Bob" -> "Alice":
                "I forgot to tell you: you left your car keys in your car and now it's locked. What are you gonna do?");

            let send_message_result = call_run!(bob: send_message, pay 0);

            if let Err(e) = send_message_result.expect("Error w/ 'send_message' (bob, 2)").return_value() {

                panic!("{:?}",e);

            }


            // Alice reads her messages again. 

            let mut message_hash = [0u8;32];

            let read_messages = get_all_messages!("Alice");

            let read_messages_result = call_run!(alice: read_messages, pay 0);

            match read_messages_result.expect("Error w/ 'get_all_messages' (alice, 2)").return_value() {
                
                Ok(messages) => {

                    if messages.len() != 1 {

                        panic!("Alice was supposed to have received only one message. Instead she got {}.",messages.len());

                    }

                    message_hash = messages[0].hash;

                },
                Err(e) => {

                    panic!("{:?}",e);

                }

            }


            // To be sure that the message was transmitted correctly, and not a random error, she checks her messages again.

            let read_messages = get_all_messages!("Alice");

            let read_messages_result = call_run!(alice: read_messages, pay 0);

            match read_messages_result.expect("Error w/ 'get_all_messages' (alice, 3)").return_value() {
                
                Ok(messages) => {

                    if messages.len() != 1 {

                        panic!("Alice was supposed to have received only one message. Instead she got {}.",messages.len());

                    }

                    message_hash = messages[0].hash;

                },
                Err(e) => {

                    panic!("{:?}",e);

                }

            }


            // Alice, in a fit of rage, decides to delete that message specifically, using the message hash.

            let delete_message = delete_message!("Alice",message_hash);

            let delete_message_result = call_run!(alice: delete_message, pay 0);

            if let Err(e) = delete_message_result.expect("Error w/ 'delete_message' (alice).").return_value() {

                panic!("{:?}",e);

            }


            // Bob decides that he has done his duty in good conscience and checks his balance.

            let check_balance = get_balance!();

            let check_balance_result = call_run!(bob: check_balance, pay 0);

            match check_balance_result.expect("Error w/ 'check_balance' (bob)").return_value() {
                Ok(balance) => {

                    if balance != 1 {

                        panic!("Bob's balance should be 1.");

                    }

                },
                Err(e) => {

                    panic!("{:?}",e);

                }
            }

            
            // Bob is obsessed with money. He checks his balance again.

            let check_balance = get_balance!();

            let check_balance_result = call_run!(bob: check_balance, pay 0);

            match check_balance_result.expect("Error w/ 'check_balance' (bob, 2)").return_value() {
                Ok(balance) => {

                    if balance != 1 {

                        panic!("Bob's balance should be 1.");

                    }

                },
                Err(e) => {

                    panic!("{:?}",e);

                }
            }


            // After considering pros and cons for a while, Bob decides do withdraw his money.

            let withdraw_balance = withdraw_balance!();

            let withdraw_balance_result = call_run!(bob: withdraw_balance, pay 0);

            if let Err(e) = withdraw_balance_result.expect("Error w/ 'withdraw_balance' (bob).").return_value() {

                panic!("{:?}",e);

            }


            // Alice has thought of a really cool username that bob might want. She decides to register it,
            // just for the pleasure of making BOb angry.

            let register_username = new_name!("Bob_resembles_a_sponge");

            let register_username_result = call_run!(alice: register_username, pay 1);

            if let Err(e) = register_username_result.expect("Error w/ 'register_username' (alice, 2).").return_value() {

                panic!("{:?}",e);

            }


            // Alice decides she would like to sell the username to Bob.

            // let make_sale_proposition = sell_username_to!("Bob_resembles_a_sponge",bob,100);

            // let make_sale_proposition_result = call_run!(alice: make_sale_proposition, pay 0);

            // if let Err(e) = make_sale_proposition_result.expect("Error w/ 'make_sale_proposition' (alice).").return_value() {

            //     panic!("{:?}",e);

            // }

            // --> UNFORTUNATELY DON'T KNOW HOW TO FIND BOB's ACCOUNT_ID!!!!


            // Bob has heard from Alice that she has a username to sell to him.

            let get_sale_propositions = build_message::<TransmitterRef>(contract_account_id.clone())
                .call(|transmitter| transmitter.get_sale_propositions());

            let get_sale_propositions_result = call_run!(bob: get_sale_propositions, pay 0);

            match get_sale_propositions_result.expect("Error w/ 'get_sale_propositions' (bob).").return_value() {

                Ok(sales) => {

                    if sales.len() != 1 {

                        panic!("The amount of sales for bob should be 1. Instead, they are {}.",sales.len());

                    }

                },
                Err(e) => {

                    panic!("{:?}",e);

                }

            }

            Ok(())
        }

    }
}
