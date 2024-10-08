use serde::{Deserialize, Serialize};

use crate::header::BitcoinHeader;
use std::collections::BTreeMap;
use std::error::Error as ErrorTrait;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

pub mod test_json;

const TEST_DATA_PATH: &str =
    "/Users/hra/Workspace/Code/layerX/bitcoin-fold/src/bitcoin/data/test_data.json";

#[derive(Error, Debug)]
#[error("transparent")]
pub struct BlockReaderError;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct BlockHeaderRpc {
    #[serde(with = "hex")]
    hash: Vec<u8>,
    confirmations: u32,
    height: u32,
    version: u32,
    #[serde(with = "hex")]
    merkleroot: Vec<u8>,
    time: u32,
    nonce: u32,
    #[serde(with = "hex")]
    bits: Vec<u8>,
    #[serde(with = "hex")]
    previousblockhash: Vec<u8>,
}

pub struct BlockReader {
    // sorted map of header with height as key
    headers_rpc: BTreeMap<u32, BlockHeaderRpc>,
}

impl BlockReader {
    pub fn new_from_file(data_file_path: &str) -> Result<BlockReader, Box<dyn ErrorTrait>> {
        let path = Path::new(data_file_path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let headers: Vec<BlockHeaderRpc> = serde_json::from_reader(reader)?;
        let mut headers_rpc = BTreeMap::new();
        for header in headers {
            headers_rpc.insert(header.height, header);
        }
        Ok(BlockReader { headers_rpc })
    }

    pub fn new_from_json(json: &str) -> Result<BlockReader, Box<dyn ErrorTrait>> {
        let headers: Vec<BlockHeaderRpc> = serde_json::from_str(json)?;
        let mut headers_rpc = BTreeMap::new();
        for header in headers {
            headers_rpc.insert(header.height, header);
        }
        Ok(BlockReader { headers_rpc })
    }

    fn into_internal(&self, header: BlockHeaderRpc) -> BitcoinHeader {
        let mut header_internal = BitcoinHeader {
            version: header.version,
            hash_prev_block: header.previousblockhash,
            hash_merkle_root: header.merkleroot,
            timestamp: header.time,
            target_bits: header.bits,
            nonce: header.nonce,
        };
        // Note: All returned hash values by json-RPC are reversed in reversed order, and need to be transformed back into internal format (reversed) before being used.
        // Ref: https://btcinformation.org/en/glossary/rpc-byte-order
        header_internal.hash_prev_block.reverse();
        header_internal.hash_merkle_root.reverse();
        header_internal.target_bits.reverse();

        header_internal
    }

    pub fn get_block_header(&self, height: u32) -> Result<BitcoinHeader, Box<dyn ErrorTrait>> {
        if let Some(header) = self.headers_rpc.get(&height) {
            let header = header.clone();

            let header_internal = self.into_internal(header.clone());

            Ok(header_internal)
        } else {
            Err(Box::new(BlockReaderError))
        }
    }

    pub fn get_block_headers(&self) -> Result<Vec<(u32, BitcoinHeader)>, Box<dyn ErrorTrait>> {
        let headers: Vec<(u32, BitcoinHeader)> = self
            .headers_rpc
            .iter()
            .map(|(height, header)| (height.clone(), self.into_internal(header.clone())))
            .collect();

        return Ok(headers);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_crypto_primitives::crh::sha256::{digest::Digest, Sha256};
    #[test]
    fn read_block_header_in_rpc_format() {
        let reader = BlockReader::new_from_file(TEST_DATA_PATH).unwrap();
        let header_internal = reader.get_block_header(838637).unwrap();
        assert_eq!(header_internal.nonce, 3878033683);
    }

    fn read_block_headers_in_rpc_format() {
        let reader = BlockReader::new_from_file(TEST_DATA_PATH).unwrap();
        let headers = reader.get_block_headers().unwrap();

        let (height, header_internal) = headers[0].clone();
        assert_eq!(header_internal.nonce, 3878033683);
        assert_eq!(height, 838637);

        let (height, header_internal) = headers[1].clone();
        assert_eq!(header_internal.nonce, 4255188596);
        assert_eq!(height, 838638);
    }

    #[test]
    fn read_block_headers_verify_chain_hash() {
        let reader = BlockReader::new_from_file(TEST_DATA_PATH).unwrap();
        let headers = reader.get_block_headers().unwrap();
        let mut prev_hash = headers[0].1.hash_prev_block.clone();
        for (weight, header) in headers {
            // verify chain hash
            assert_eq!(header.hash_prev_block, prev_hash);

            let header_bytes = header.to_bytes();
            prev_hash = Sha256::digest(Sha256::digest(header_bytes).to_vec()).to_vec();
        }
    }
}
