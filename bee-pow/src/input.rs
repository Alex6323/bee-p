use bee_common::constants::TRANSACTION_TRIT_LEN as INPUT_LEN;
use bee_common::constants::TRANSACTION_TRIT_LEN as TRANS_LEN;
use bee_common::Trit;

pub struct InputTrits(pub(crate) [Trit; INPUT_LEN]);

impl std::ops::Deref for InputTrits {
    type Target = [Trit; TRANS_LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
