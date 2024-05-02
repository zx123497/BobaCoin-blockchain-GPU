use openssl::rsa::Rsa;
pub struct Client {
    pub public_key: String,
    pub private_key: String,
}

impl Client {
    pub fn new() -> Client {
        let (public_key, private_key) = generate_keypair();
        Client {
            public_key: public_key,
            private_key: private_key,
        }
    }
}

fn generate_keypair() -> (String, String) {
    let rsa = Rsa::generate(2048).unwrap();
    let public_key = hex::encode(rsa.public_key_to_pem().unwrap());
    let private_key = hex::encode(rsa.private_key_to_pem().unwrap());
    (public_key, private_key)
}
