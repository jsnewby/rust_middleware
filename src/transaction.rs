#![allow(missing_docs, unused_variables, trivial_casts)]

extern crate rust_base58;
use rust_base58::{ToBase58, FromBase58};

extern crate crypto;
//use crypto::{ symmetriccipher, buffer, aes, blockmodes };

extern crate hex;

extern crate blake2b;
use blake2b::blake2b;

use std::fs::File;
use std::io::Read;
use std::io::Write;

extern crate rand;
use rand::prelude::*;

extern crate rust_sodium;

pub fn hash(input: Vec<u8>) -> Vec<u8> {
    blake2b(32, &input).to_vec()
}

pub struct KeyPair {
    pub public_key: [u8; 32],
    pub private_key: [u8; 64],
}

impl KeyPair {

    pub fn new(public_key: [u8; 32], private_key: [u8; 64]) -> KeyPair {
        KeyPair{public_key: public_key, private_key: private_key}
    }

    // this one doesn't work, don't know why.
    pub fn generate_broken() -> Result<KeyPair, ()> {
        let mut rng = rand::OsRng::new().unwrap();
        let mut buf = [0u8; 64];
        rng.fill_bytes(&mut buf);
        let (priv_key, pub_key) = crypto::ed25519::keypair(&buf);
        Ok(KeyPair { public_key: pub_key.clone(),
                     private_key: priv_key.clone(), })
    }

    // this one does work. 
    pub fn generate() -> Result<KeyPair, ()> {
        rust_sodium::init()?;
        let (_pub, _priv) = rust_sodium::crypto::sign::ed25519::gen_keypair();
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&_pub[0..32]);
        let mut private_key = [0u8; 64];
        private_key.copy_from_slice(&_priv[0..64]);
        Ok(KeyPair { public_key: public_key, private_key: private_key })
    }
    
    pub fn sign(&self, val: &[u8]) -> Result<[u8; 64], String> {
        
        Ok(crypto::ed25519::signature(&val, &self.private_key))
    }

    pub fn verify(&self, signature: &[u8], message: &[u8])
                  -> Result<bool, String> {
        if !crypto::ed25519::verify(&message, &self.public_key, &signature) {
            return Err(String::from("Verification failed"));
        }
        Ok(true)
    }

    pub fn read_from_files(public_key_file: &String, private_key_file: &String,
                       password: &String) -> Result<KeyPair, Box<::std::error::Error>> {
        let mut public_key = String::new();
        let mut private_key = String::new();
        let mut file = File::open(public_key_file)?;
        let result = file.read_to_string(&mut public_key);
        file = File::open(private_key_file)?;
        file.read_to_string(&mut private_key)?;
        Ok(KeyPair::from_public_private_key_strings(&public_key, &private_key,
                                                    &password))
    }

    pub fn write_to_files(&self, public_key_file: &String, private_key_file: &String)
                          -> Result<(), ::std::io::Error> {
        let mut f = File::create(public_key_file)?;
        f.write_all(self.get_public_key_readable().as_bytes())?;
        f.flush()?;
        f = File::create(private_key_file)?;
        f.write_all(KeyPair::bytes_to_hex(self.private_key).as_bytes());
        f.flush();
        Ok(())
    }
        

    pub fn from_public_private_key_strings(public_key: &String,
                                           private_key: &String,
                                           password: &String)
                                           -> KeyPair {
        let bin_public_key = public_key.get(3..).unwrap().from_base58().unwrap();
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&bin_public_key[0..32]);
        let bin_private_key = hex::decode(private_key).unwrap();
        let mut private_key = [0u8; 64];
        private_key.copy_from_slice(&bin_private_key);
        KeyPair { private_key: private_key, 
                  public_key: public_key, }
    }

    pub fn get_public_key_readable(&self) -> String {
        format!("{}{}", String::from("ak$"), self.public_key.to_base58())
    }

    pub fn get_private_key_readable(&self) -> String {
        KeyPair::bytes_to_hex(self.private_key)
    }

    pub fn public_key_bytes_from_readable(public_key: String) -> Vec<u8> {
        let bin_public_key = public_key.get(3..).unwrap().from_base58().unwrap();
        bin_public_key
    }

    pub fn to_public_private_key_strings(&self) -> (String, String) {
        let asc_public_key = self.get_public_key_readable();
        let asc_private_key = KeyPair::bytes_to_hex(self.private_key);
        (asc_public_key, asc_private_key)
    }

    pub fn bytes_to_hex(bytes: [u8; 64]) -> String {
        let strs: Vec<String> = bytes.iter().map(|b|format!("{:02X}", b)).collect();
        strs.join("")
    }

}
