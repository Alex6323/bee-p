// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_ternary::{T1B1Buf, TritBuf};
use bee_transaction::BundledTransaction;

use serde_json::Value as JsonValue;

use bee_tangle::TransactionRef;

pub struct TransactionRefItem(pub TransactionRef);

impl From<&TransactionRefItem> for JsonValue {
    fn from(value: &TransactionRefItem) -> Self {
        JsonValue::String(String::from(value))
    }
}

impl From<&TransactionRefItem> for String {
    fn from(value: &TransactionRefItem) -> Self {
        let mut tx_buf = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
        value.0.into_trits_allocated(&mut tx_buf);
        tx_buf.iter_trytes().map(|trit| char::from(trit)).collect::<String>()
    }
}
