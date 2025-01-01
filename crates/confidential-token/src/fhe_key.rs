use serde::{Deserialize, Serialize};
use tfhe::safe_serialization::{safe_deserialize, safe_serialize};
use tfhe::{
    generate_keys, prelude::*, ClientKey, CompressedPublicKey, CompressedServerKey, ConfigBuilder,
    PublicKey, ServerKey,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Status of FHE key serialization
pub enum SerializationStatus {
    /// The keys are not serialized
    NotSerialized,
    /// Some keys are serialized
    PartiallySerialized,
    /// All keys are serialized
    FullySerialized,
}

#[derive(Serialize, Deserialize)]
/// Configuration for FHE keys
pub struct FheKeyConfig {
    /// The serialized FHE public key
    pub fhe_public_key: Vec<u8>,
    /// The serialized FHE server key
    pub fhe_server_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Configuration for FHE keys generation
pub struct FheKeyGenConfig {
    /// The serialized FHE public key
    pub public_key: Vec<u8>,
    /// The serialized FHE server key
    pub server_key: Vec<u8>,
    /// The serialized FHE private key
    pub private_key: Vec<u8>,
    /// The serialization status of the keys
    pub serialization_status: SerializationStatus,
}

impl FheKeyGenConfig {
    /// Creates a new FHE key generation configuration
    pub fn new() -> Self {
        Self {
            public_key: Vec::new(),
            server_key: Vec::new(),
            private_key: Vec::new(),
            serialization_status: SerializationStatus::NotSerialized,
        }
    }

    /// Serializes the FHE keys
    pub fn serialize_keys(
        &mut self,
        client_key: &ClientKey,
        server_key: &CompressedServerKey,
        public_key: &CompressedPublicKey,
    ) {
        let public_key_result = serialize_key(public_key, "public");
        let server_key_result = serialize_key(server_key, "server");
        let private_key_result = serialize_key(client_key, "private");

        self.serialization_status = match (
            public_key_result.is_ok(),
            server_key_result.is_ok(),
            private_key_result.is_ok(),
        ) {
            (true, true, true) => SerializationStatus::FullySerialized,
            (false, false, false) => SerializationStatus::NotSerialized,
            _ => SerializationStatus::PartiallySerialized,
        };

        self.public_key = public_key_result.unwrap_or_else(|_| Vec::new());
        self.server_key = server_key_result.unwrap_or_else(|_| Vec::new());
        self.private_key = private_key_result.unwrap_or_else(|_| Vec::new());
    }

    /// Deserializes the FHE keys
    pub fn deserialize_keys(&self) -> Option<(ClientKey, ServerKey, PublicKey)> {
        let client_key: ClientKey = deserialize_key(&self.private_key, "private")?;
        let compressed_server_key: CompressedServerKey =
            deserialize_key(&self.server_key, "server")?;
        let compressed_public_key: CompressedPublicKey =
            deserialize_key(&self.public_key, "public")?;

        // decompress keys
        let server_key = compressed_server_key.decompress();
        let public_key = compressed_public_key.decompress();

        Some((client_key, server_key, public_key))
    }

    /// Checks if the FHE keys are fully serialized
    pub fn is_fully_serialized(&self) -> bool {
        self.serialization_status == SerializationStatus::FullySerialized
    }
}

/// Generates FHE keys
pub fn fhe_key_gen() -> FheKeyGenConfig {
    let config = ConfigBuilder::default().build();
    let (client_key, _) = generate_keys(config);
    let compressed_public_key = CompressedPublicKey::new(&client_key);
    let compressed_server_key = CompressedServerKey::new(&client_key);

    let mut fhe_keygen_config = FheKeyGenConfig::new();
    fhe_keygen_config.serialize_keys(&client_key, &compressed_server_key, &compressed_public_key);
    fhe_keygen_config
}

/// Serializes a FHE key
fn serialize_key<T>(key: &T, key_name: &str) -> Result<Vec<u8>, ()>
where
    T: tfhe::Versionize + tfhe::named::Named,
    T: serde::Serialize,
{
    let mut buffer = vec![];
    safe_serialize(key, &mut buffer, 1 << 30).map_err(|_| {
        eprintln!("Failed to serialize {} key", key_name);
        ()
    })?;
    Ok(buffer)
}

/// Deserializes a FHE key
fn deserialize_key<T>(key: &[u8], key_name: &str) -> Option<T>
where
    T: tfhe::Unversionize + tfhe::named::Named,
    T: serde::de::DeserializeOwned,
{
    safe_deserialize(key, 1 << 30)
        .map_err(|_| {
            eprintln!("Failed to deserialize {} key", key_name);
            ()
        })
        .ok()
}
