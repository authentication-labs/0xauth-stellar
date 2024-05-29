use soroban_sdk::{contracttype, Address, Bytes, BytesN, Vec};

#[contracttype]
#[derive(Clone,Copy)]
pub enum KeyType {
    ECDSA = 1,
    RSA = 2,
}

impl TryFrom<u32> for KeyType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(KeyType::ECDSA),
            2 => Ok(KeyType::RSA),
            _ => Err(()),
        }
    }
}


#[contracttype]
#[derive(Clone, Copy, PartialEq)]
pub enum KeyPurpose {
    Management = 1,
    Action = 2,
    Claim = 3,
    Encryption = 4,
}

impl TryFrom<u32> for KeyPurpose {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(KeyPurpose::Management),
            2 => Ok(KeyPurpose::Action),
            3 => Ok(KeyPurpose::Claim),
            4 => Ok(KeyPurpose::Encryption),
            _ => Err(()),
        }
    }
}

#[contracttype]
#[derive(Clone)]
pub struct Key {
    pub purposes: Vec<KeyPurpose>,
    pub key_type: KeyType,
    pub key: BytesN<32>,
}

#[contracttype]
#[derive(Clone)]
pub struct Claim {
    pub topic: u32,
    pub scheme: u32,
    pub issuer: Address,
    pub signature: Bytes,
    pub data: Bytes,
    pub uri: Bytes,
}