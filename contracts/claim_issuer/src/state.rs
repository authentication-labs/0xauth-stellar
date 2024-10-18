use soroban_sdk::{contracttype, contracterror, Address, Bytes, BytesN, Vec, U256};

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
    pub topic: U256,
    pub scheme: U256,
    pub issuer: Address,
    pub issuer_wallet: Address,
    pub signature: Bytes,
    pub data: Bytes,
    pub uri: Bytes,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    KeyNotFound = 2,
    InvalidKeyPurpose = 3,
    InvalidKeyType = 4,
    KeyConflict = 5,
    IndexOutOfBounds = 6,
    ClaimNotFound = 7,
    KeyDoesNotHavePurpose = 8,
    ClaimAlreadyRevoked = 9,
    InsufficientPermissions = 10,
    InvalidSignature = 11,
    InvalidClaim = 12,
    InvalidIssuer = 13,
    InvalidAddressBytes = 14,
}