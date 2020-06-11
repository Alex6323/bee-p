
use bee_ternary::{TritBuf, T1B1Buf, Tryte};
use bee_transaction::BundledTransaction;
use crate::api::{ApiImpl, Api};
use crate::deserialize::deserialize_hash_array;
use serde_json::Value;

pub async fn transaction_by_hash(json: Value) -> Result<impl warp::Reply, warp::Rejection> {

    match deserialize_hash_array(json["hashes"].as_array()) {

        Ok(hashes) => {

            let mut ret = Vec::new();
            for hash in hashes {
                match ApiImpl::transaction_by_hash(&hash) {
                    Some(tx_ref) => {
                        let mut trits = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
                        tx_ref.into_trits_allocated(&mut trits);
                        let tryte_string = trits
                            .chunks(3)
                            .map(|trits| char::from(Tryte::from_trits([trits.get(0).unwrap(), trits.get(1).unwrap(), trits.get(2).unwrap()])))
                            .collect::<String>();
                        ret.push((hash.to_string(), tryte_string) );
                    }
                    None => {
                        ret.push((hash.to_string(), String::from("")) );
                    }
                }
            }

            Ok(warp::reply::json(&ret))

        }

        Err(x) => {
            Ok(warp::reply::json(&x.msg ))
        }

    }

}


