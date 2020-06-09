use serde::{Serialize, Deserialize};
use bee_ternary::{TritBuf, T1B1Buf, TryteBuf, Tryte, Error};
use bee_transaction::{Hash, BundledTransactionField, BundledTransaction};
use crate::api::{ApiImpl, Api};

#[derive(Deserialize)]
pub struct TransactionByHashRequest {
    hashes: Vec<String> // should be Vec<Hash>
}

#[derive(Serialize)]
pub struct TransactionByHashResponse {
    pub hashes: Vec<(String, String)> // should be Vec<Hash, BundledTransaction>
}

fn deserialize_tryte_str(tryte_str: &str) -> Result<TritBuf, Error> {
    match TryteBuf::try_from_str(tryte_str) {
        Ok(buf) => Ok(buf.as_trits().encode::<T1B1Buf>()),
        Err(err) => Err(err)
    }
}

pub async fn transaction_by_hash(req: TransactionByHashRequest) -> Result<impl warp::Reply, warp::Rejection> {

    // deserialize provided hashes; if any of the provided hashes is invalid, reject request
    let mut hashes = Vec::new();
    for hash in req.hashes {
        match deserialize_tryte_str(&hash) {
            Ok(tryte_string) => {
                match Hash::try_from_inner(tryte_string) {
                    Ok(hash) => hashes.push(hash),
                    Err(_err) => return Ok(warp::reply::json(&String::from("invalid hash provided")))
                }
            }
            Err(_err) => return Ok(warp::reply::json(&String::from("invalid hash provided")))
        }
    }

    let mut response = Vec::new();
    for hash in hashes {

        match ApiImpl::transaction_by_hash(&hash) {
            Some(tx_ref) => {
                let mut trits = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
                tx_ref.into_trits_allocated(&mut trits);
                let tryte_string = trits
                    .chunks(3)
                    .map(|trits| char::from(Tryte::from_trits([trits.get(0).unwrap(), trits.get(1).unwrap(), trits.get(2).unwrap()])))
                    .collect::<String>();
                response.push((hash.to_string(), tryte_string) );
            }
            None => {
                response.push((hash.to_string(), String::from("")) );
            }
        }

    }

    Ok(warp::reply::json(&TransactionByHashResponse { hashes: response } ))
}