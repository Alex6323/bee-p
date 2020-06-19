// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_ternary::{T1B1Buf, TryteBuf};
use bee_transaction::{BundledTransactionField, Hash};

use serde_json::{Map, Value as JsonValue};

use std::convert::TryFrom;

pub struct DeserializedHashes(pub Vec<Hash>);
pub struct DeserializedHash(pub Hash);

impl TryFrom<&Vec<JsonValue>> for DeserializedHashes {
    type Error = &'static str;
    fn try_from(value: &Vec<JsonValue>) -> Result<Self, Self::Error> {
        let mut ret = Vec::new();
        for hash in value {
            ret.push(DeserializedHash::try_from(hash)?.0);
        }
        Ok(DeserializedHashes(ret))
    }
}

impl TryFrom<&JsonValue> for DeserializedHash {
    type Error = &'static str;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(tryte_str) => match TryteBuf::try_from_str(tryte_str) {
                Ok(buf) => match Hash::try_from_inner(buf.as_trits().encode::<T1B1Buf>()) {
                    Ok(hash) => Ok(DeserializedHash(hash)),
                    Err(_err) => Err("String has invalid size"),
                },
                Err(_err) => Err("String contains invalid characters"),
            },
            None => Err("No string provided"),
        }
    }
}

pub fn json_success(data: Map<String, JsonValue>) -> JsonValue {
    let mut response = Map::new();
    response.insert(String::from("data"), JsonValue::Object(data));
    JsonValue::Object(response)
}

pub fn json_error(msg: &str) -> JsonValue {
    let mut message = Map::new();
    message.insert(String::from("message"), JsonValue::String(String::from(msg)));
    let mut response = Map::new();
    response.insert(String::from("error"), JsonValue::Object(message));
    JsonValue::Object(response)
}
